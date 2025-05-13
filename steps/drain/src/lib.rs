use async_trait::async_trait;
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use core_ui::{
  cmd::stream_child,
  state::{
  DesiredVersions,
  PipelineState,
  },
};
use shared_traits::step_traits::{Step, StepError};


pub struct Drain;

#[async_trait]
impl Step for Drain {
    fn name(&self) -> &'static str {
        "Drain"
    }

    async fn run(
      &mut self,
      output_tx: &Sender<String>,
      _desired_versions: &mut DesiredVersions,
      pipeline_state: &mut PipelineState,
      ) -> Result<(), StepError> {
        // The shell command to run
        let shell_cmd = &format!(
          r#"export KUBECONFIG=$HOME/.kube/config; kubectl drain {} --ignore-daemonsets --delete-emptydir-data;"#,
          pipeline_state.log.clone().shared_state_iter("node_name")[0].clone()
        );

        if pipeline_state.log.clone().shared_state_iter("node_role")[0].clone() == "Controller" {
          let child = Command::new("bash")
             .arg("-c")
             .arg("echo 'This a single controller node, will skip Drain Step for it to stay reachable on upgrade.'")
             .stdout(std::process::Stdio::piped())
             .stderr(std::process::Stdio::piped())
             .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

           // Stream output + handle timeout via helper
           let send_stream = stream_child(self.name(), child, output_tx.clone()).await
             .map_err(|e| StepError::Other(e.to_string()));
           send_stream      	
        } else {
          let child = Command::new("bash")
             .arg("-c")
             .arg(shell_cmd)
             .stdout(std::process::Stdio::piped())
             .stderr(std::process::Stdio::piped())
             .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

           // Stream output + handle timeout via helper
           let send_stream = stream_child(self.name(), child, output_tx.clone()).await
             .map_err(|e| StepError::Other(e.to_string()));
           send_stream
        }
    }
}
