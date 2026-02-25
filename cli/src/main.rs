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

    if let Err(e) = args.command.run().await {
        eprintln!("{e}");
        std::process::exit(1);
    }

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
                match api::user::add_user(add_user.username, add_user.email, add_user.password)
                    .await
                {
                    Ok(Ok(())) => println!("User added successfully"),
                    Ok(Err(err)) => anyhow::bail!("Error adding user: {err}"),
                    Err(err) => anyhow::bail!("{err}"),
                }
            }
            Self::DeleteUser(delete_user) => {
                if let Err(err) = api::user::delete_user(delete_user.email).await {
                    anyhow::bail!("Server error: {err}");
                }
                println!("User deleted successfully");
            }
            Self::ChangePassword(change_password) => {
                match api::user::change_password(
                    change_password.email,
                    change_password.current_password,
                    change_password.new_password,
                )
                .await
                {
                    Ok(Ok(())) => println!("Password changed successfully"),
                    Ok(Err(err)) => anyhow::bail!("Error changing password: {err}"),
                    Err(err) => anyhow::bail!("Server error: {err}"),
                }
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
