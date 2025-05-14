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


pub struct MadisonVersion;

#[async_trait]
impl Step for MadisonVersion {
    fn name(&self) -> &'static str {
        "Madison Version"
    }

    async fn run(
      &mut self,
      output_tx: &Sender<String>,
      desired_versions: &mut DesiredVersions,
      pipeline_state: &mut PipelineState,
      ) -> Result<(), StepError> {

        // we capture the `node_type`
        let node_type = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
        
        // The shell command to run
        /*
        sudo apt-cache madison kubectl
        */
        // normally here the state should be persistent for the full app lifetime so we can pull the user desired version for `kube` components
        let desired_version_clone = desired_versions.target_kube_versions.clone();
        let user_desired_kube_components_version = desired_version_clone.split(".").collect::<Vec<&str>>();
        let major_version = user_desired_kube_components_version[0];
        let minor_version = user_desired_kube_components_version[1];
        // if user play to much and give use a version like `1.29.34.45.etc...`, we want just the first (major version) and second (minor version)
        let user_desired_kube_verison_parsed = format!("{}.{}", major_version, minor_version);
        // we grep the line with version number corresponding and then get the first row `NR==1`, `$0` for the full row (we will in `engine/src/lib.rs` parse the version from that line) 
        let command_formatted = format!(
          r#"sudo -n apt-cache madison kubectl | grep '{}' | awk 'NR==1{{print $0}}'"#,
          user_desired_kube_verison_parsed
        );

        // Prepare the child process (standard Rust async Command)
        // type of `child` is `tokio::process::Child`
        if &node_type == "Controller" {
          let child = Command::new("bash")
              .arg("-c")
              .arg(command_formatted)
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
              .arg("We are working on Worker Node Upgrade so no need to parse versions available, already done by Controller Node.")
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
