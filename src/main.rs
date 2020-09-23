//! Runs a command in all subaccounts of an AWS Organization
use colored::Colorize;
use futures::stream::{self, StreamExt};
use std::error::Error;
use structopt::StructOpt;
mod aws;
use aws::{Account, Aws, Cmd};
mod exec;
use anyhow::anyhow;
use exec::exec;

fn nonempty_command(src: &str) -> Result<String, anyhow::Error> {
    let trim = src.trim();
    if trim.is_empty() {
        return Err(anyhow!("Please provide a valid non-empty command"));
    }
    Ok(trim.into())
}

fn role_name(src: &str) -> Result<String, anyhow::Error> {
    let role = src.trim();
    if role.starts_with("arn:aws:") {
        return Err(anyhow!("Please provide a role name, not a role arn"));
    }
    Ok(role.into())
}

/// Runs a command in all subaccounts of an AWS Organization
#[derive(Debug, StructOpt, Clone, PartialEq)]
struct Opts {
    /// Name of IAM role to assume
    #[structopt(short, long, parse(try_from_str = role_name))]
    role: String,
    /// Command to execute
    #[structopt(short, long, parse(try_from_str = nonempty_command))]
    command: String,
}

async fn run<A>(
    opts: Opts,
    aws: A,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>>
where
    A: Aws + Clone + Sync + Send + 'static,
{
    let results = stream::iter(aws.accounts().await?.into_iter().map(
        |Account { id, name, .. }| {
            let Opts { role, command } = opts.clone();
            let aws = aws.clone();
            let id2 = id.clone();
            let role2 = role.clone();
            tokio::spawn(async move {
                println!(
                    "working on {} in thread {:?}",
                    id,
                    std::thread::current().id()
                );
                match aws.assume_role(id2, role2).await {
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
                    Ok(creds) => match exec(creds, id.clone(), command.clone()).await {
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
            })
        },
    ))
    .buffer_unordered(8)
    .collect::<Vec<_>>();
    results.await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    env_logger::init();
    let opts = Opts::from_args();
    run(opts, Cmd).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws::Credentials;

    #[test]
    fn opts_errs_on_empty_command() {
        match Opts::from_iter_safe(&["aws-crossing", "-r", "role", "-c", ""]) {
            Err(e) => {
                assert!(format!("{}", e).contains("Please provide a valid non-empty command"))
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn opts_errs_on_role_arn() {
        match Opts::from_iter_safe(&[
            "aws-crossing",
            "-r",
            "arn:aws:iam::account-id:role/role-name-with-path",
            "-c",
            "test",
        ]) {
            Err(e) => {
                assert!(format!("{}", e).contains("Please provide a role name, not a role arn"))
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn opts_accepts_role_and_command() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        assert_eq!(
            Opts {
                role: "role-name-with-path".into(),
                command: "test".into()
            },
            Opts::from_iter_safe(&["aws-crossing", "-r", "role-name-with-path", "-c", "test"])?
        );
        Ok(())
    }

    #[tokio::test]
    async fn errs_if_accounts_fetch_fails() {
        #[derive(Clone)]
        struct FakeAws;
        #[async_trait::async_trait]
        impl Aws for FakeAws {
            async fn accounts(
                &self
            ) -> Result<Vec<Account>, Box<dyn Error + Send + Sync + 'static>> {
                Err(anyhow!("boom".to_string()).into())
            }

            async fn assume_role(
                &self,
                _: String,
                _: String,
            ) -> Result<Credentials, Box<dyn Error + Send + Sync + 'static>> {
                Ok(Credentials {
                    access_key_id: "xxx".into(),
                    secret_access_key: "yyy".into(),
                    session_token: "zzz".into(),
                })
            }
        }

        match run(
            Opts {
                role: "role".into(),
                command: "test".into(),
            },
            FakeAws,
        )
        .await
        {
            Err(e) => assert_eq!("boom", format!("{}", e)),
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn runs_commands() {
        #[derive(Clone)]
        struct FakeAws;
        #[async_trait::async_trait]
        impl Aws for FakeAws {
            async fn accounts(
                &self
            ) -> Result<Vec<Account>, Box<dyn Error + Send + Sync + 'static>> {
                Ok(Vec::default())
            }

            async fn assume_role(
                &self,
                _: &str,
                _: &str,
            ) -> Result<Credentials, Box<dyn Error + Send + Sync + 'static>> {
                Ok(Credentials {
                    access_key_id: "xxx".into(),
                    secret_access_key: "yyy".into(),
                    session_token: "zzz".into(),
                })
            }
        }
        assert!(run(
            Opts {
                role: "role".into(),
                command: "test".into()
            },
            FakeAws
        )
        .await
        .is_ok())
    }
}
