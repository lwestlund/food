use auth_service::AuthService;
use dioxus::fullstack::{FullstackContext, extract::FromRef};
use recipe_service::RecipeService;
use sqlx::SqlitePool;
use user_service::UserService;

#[derive(Clone)]
pub struct ServerState {
    pub user: UserService,
    pub auth: AuthService,
    pub recipe: RecipeService,
}

impl ServerState {
    pub fn new(pool: SqlitePool) -> Self {
        let user = UserService::new(pool.clone());
        let auth = AuthService::new(pool.clone());
        let recipe = RecipeService::new(pool);
        Self { user, auth, recipe }
    }
}

// Needed to be able to extract Database as an axum::extract::State in server functions.
// If a Database isn't added to the axum router, this will panic at runtime and cause a 500 for the
// client.
impl FromRef<FullstackContext> for ServerState {
    fn from_ref(state: &FullstackContext) -> Self {
        state.extension::<ServerState>().unwrap()
    }
}
