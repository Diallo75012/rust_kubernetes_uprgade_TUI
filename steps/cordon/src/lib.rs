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


pub struct Cordon;

#[async_trait]
impl Step for Cordon {
    fn name(&self) -> &'static str {
        "Cordon"
    }

    async fn run(
      &mut self,
      output_tx: &Sender<String>,
      _desired_versions: &mut DesiredVersions,
      pipeline_state: &mut PipelineState,
      ) -> Result<(), StepError> {
        let commands = [
         &format!(r#"export KUBECONFIG=$HOME/.kube/config; kubectl cordon {};"#, pipeline_state.log.clone().shared_state_iter("node_name")[0].clone()),
        ];
        // Prepare the child process (standard Rust async Command)
        // type of `child` is `tokio::process::Child`
        let _multi_command = for command in 0..commands.len() {
          if pipeline_state.log.clone().shared_state_iter("node_role")[0].clone() == "Controller" {
            let cmd = "echo 'This a single controller node, will skip Cordon Step for it to stay reachable on upgrade.'";
            let child = Command::new("bash")
              .arg("-c")
              .arg(cmd)
              .stdout(std::process::Stdio::piped())
              .stderr(std::process::Stdio::piped())
              .spawn()?;
            // Stream output + handle timeout via helper
            let _ = stream_child(self.name(), child, output_tx.clone()).await
              .map_err(|e| StepError::Other(e.to_string()));	
          } else {
          
            let cmd = commands[command];
            let child = Command::new("bash")
              .arg("-c")
              .arg(cmd)
              .stdout(std::process::Stdio::piped())
              .stderr(std::process::Stdio::piped())
              .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

            // Stream output + handle timeout via helper
            stream_child(self.name(), child, output_tx.clone()).await
              .map_err(|e| StepError::Other(e.to_string()))?;
          }
        };
        Ok(())
    }
}
