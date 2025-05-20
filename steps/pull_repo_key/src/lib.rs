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
use shared_fn::debug_to_file::print_debug_log_file;


pub struct PullRepoKey;

#[async_trait]
impl Step for PullRepoKey {
    fn name(&self) -> &'static str {
        "Pull Repo Key"
    }

    async fn run(
      &mut self,
      output_tx: &Sender<String>,
      desired_versions: &mut DesiredVersions,
      pipeline_state: &mut PipelineState,
      _node_state_tracker: &mut NodeUpdateTrackerState,
      ) -> Result<(), StepError> {

        // we get the node name
        let node_type = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
        let node_name = pipeline_state.log.clone().shared_state_iter("node_name")[0].clone();
        
        // normally here the state should be persistent for the full app lifetime so we can pull the user desired version for `kube` components
        let desired_version_clone = desired_versions.target_kube_versions.clone();
        let user_desired_kube_components_version = desired_version_clone.split(".").collect::<Vec<&str>>();
        let major_version = user_desired_kube_components_version[0];
        let minor_version = user_desired_kube_components_version[1];
        // if user play to much and give use a version like `1.29.34.45.etc...`, we want just the first (major version) and second (minor version)
        let user_desired_kube_version_parsed = format!("{}.{}", major_version, minor_version);
        let _ = print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "Pull Repo Key (User desired kube version parsed): ",
          &user_desired_kube_version_parsed
        );

        // use `-n` for non-interactive and make sure that beforehands you habve set up `sudo visudo` to allow `no password` for the user and for only upgrade concerned binaries path
        // we also use `--yes` for the `gpg` command so that if another key exist it will overwrite it without any prompt asking to confirm 
        let commands = [
          "sudo -n apt-get update -y",
          "sudo -n apt-get install -y curl apt-transport-https",
          "echo 'curl install and apt-transport-https checked'",
          &format!(
            r#"curl -fsSL https://pkgs.k8s.io/core:/stable:/v{}/deb/Release.key | sudo gpg --yes --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg"#,
            user_desired_kube_version_parsed,
          ),
          "echo 'successfully pulled the keyrings'",
          &format!(
            r#"echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v{}/deb/ /' | sudo tee /etc/apt/sources.list.d/kubernetes.list"#,
            user_desired_kube_version_parsed,
          ),
          &format!(
            r#"echo 'successfully update the kubernetes repository to version {}. Just wait for last update before next step starts..... waitoooooo.....'"#,
            user_desired_kube_version_parsed,
          ),
          "sudo -n apt-get update -y",
        ];

        // for ssh we need to repeat ssh <node name> even int he command after the pipe
        let ssh_node_name = format!("ssh {}", &node_name);
        let commands_ssh = [
          &format!(
            "{} sudo -n apt-get update -y",
            &ssh_node_name,
          ),
          &format!(
            "{} sudo -n apt-get install -y curl apt-transport-https",
            &ssh_node_name
          ),
          "echo 'curl install and apt-transport-https checked'",
          &format!(
            r#"{ssh_node} curl -fsSL https://pkgs.k8s.io/core:/stable:/v{}/deb/Release.key | {ssh_node} sudo gpg --yes --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg"#,
            user_desired_kube_version_parsed,
            ssh_node = &ssh_node_name,
          ),
          "echo 'successfully pulled the keyrings'",
          &format!(
            r#"{ssh_node} echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v{}/deb/ /' | {ssh_node} sudo tee /etc/apt/sources.list.d/kubernetes.list"#,
            user_desired_kube_version_parsed,
            ssh_node = &ssh_node_name,
          ),
          &format!(
            r#"echo 'successfully update the kubernetes repository to version {}. Just wait for last update before next step starts..... waitoooooo.....'"#,
            user_desired_kube_version_parsed,
          ),
          &format!(
            "{} sudo -n apt-get update -y",
            &ssh_node_name,
          ),
        ];

        // Prepare the child process (standard Rust async Command)
        // type of `child` is `tokio::process::Child`
        if &node_type == "Controller" {
          let _multi_command = for idx in 0..commands.len() {
            let cmd = commands[idx];
            let child = Command::new("bash")
              .arg("-c")
              .arg(cmd)
              .stdout(std::process::Stdio::piped())
              .stderr(std::process::Stdio::piped())
              .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

            // Stream output + handle timeout via helper
            stream_child(self.name(), child, output_tx.clone()).await
              .map_err(|e| StepError::Other(e.to_string()))?;
          };
          Ok(())
        } else {
          let _multi_command = for idx in 0..commands_ssh.len() {
            let ssh_cmd = commands_ssh[idx];
            // let ssh_cmd = format!(r#"ssh {} {}"#, &node_name, cmd);
            let _ = print_debug_log_file(
              "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
              "Pull Repo Key (SSH CMD): ",
              &ssh_cmd
            );
            let child = Command::new("bash")
              .arg("-c")
              .arg(ssh_cmd)
              .stdout(std::process::Stdio::piped())
              .stderr(std::process::Stdio::piped())
              .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

            // Stream output + handle timeout via helper
            stream_child(self.name(), child, output_tx.clone()).await
              .map_err(|e| StepError::Other(e.to_string()))?;
          };
          Ok(())        	
        }
    }
}
