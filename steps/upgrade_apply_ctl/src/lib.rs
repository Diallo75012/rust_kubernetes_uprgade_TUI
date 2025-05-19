use async_trait::async_trait;
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use core_ui::{
  cmd::stream_child,
  state::{
  DesiredVersions,
  PipelineState,
  //ClusterNodeType,
  NodeUpdateTrackerState,
  },
};
use shared_traits::step_traits::{Step, StepError};
use shared_fn::{
  parse_version::parse_versions,
  debug_to_file::print_debug_log_file,
};


pub struct UpgradeApplyCtl;

#[async_trait]
impl Step for UpgradeApplyCtl {
    fn name(&self) -> &'static str {
        "Upgrade Apply CTL"
    }

    async fn run(
      &mut self,
      output_tx: &Sender<String>,
      desired_versions: &mut DesiredVersions,
      pipeline_state: &mut PipelineState,
      _node_state_tracker: &mut NodeUpdateTrackerState,
      ) -> Result<(), StepError> {

        // we capture the `node_type`
        let node_type = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
        let kube_actual_version = pipeline_state.log.clone().shared_state_iter("kubeadm_v")[0].clone();
        let target_kube_version = desired_versions.madison_parsed_upgrade_apply_version.clone();
        let _ = print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "Upgrade Apply (Target Kube Version): ",
          &target_kube_version
        );

        // command echo for worker nodes to skip step
        let command_worker_skip = r#"echo Skip because worker node nothing to apply"#;

        // Prepare the child process (standard Rust async Command)
        // type of `child` is `tokio::process::Child`
        // here check that the `node_type` is controller otherwise just `echo 'Junko'
        if &node_type == "Controller" {
        // here we need to check which is the node type as it is only for `Controller` type
        // command: `sudo kubeadm upgrade apply v1.29.15 --yes` and here `y or --yes` does exist as there is interactivity

          // here we check and try a downgrade if the version desired `target_kube_version` is lower than the actual kube version
          if parse_versions(&target_kube_version).1 < parse_versions(&kube_actual_version).1 {
            // we add the `until kubectl get nodes; do sleep 5; done` so that node is ready as we get errors for node not ready
            let command = format!(r#"export KUBECONFIG=$HOME/.kube/config; until kubectl get nodes &> /dev/null; do sleep 5; done; sudo kubeadm upgrade apply v{} --yes --kubeconfig=/home/creditizens/.kube/config"#, target_kube_version);
            let _ = print_debug_log_file(
              "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
              "FULL Upgrade Apply CMD (normal)",
              &command
            );

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

            let command = format!(r#"export KUBECONFIG=$HOME/.kube/config; until kubectl get nodes &> /dev/null; do sleep 5; done; sudo kubeadm upgrade apply v{} --yes --kubeconfig=/home/creditizens/.kube/config"#, target_kube_version);
            let _ = print_debug_log_file(
              "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
              "FULL Upgrade Apply CMD",
              &command
            );

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
          }
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
