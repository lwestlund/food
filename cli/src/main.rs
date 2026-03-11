use clap::Parser as _;
use food::api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    {
        // Target the locally running Dioxus fullstack backend by formatting the server url and
        // making it a `&'static str`.
        let url = format!("http://localhost:{port}", port = args.port);
        let url: &'static str = Box::leak(url.into_boxed_str());
        dioxus::fullstack::set_server_url(url);
    }

    args.command.run().await?;

    Ok(())
}

#[derive(clap::Parser)]
struct Args {
    #[arg(long, default_value_t = 8080)]
    port: u16,

    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    AddUser(AddUser),
    DeleteUser(DeleteUser),
    ChangePassword(ChangePassword),
}

impl Command {
    async fn run(self) -> anyhow::Result<()> {
        match self {
            Self::AddUser(add_user) => {
                api::user::add_user(add_user.username, add_user.email, add_user.password).await?;
                println!("User added successfully");
            }
            Self::DeleteUser(delete_user) => {
                api::user::delete_user(delete_user.email).await?;
                println!("User deleted successfully");
            }
            Self::ChangePassword(change_password) => {
                api::user::change_password(
                    change_password.email,
                    change_password.current_password,
                    change_password.new_password,
                )
                .await?;
                println!("Password changed successfully");
            }
        }
        Ok(())
    }
}

#[derive(clap::Args)]
struct AddUser {
    username: String,
    email: String,
    password: String,
}

#[derive(clap::Args)]
struct DeleteUser {
    email: String,
}

#[derive(clap::Args)]
struct ChangePassword {
    email: String,
    current_password: String,
    new_password: String,
}
