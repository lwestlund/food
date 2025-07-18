set dotenv-load

@default:
    just --list

database-url := `grep DATABASE_URL .env | awk -F':' '{ print $2 }'`
backend-log := "axum=info,sqlx=info,trace"

db-setup:
    sqlx database setup

db-reset:
    sqlx database reset

db-interactive:
    sqlite3 {{database-url}}

backend $RUST_LOG=backend-log:
    RUST_LOG=sqlx=info,debug cargo run --bin food-backend

backend-watch $RUST_LOG=backend-log:
    cargo watch --watch food-backend -- cargo run --bin food-backend

frontend:
    cargo run --bin food-frontend

frontend-watch:
    cargo watch --watch food-frontend -- cargo run --bin food-frontend
