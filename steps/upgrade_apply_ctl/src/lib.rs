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


pub struct UpgradePlan;

#[async_trait]
impl Step for UpgradePlan {
    fn name(&self) -> &'static str {
        "Upgrade Plan"
    }

    async fn run(
      &mut self,
      output_tx: &Sender<String>,
      desired_versions: &mut DesiredVersions,
      pipeline_state: &mut PipelineState,
      ) -> Result<(), StepError> {

        // we capture the `node_type`
        let node_type = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
        let target_kube_version = desired_versions.target_kube_versions.clone();
        let _ = print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "Upgrade Apply (Target Kube Version): ",
          &target_kube_version
        );

        // here we need to check which is the node type as it is only for `Controller` type
        // command: `sudo kubeadm upgrade apply v1.29.15 --yes` and here `y or --yes` does exist as there is interactivity
        let command = format!(r#"sudo kubeadm upgrade apply v{} --yes"#, target_kube_version);
        let _ = print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "FULL Upgrade Apply CMD",
          &command
        );

        // command echo for worker nodes to skip step
        let command_worker_skip = r#"echo 'This is a Worker Node so we don't apply the upgrade, the step Upgrade Node, will do the job for Worker Node Types.''"#;

        // Prepare the child process (standard Rust async Command)
        // type of `child` is `tokio::process::Child`
        // here check that the `node_type` is controller otherwise just `echo 'Junko'
        if &node_type == "Controller" {
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
            .arg(command_worker_skip)
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
