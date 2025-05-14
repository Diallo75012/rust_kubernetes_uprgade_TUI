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


pub struct VerifyCoreDnsProxy;

#[async_trait]
impl Step for VerifyCoreDnsProxy {
    fn name(&self) -> &'static str {
        "Verify Core DNS Proxy"
    }

    async fn run(
      &mut self,
      output_tx: &Sender<String>,
      _desired_versions: &mut DesiredVersions,
      _pipeline_state: &mut PipelineState,
      ) -> Result<(), StepError> {
 
        let command = r#"export KUBECONFIG=$HOME/.kube/config;  kubectl get daemonset kube-proxy -n kube-system -o=jsonpath='{.spec.template.spec.containers[0].image}' | awk '{split($0,a,"v"); print a[2]}' | awk -F "[v]" '{ print "kubeproxy "$2 $NF}'"#;
        // Prepare the child process (standard Rust async Command)
        // type of `child` is `tokio::process::Child`
        let child = Command::new("bash")
          .arg("-c")
          .arg(command)
          .stdout(std::process::Stdio::piped())
          .stderr(std::process::Stdio::piped())
          .spawn()?;
        // Stream output + handle timeout via helper
        let _ = stream_child(self.name(), child, output_tx.clone()).await
          .map_err(|e| StepError::Other(e.to_string()))?;
        Ok(())
    }
}
