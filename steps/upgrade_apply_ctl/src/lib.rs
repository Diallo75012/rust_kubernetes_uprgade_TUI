use async_trait::async_trait;
use core::{cmd::stream_child, state::{AppState, StepColor}};
use anyhow::Result;
use tokio::{process::Command, sync::{mpsc::Sender, watch::Sender as WatchTx}};


#[async_trait]
pub trait Step {
  fn name(&self) -> &'static str;
  async fn run(&self, tx_log: Sender<String>, tx_state: WatchTx) -> Result<()>;
}

pub struct UpgradeApplyCtl;

#[async_trait]
impl core::step::Step for UpgradeApplyCtl {
    fn name(&self) -> &'static str { "Upgrade Apply CTL" }

    async fn run(&self, tx_log: Sender<String>, _tx_state: WatchTx) -> Result<()> {
        let mut child = Command::new("bash")
            .arg("-c").arg("echo upgrade apply ctl && sleep 1 && echo done")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        stream_child(Self::name(), child, tx_log).await
    }
}
