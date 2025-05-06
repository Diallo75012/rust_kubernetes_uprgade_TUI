use async_trait::async_trait;
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use core_ui::cmd::stream_child;
use shared_traits::step_traits::{Step, StepError};

pub struct PullRepoKey;

#[async_trait]
impl Step for PullRepoKey {
    fn name(&self) -> &'static str {
        "Pull Repo Key"
    }

    async fn run(&mut self, output_tx: &Sender<String>) -> Result<(), StepError> {
        // The shell command to run
        let shell_cmd = "echo pulling repo key && sleep 1 && echo done";

        // Prepare the child process (standard Rust async Command)
        let child = Command::new("bash")
            .arg("-c")
            .arg(shell_cmd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

        // Stream output + handle timeout via helper
        stream_child(self.name(), child, output_tx.clone()).await
            .map_err(|e| StepError::Other(e.to_string()))
    }
}
