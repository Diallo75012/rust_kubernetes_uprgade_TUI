/*use async_trait::async_trait;
use core_ui::{state::AppState};
use anyhow::Result;
use tokio::sync::{mpsc::Sender, watch::Sender as WatchTx};


#[async_trait]
pub trait Step {
  fn name(&self) -> &'static str;
  async fn run(&self, tx_log: Sender<String>, tx_state: WatchTx<AppState>) -> Result<()>;
}
*/

// In some core module, e.g., steps/core.rs or engine/step_trait.rs
use async_trait::async_trait;
use std::io;
use thiserror::Error;


#[async_trait]
pub trait Step: Send + Sync {
    /// Human-readable name of the step (for logging/UI).
    fn name(&self) -> &'static str;
    /// Execute the step. Takes a transmitter for log output and returns Ok on success or an error.
    async fn run(&mut self, output_tx: &tokio::sync::mpsc::Sender<String>) -> Result<(), StepError>;
}

/// Define a StepError to encapsulate various failure modes.
#[derive(Debug, Error)]
pub enum StepError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),  // Handles .spawn() and other IO ops

    #[error("Timeout waiting for step")]
    Timeout,

    #[error("Step failed: {0}")]
    Other(String),  // For any custom errors if needed
}
