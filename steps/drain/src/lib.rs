use async_trait::async_trait;
use core_ui::{cmd::stream_child, state::{AppState, StepColor}};
use anyhow::Result;
use tokio::{process::Command, sync::{mpsc::Sender, watch::Sender as WatchTx}};
use shared_traits::step_traits::Step;


pub struct Drain;

#[async_trait]
impl Step for Drain {
    fn name(&self) -> &'static str { "Drain" }

    async fn run(&self, tx_log: Sender<String>, _tx_state: WatchTx<AppState>) -> Result<()> {
        let mut child = Command::new("bash")
            .arg("-c").arg("echo drain nodes && sleep 1 && echo done")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        stream_child(self.name(), child, tx_log).await
    }
}
