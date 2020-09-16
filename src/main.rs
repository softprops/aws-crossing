//! Runs a command in all subaccounts of an AWS Organization
use colored::Colorize;
use futures::stream::{self, StreamExt};
use std::error::Error;
use structopt::StructOpt;
mod aws;
use aws::{Account, Aws, Cmd};
mod exec;
use exec::exec;

/// Runs a command in all subaccounts of an AWS Organization
#[derive(StructOpt, Clone)]
struct Opts {
    /// Name of IAM role to assume
    #[structopt(short, long)]
    role: String,
    /// Command to execute
    #[structopt(short, long)]
    command: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();
    let results = stream::iter(Cmd.accounts().await?.into_iter().map(
        |Account { id, name, .. }| {
            let Opts { role, command } = opts.clone();
            async move {
                match Cmd.assume_role(&id, &role).await {
                    Err(e) => {
                        eprintln!(
                            "{}",
                            format!(
                                "Failed to assume role {} in account {} ({})",
                                role.bold(),
                                name.bold(),
                                id.bold()
                            )
                            .red()
                        );
                        eprintln!("{}", e)
                    }
                    Ok(creds) => match exec(creds, &id, &command).await {
                        Err(e) => {
                            eprintln!(
                                "{}",
                                format!(
                                    "Error executing command as {} in account {} ({})",
                                    role.bold(),
                                    name.bold(),
                                    id.bold()
                                )
                                .red()
                            );
                            eprintln!("{}", e)
                        }
                        Ok(output) => println!("{}", output),
                    },
                }
            }
        },
    ))
    .buffer_unordered(8)
    .collect::<Vec<()>>();
    results.await;

    Ok(())
}
