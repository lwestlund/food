set dotenv-load

@default:
    just --list

database-url := `grep DATABASE_URL .env | awk -F':' '{ print $2 }'`

db-setup:
    sqlx database setup

db-reset:
    sqlx database reset

db-interactive:
    sqlite3 {{database-url}}
