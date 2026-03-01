use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use user_service::{AddUserError, ChangePasswordError, PasswordRequirement, UserService};

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

async fn setup_service(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) -> UserService {
    let pool = pool_options
        .connect_with(options.foreign_keys(true))
        .await
        .unwrap();
    UserService::new(pool)
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_add_user_success(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    // GIVEN
    let service = setup_service(pool_options, options).await;

    // WHEN
    let result = service
        .add_user("testuser", "test@example.com", "ValidPassword123!")
        .await;
    assert!(result.is_ok());

    // THEN
    let user = service.user_by_email("test@example.com").await.unwrap();
    assert_eq!(user.username, "testuser");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_add_user_already_exists(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    service
        .add_user("testuser", "test@example.com", "ValidPassword123!")
        .await
        .unwrap();

    // WHEN adding another user with the same email
    let result = service
        .add_user("testuser2", "test@example.com", "ValidPassword123!")
        .await;

    // THEN
    assert!(matches!(result, Err(AddUserError::AlreadyExists)));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_add_user_password_requirements_too_short(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    let password = "Short1";

    // WHEN
    let result = service
        .add_user("testuser", "test@example.com", password)
        .await;

    // THEN
    assert!(matches!(
        result,
        Err(AddUserError::PasswordRequirement(reqs)) if reqs.contains(&PasswordRequirement::PasswordTooShort{ min_length: 14 })
    ));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_add_user_password_requirements_no_lowercase(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    let password = "VALID_BUT_NO_LOWERCASE_123";

    // WHEN
    let result = service
        .add_user("testuser", "test@example.com", password)
        .await;

    // THEN
    assert!(matches!(
        result,
        Err(AddUserError::PasswordRequirement(reqs)) if reqs.contains(&PasswordRequirement::NoLowerCase)
    ));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_add_user_password_requirements_no_uppercase(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    let password = "valid_but_no_uppercase_123";

    // WHEN
    let result = service
        .add_user("testuser", "test@example.com", password)
        .await;

    // THEN
    assert!(matches!(
        result,
        Err(AddUserError::PasswordRequirement(reqs)) if reqs.contains(& PasswordRequirement::NoUppercase)
    ));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_add_user_password_requirements_no_digit(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    let password = "ValidPasswordButNoDigit!";

    // WHEN
    let result = service
        .add_user("testuser", "test@example.com", password)
        .await;

    // THEN
    assert!(matches!(
        result,
        Err(AddUserError::PasswordRequirement(reqs)) if reqs.contains(&PasswordRequirement::NoDigit)
    ));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_add_user_password_requirements_no_special(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    let password = "ValidPasswordButNoSpecial123";

    // WHEN
    let result = service
        .add_user("testuser", "test@example.com", password)
        .await;

    // THEN
    assert!(matches!(
        result,
        Err(AddUserError::PasswordRequirement(reqs)) if reqs.contains(&PasswordRequirement::NoSpecial)
    ));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_get_user_by_email(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    service
        .add_user("getuser", "get@example.com", "ValidPassword123!")
        .await
        .unwrap();

    // WHEN getting user by email
    let user = service.user_by_email("get@example.com").await.unwrap();
    // THEN
    assert_eq!(user.username, "getuser");

    // WHEN no user with email, THEN no user
    assert!(
        service
            .user_by_email("nonexistent@example.com")
            .await
            .is_err()
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_get_user_by_id(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    let id = service
        .add_user("getuser", "get@example.com", "ValidPassword123!")
        .await
        .unwrap();

    // WHEN getting user by id
    let user = service.user_by_id(id).await.unwrap();
    // THEN
    assert_eq!(user.username, "getuser");

    // WHEN no user with id, THEN no user
    assert!(service.user_by_id(999).await.is_err());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_delete_user(pool_options: SqlitePoolOptions, options: SqliteConnectOptions) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    service
        .add_user("deleteuser", "delete@example.com", "ValidPassword123!")
        .await
        .unwrap();

    // WHEN an existing user is deleted
    service.delete_user("delete@example.com").await.unwrap();
    // THEN user is not present
    assert!(service.user_by_email("delete@example.com").await.is_err());

    // WHEN a non-existing user is deleted, THEN deleting is an error
    service
        .delete_user("nonexistent@example.com")
        .await
        .unwrap_err();
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_change_password_success(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    service
        .add_user("changepw", "changepw@example.com", "OldPassword123!")
        .await
        .unwrap();

    // WHEN channging password, THEN no error
    service
        .change_password(
            "changepw@example.com",
            "OldPassword123!",
            "NewValidPassword123!",
        )
        .await
        .unwrap();

    // Verify new password works by trying to change it again with the new password
    service
        .change_password(
            "changepw@example.com",
            "NewValidPassword123!",
            "AnotherNewPassword123!",
        )
        .await
        .unwrap();
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_change_password_wrong_current(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) {
    // GIVEN
    let service = setup_service(pool_options, options).await;
    service
        .add_user("username", "user@example.com", "OldPassword123!")
        .await
        .unwrap();

    // WHEN changing password with wrong old password
    let change_result = service
        .change_password(
            "user@example.com",
            "WrongOldPassword",
            "NewValidPassword123!",
        )
        .await;

    // THEN it's an error
    assert!(matches!(
        change_result,
        Err(ChangePasswordError::WrongCurrentPassword)
    ));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_change_password_no_user(
    pool_options: SqlitePoolOptions,
    options: SqliteConnectOptions,
) {
    // GIVEN no user
    let service = setup_service(pool_options, options).await;

    // WHEN changing password with wrong old password
    let change_result = service
        .change_password(
            "nouser@example.com",
            "WrongOldPassword",
            "NewValidPassword123!",
        )
        .await;

    // THEN it's an error
    assert!(matches!(
        change_result,
        Err(ChangePasswordError::UserNotFound)
    ));
}
