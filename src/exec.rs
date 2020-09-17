//! Command execution interface
use crate::aws::Credentials;
use anyhow::anyhow;
use std::{error::Error, str::from_utf8};
use tokio::process::Command;

pub async fn exec(
    credentials: Credentials,
    account_id: &str,
    command: &str,
) -> Result<String, Box<dyn Error>> {
    let Credentials {
        access_key_id,
        secret_access_key,
        session_token,
    } = credentials;
    let split = shlex::split(&command).unwrap();
    if split.iter().next().is_none() {
        return Err(anyhow!("Empty command".to_string()).into());
    }
    let output = split
        .iter()
        .skip(1)
        .fold(
            Command::new(&split[0])
                .env("AWS_ACCESS_KEY_ID", access_key_id)
                .env("AWS_SECRET_ACCESS_KEY", secret_access_key)
                .env("AWS_SESSION_TOKEN", session_token)
                .env("AWS_ACCOUNT_ID", account_id),
            |cmd, arg| cmd.arg(arg),
        )
        .output()
        .await?;

    if output.status.success() {
        Ok(from_utf8(&output.stdout)?.to_string())
    } else {
        Err(anyhow!(from_utf8(&output.stderr)?.to_string()).into())
    }
}
