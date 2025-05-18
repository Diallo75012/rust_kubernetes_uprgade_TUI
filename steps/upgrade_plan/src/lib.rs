use async_trait::async_trait;
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use core_ui::{
  cmd::stream_child,
  state::{
  DesiredVersions,
  PipelineState,
  NodeUpdateTrackerState,
  },
};
use shared_traits::step_traits::{Step, StepError};
use shared_fn::{
  parse_version::parse_versions,
  debug_to_file::print_debug_log_file,
};


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
      _node_state_tracker: &mut NodeUpdateTrackerState,
      ) -> Result<(), StepError> {
      
      // we capture the `node_type`
      let node_type = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
      let _ = print_debug_log_file(
        "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
        "UpgradePlan Node Type",
        &node_type
      );
      let kube_actual_version = pipeline_state.log.clone().shared_state_iter("kubeadm_v")[0].clone();
      let target_kube_version = desired_versions.madison_parsed_upgrade_apply_version.clone();
      let _ = print_debug_log_file(
        "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
        "Upgrade Apply (Target Kube Version): ",
        &target_kube_version
      );
      
      
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
      // using keyword to capt lines having the versions `kubeadm_plan, kubelet_plan, kubectl_plan, containerd_plan` with space so that i can split on it
      // and will compare those to the one saved in state in `core_ui/src/parse_lines.rs`... and use at the end of `engine/src/lib/rs`
      let export_kube_config = "export KUBECONFIG=$HOME/.kube/config";
      let unhold_versions = "sudo apt-mark unhold kubeadm kubelet kubectl";
      let containerd_version_upgrade = format!("sudo apt-get install containerd.io={}", containerd_desired_version_clone_madison_pulled_full_version);
      let kube_versions_upgrade = format!("sudo apt-get install -y kubeadm={v} kubelet={v} kubectl={v}", v = kube_desired_version_clone_madison_pulled_full_version);
      let kube_versions_upgrade_allow_downgrades = format!(
        "sudo apt-get install -y kubeadm={v} kubelet={v} kubectl={v} --allow-downgrades",
        v = kube_desired_version_clone_madison_pulled_full_version
      );
      let hold_versions_back = "sudo apt-mark hold kubeadm kubelet kubectl";
      // let apt_update = "sudo -n apt-get update -y";
      let restart_kubelet_and_containerd = "sudo systemctl restart kubelet containerd";
      // upgrade plan is non interactive and do not prompt anything
      let upgrade_plan = "sudo kubeadm upgrade plan";
      let kubeadm_plan = r#"kubeadm version | awk '{split($0,a,"\""); print a[6]}' | awk -F "[v]" '{ print "kubeadm_plan "$1 $NF }'"#;
      let kubelet_plan = r#"kubelet --version | awk '{ print $2 }' | awk -F "[v]" '{ print "kubelet_plan "$1 $NF }'"#;
      let kubectl_plan = r#"kubectl version | awk 'NR==1{ print $3 }' | awk -F "[v]" '{ print "kubectl_plan "$1 $NF }'"#;
      let containerd_plan = r#"containerd --version | awk '{ print "containerd_plan "$3 }'"#;

      let command = format!(r#"{} && {} && {} && {} && {} && {} && {} && {} && {} && {} && {}"#,
        export_kube_config,
        unhold_versions, 
        containerd_version_upgrade,
        kube_versions_upgrade,
        hold_versions_back,
        //apt_update,
        restart_kubelet_and_containerd,
        upgrade_plan,
        kubeadm_plan,
        kubelet_plan,
        kubectl_plan,
        containerd_plan,        
      );

      let command_downgrade = format!(r#"{} && {} && {} && {} && {} && {} && {} && {} && {} && {} && {}"#,
        export_kube_config,
        unhold_versions, 
        containerd_version_upgrade,
        kube_versions_upgrade_allow_downgrades,
        hold_versions_back,
        //apt_update,
        restart_kubelet_and_containerd,
        upgrade_plan,
        kubeadm_plan,
        kubelet_plan,
        kubectl_plan,
        containerd_plan,
      );

      let command_worker_upgrade_components = format!(r#"{} && {} && {} && {} && {} && {} && {} && {} && {}"#,
        unhold_versions, 
        containerd_version_upgrade,
        kube_versions_upgrade,
        hold_versions_back,
        //apt_update,
        restart_kubelet_and_containerd,
        kubeadm_plan,
        kubelet_plan,
        kubectl_plan,
        containerd_plan,
      );

      let command_worker_downgrade = format!(r#"{} && {} && {} && {} && {} && {} && {} && {} && {} && {}"#,
        export_kube_config,
        unhold_versions, 
        containerd_version_upgrade,
        kube_versions_upgrade_allow_downgrades,
        hold_versions_back,
        //apt_update,
        restart_kubelet_and_containerd,
        kubeadm_plan,
        kubelet_plan,
        kubectl_plan,
        containerd_plan,
      );

      // Prepare the child process (standard Rust async Command)
      // type of `child` is `tokio::process::Child`
      if &node_type == "Controller" {
        // here we check and try a downgrade if the version desired `target_kube_version` is lower than the actual kube version
        if parse_versions(&target_kube_version).1 < parse_versions(&kube_actual_version).1 {
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "FULL UpgradePlan CMD DOWNGRADE",
            &command_downgrade
          );
          let child = Command::new("bash")
            .arg("-c")
            .arg(command_downgrade)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

          // Stream output + handle timeout via helper
          stream_child(self.name(), child, output_tx.clone()).await
            .map_err(|e| StepError::Other(e.to_string()))?;
          Ok(())
        } else {
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "FULL UpgradePlan CMD",
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
        // ChatGPT told me this is clearner and it is true but i keep my initial code and put in comments chatGPT suggestion
        // Cleanly skip the step
        // output_tx.send("Skipping Upgrade Plan for Worker Node.".to_string()).await.ok(); return Ok(());
        if parse_versions(&target_kube_version).1 < parse_versions(&kube_actual_version).1 {
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "FULL UpgradePlan CMD WORKER DOWNGRADE",
            &command_worker_downgrade
          );
          let child = Command::new("bash")
            .arg("-c")
            .arg(command_worker_downgrade)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

          // Stream output + handle timeout via helper
          stream_child(self.name(), child, output_tx.clone()).await
            .map_err(|e| StepError::Other(e.to_string()))?;
          Ok(())
        } else {
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "FULL UpgradePlan CMD WORKER",
            &command_worker_upgrade_components
          );
          let child = Command::new("bash")
            .arg("-c")
            .arg(command_worker_upgrade_components)
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
}
