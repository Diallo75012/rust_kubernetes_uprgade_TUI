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
    // The shell command to run
    let shell_cmd = r#"export KUBECONFIG=$HOME/.kube/config; kubectl get nodes --no-headers | awk '{print $1}' && kubeadm version | awk '{split($0,a,"\""); print a[6]}' | awk -F "[v]" '{ print "kubeadm "$1 $NF}' && containerd --version | awk '{ print "containerd "$3 }'"#;
    /*
    let shell_cmd = r#"
      which kubectl && kubectl version --client &&
      export KUBECONFIG=$HOME/.kube/config;
      nodes=""; 
      for elem in $(kubectl get nodes --no-headers | awk '{print $1}'); 
      do nodes="$nodes $elem"; 
      done; 
      echo $nodes | xargs
    "#;
     */
    // Prepare the child process (standard Rust async Command)
    // type of `child` is `tokio::process::Child`
    // we run discovery node only once at the beginning then the line parser function will turn this field `discovery_already_done` to `true`
    // so we will skip for next nodes, this makes life easy no need to filter everywhere the state
    if node_state_tracker.discovery_already_done {
      let child = Command::new("bash")
          .arg("-c")
          .arg("echo 'Discovery Node already done before, we skip this step here as we have track of which nodes needs to be done.'")
          .stdout(std::process::Stdio::piped())
          .stderr(std::process::Stdio::piped())
          .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

      // Stream output + handle timeout via helper
      stream_child(self.name(), child, output_tx.clone()).await
        .map_err(|e| StepError::Other(e.to_string()))?;
    } else {
      let child = Command::new("bash")
          .arg("-c")
          .arg(shell_cmd)
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
