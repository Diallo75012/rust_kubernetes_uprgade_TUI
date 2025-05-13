use async_trait::async_trait;
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
      _pipeline_state: &mut PipelineState,
      ) -> Result<(), StepError> {

        let containerd_desired_version_clone_madison_pulled_full_version = desired_versions.target_containerd_version.clone();
        // kube components: should be fine as it comes from `apt-cache madison` command
        let kube_desired_version_clone_madison_pulled_full_version = desired_versions.madison_pulled_full_version.clone();
        let _ = print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "Upgrade Plan Version For Plan (Kube Components): ",
          &kube_desired_version_clone_madison_pulled_full_version
        );
        let _ = print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "Upgrade Plan Version For Plan (Containerd Components): ",
          &containerd_desired_version_clone_madison_pulled_full_version
        );
        // use `-n` for non-interactive and make sure that beforehands you habve set up `sudo visudo` to allow `no password` for the user and for only upgrade concerned binaries path
        // we also use `--yes` for the `gpg` command so that if another key exist it will overwrite it without any prompt asking to confirm
        // unhold version to be able to upgrade those and then hold back those versions
        //Install Compatible Version of Containerd (Optional but better have it updated even if it is Ok for few years...)
        // Verify Upgrade Plan and Ugrade (Only in the first Control Plane: other ones are going to pick it up)
        let containerd_version_upgrade = &format!("sudo apt install containerd.io={}", containerd_desired_version_clone_madison_pulled_full_version);
        let kube_versions_upgrade = &format!("sudo apt-get install -y kubeadm={v} kubelet={v} kubectl={v}", v = kube_desired_version_clone_madison_pulled_full_version);
        let command = &format!(r#"
          sudo apt-mark unhold kubeadm kubelet kubectl && \
          sudo apt-get update && \
          {} && \
          sudo apt-mark hold kubeadm kubelet kubectl && \
          {} && \
          sudo systemctl restart containerd && \          
          sudo systemctl restart kubelet containerd && \
          sudo kubeadm upgrade plan --yes && \
          sudo -n apt-get update -y"#,
          containerd_version_upgrade,
          kube_versions_upgrade
        );
        // Prepare the child process (standard Rust async Command)
        // type of `child` is `tokio::process::Child`
        let child = Command::new("bash")
          .arg("-c")
          .arg(command)
          .stdout(std::process::Stdio::piped())
          .stderr(std::process::Stdio::piped())
          .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

        // Stream output + handle timeout via helper
        let _ = stream_child(self.name(), child, output_tx.clone()).await
          .map_err(|e| StepError::Other(e.to_string()));
        Ok(())
    }
}
