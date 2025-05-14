use async_trait::async_trait;
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use core_ui::{
  cmd::stream_child,
  state::{
  DesiredVersions,
  PipelineState,
  ClusterNodeType,
  },
};
use shared_traits::step_traits::{Step, StepError};
use shared_fn::debug_to_file::print_debug_log_file;


pub struct UpgradeNode;

#[async_trait]
impl Step for UpgradeNode {
    fn name(&self) -> &'static str {
        "Upgrade Node"
    }

    async fn run(
      &mut self,
      output_tx: &Sender<String>,
      desired_versions: &mut DesiredVersions,
      pipeline_state: &mut PipelineState,
      ) -> Result<(), StepError> {

        // we capture the `node_type`
        let node_type = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
        let node_name = pipeline_state.log.clone().shared_state_iter("node_name")[0].clone();
        let target_kube_version = desired_versions.target_kube_versions.clone();
        let _ = print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "Upgrade Node (Target Kube Version): ",
          &target_kube_version
        );

        // example of `ssh` commands from controller to upgrade the `worker` node
        // ssh creditizens@node1 'bash -c command'
        // ssh creditizens@node1 'bash -c commad && other_command'
        // ssh creditizens@node1 'command; other_command; some_more_commands'
        let command = format!(r#"ssh {} 'sudo kubeadm upgrade node'"#, node_name);
        let _ = print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "FULL Upgrade Node CMD",
          &command
        );

        // controller echo message
        let command_controller_skip = format!(
          r#"This is a Controller Node it will be applied upgrade on it and other Worker nodes would pull the new Cluster Version, here: v{}."#,
          target_kube_version,
        );

        // Prepare the child process (standard Rust async Command)
        // type of `child` is `tokio::process::Child`
        // here check that the `node_type` is controller otherwise just `echo 'Junko'
        if &node_type == "Worker" {
          let child = Command::new("bash")
            .arg("-c")
            .arg(command)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

          // Stream output + handle timeout via helper
          stream_child(self.name(), child, output_tx.clone()).await
            .map_err(|e| StepError::Other(e.to_string()))?;
          Ok(())
        } else {
          let child = Command::new("bash")
            .arg("-c")
            .arg(command_controller_skip)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

          // Stream output + handle timeout via helper
          stream_child(self.name(), child, output_tx.clone()).await
            .map_err(|e| StepError::Other(e.to_string()))?;
          Ok(())
        }
    }
}
