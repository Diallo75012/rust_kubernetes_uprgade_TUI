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


pub struct RestartServices;

#[async_trait]
impl Step for RestartServices {
    fn name(&self) -> &'static str {
        "Restart Services"
    }

    async fn run(
      &mut self,
      output_tx: &Sender<String>,
      _desired_versions: &mut DesiredVersions,
      pipeline_state: &mut PipelineState,
      ) -> Result<(), StepError> {

        // we capture the `node_type`
        let node_type = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
        let node_name = pipeline_state.log.clone().shared_state_iter("node_name")[0].clone();
 
        let command = r#"sudo systemctl daemon-reload &&  sudo systemctl restart kubelet"#;
        let ssh_command = format!(
          r#"ssh {} 'sudo systemctl daemon-reload &&  sudo systemctl restart kubelet'"#,
          node_name,
        );
        // Prepare the child process (standard Rust async Command)
        // type of `child` is `tokio::process::Child`
        if node_type == "Controller" {
          let child = Command::new("bash")
            .arg("-c")
            .arg(comand)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
          // Stream output + handle timeout via helper
          let _ = stream_child(self.name(), child, output_tx.clone()).await
            .map_err(|e| StepError::Other(e.to_string()));	
        } else {
        
          let child = Command::new("bash")
            .arg("-c")
            .arg(ssh_command)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

          // Stream output + handle timeout via helper
          stream_child(self.name(), child, output_tx.clone()).await
            .map_err(|e| StepError::Other(e.to_string()))?;
        }
      Ok(())
    }
}
