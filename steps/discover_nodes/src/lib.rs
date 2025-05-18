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


pub struct DiscoverNodes;

#[async_trait]
impl Step for DiscoverNodes {
  fn name(&self) -> &'static str {
    "Discover Nodes"
  }

  async fn run(
    &mut self,
    output_tx: &Sender<String>,
    _desired_versions: &mut DesiredVersions,
    _pipeline_state: &mut PipelineState,
    node_state_tracker: &mut NodeUpdateTrackerState,
  ) -> Result<(), StepError> {

    // Prepare the child process (standard Rust async Command)
    // type of `child` is `tokio::process::Child`
    // we run discovery node only once at the beginning then the line parser function will turn this field `discovery_already_done` to `true`
    // we won't skip this step but just send the content of next node to work on taking it from the state `NodeUpdateTrackerState`
    if node_state_tracker.discovery_already_done {
      // we get here the next node to work on from the list of node `TO DO` and will send it in the stream which is gonna be capture by the line parser
      let next_node_to_do = node_state_tracker.discovered_node[0].clone().to_string();
      // getting error with curly braces as Rust interprets `$NF` as command, so will inject those as raw input in the String formatting
      let subsequent_discovery_cmd = [
        r#"kubeadm version | awk '{split($0,a,"\""); print a[6]}' | awk -F "[v]" '{ print "kubeadm "$1 $NF }'"#,
        r#"containerd --version | awk '{ print "containerd "$3 }'"#,
        &format!(r#"echo {}"#, &next_node_to_do),
      ];
      for idx in 0..subsequent_discovery_cmd.len() {
        // we can concatenate `String` to `&str` using `+` sign
        // add little space for the command to come after with a little space from ssh..
        let cmd = format!("ssh {} ", &next_node_to_do) + subsequent_discovery_cmd[idx];
        
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
    } else {
      // The shell command to run
      let first_discovery_cmd = r#"export KUBECONFIG=$HOME/.kube/config; kubectl get nodes --no-headers | awk '{print $1}' && kubeadm version | awk '{split($0,a,"\""); print a[6]}' | awk -F "[v]" '{ print "kubeadm "$1 $NF}' && containerd --version | awk '{ print "containerd "$3 }'"#;
      let child = Command::new("bash")
          .arg("-c")
          .arg(first_discovery_cmd)
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
