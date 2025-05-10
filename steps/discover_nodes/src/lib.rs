use async_trait::async_trait;
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use core_ui::cmd::stream_child;
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
      ) -> Result<(), StepError> {
        // The shell command to run
        let shell_cmd = r#"export KUBECONFIG=$HOME/.kube/config; kubectl get nodes --no-headers | awk '{print $1}' && kubeadm version | awk '{split($0,a,"\""); print a[6]}' | awk -F "[v]" '{ print "kubeadm"$1 $NF}' && containerd --version | awk '{ print "containerd"$3 }'         export KUBECONFIG=$HOME/.kube/config; kubectl get nodes --no-headers | awk '{print $1}' && kubeadm version | awk '{split($0,a,"\""); print a[6]}' | awk -F "[v]" '{ print "kubeadm"$1 $NF}' && containerd --version | awk '{ print "containerd"$3 }'"#;
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
        let child = Command::new("bash")
            .arg("-c")
            .arg(shell_cmd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?; // This returns std::io::Error, which StepError handles via `#[from]`

        // Stream output + handle timeout via helper
        stream_child(self.name(), child, output_tx.clone()).await
            .map_err(|e| StepError::Other(e.to_string()))
    }
}
