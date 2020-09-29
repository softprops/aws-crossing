//! AWS interface
use anyhow::anyhow;
use async_trait::async_trait;
use serde::Deserialize;
use std::{error::Error, str::from_utf8};
use tokio::process::Command;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    status: String,
}

impl Account {
    fn is_active(&self) -> bool {
        self.status == "ACTIVE"
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
}

#[async_trait]
pub trait Aws {
    /// Lists aws sub accounts
    async fn accounts(&self) -> Result<Vec<Account>, Box<dyn Error + Send + Sync + 'static>>;

    /// Assumes an IAM role for within a given account
    async fn assume_role(
        &self,
        account_id: String,
        role: String,
    ) -> Result<Credentials, Box<dyn Error + Send + Sync + 'static>>;
}

#[derive(Clone)]
pub struct Cmd;

#[async_trait]
impl Aws for Cmd {
    async fn accounts(&self) -> Result<Vec<Account>, Box<dyn Error + Send + Sync + 'static>> {
        let output = Command::new("aws")
            .args(&[
                "organizations",
                "list-accounts",
                "--query",
                "Accounts",
                "--output",
                "json",
            ])
            .output()
            .await?;
        if !output.status.success() {
            return Err(anyhow!(from_utf8(&output.stderr)?.to_string()).into());
        }
        Ok(serde_json::from_slice::<Vec<Account>>(&output.stdout)?
            .into_iter()
            .filter(Account::is_active)
            .collect())
    }

    async fn assume_role(
        &self,
        account_id: String,
        role: String,
    ) -> Result<Credentials, Box<dyn Error + Send + Sync + 'static>> {
        let output = Command::new("aws")
            .args(&[
                "sts",
                "assume-role",
                "--role-arn",
                &format!("arn:aws:iam::{}:role/{}", account_id, role),
                "--role-session-name",
                &format!("{}-session-access", account_id),
                "--query",
                "Credentials",
                "--output",
                "json",
            ])
            .output()
            .await
            .unwrap();
        if output.status.success() {
            Ok(serde_json::from_slice::<Credentials>(&output.stdout)?)
        } else {
            Err(anyhow!(from_utf8(&output.stderr)?.to_string()).into())
        }
    }
}
