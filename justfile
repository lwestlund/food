set dotenv-load

@default:
    just --list

database-url := `grep DATABASE_URL .env | awk -F':' '{ print $2 }'`

dev-init:
    cargo binstall sqlx-cli dioxus-cli

db-setup:
    sqlx database setup

db-reset:
    sqlx database reset

db-interactive:
    sqlite3 {{database-url}}

test:
    cargo test --features web
    cargo test --features server

lint:
    cargo clippy --all-targets --features web
    cargo clippy --all-targets --features server
