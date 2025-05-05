use async_trait::async_trait;
use core_ui::{state::AppState};
use anyhow::Result;
use tokio::sync::{mpsc::Sender, watch::Sender as WatchTx};


#[async_trait]
pub trait Step {
  fn name(&self) -> &'static str;
  async fn run(&self, tx_log: Sender<String>, tx_state: WatchTx<AppState>) -> Result<()>;
}
