use crate::state::{
  NodeUpdateTrackerState,
  DesiredVersions,
};

/*** Discovery Node Parsers  ***/

// this one will parse the line to get only the name of the node
pub fn line_parser(field: &str) -> String {
  let name = field.to_string();
  let list_part_name_to_parse = name.split(" ").collect::<Vec<&str>>();
  let mut parsed_name = String::new();
  for i in 0..list_part_name_to_parse.len() {
    if i == list_part_name_to_parse.len()-1 {
      parsed_name += list_part_name_to_parse[i];
    }
  }
  parsed_name
}

// this one would check if the node haven't been already done, if yes, pop it from the list of ndoes to update
pub fn discover_nodes_state_filter(state_node_tracking: &mut NodeUpdateTrackerState) -> anyhow::Result<()> {
  for elem in state_node_tracking.node_already_updated.iter() {
    if state_node_tracking.discovered_node.contains(elem) {
      // we keep only what hasn't been updated yet in `node_update_tracker_state.discover_node`
      state_node_tracking.discovered_node.retain(|x| x != elem);
    }
  }
  Ok(())
}

// this  will parse the version that we got from our `apt-cache madison` command to get the full version for the next steps upgrades of `kubeadm`
pub fn madison_get_full_version_for_kubeadm_upgrade_saved_to_state(line: &str, desired_version_state: &mut DesiredVersions) {
  let desired_version_clone = desired_version_state.target_kube_versions.clone();
  let user_desired_kube_components_version = desired_version_clone.split(".").collect::<Vec<&str>>();
  let major_version = user_desired_kube_components_version[0];
  let minor_version = user_desired_kube_components_version[1];
  // if user play to much and give use a version like `1.29.34.45.etc...`, we want just the first (major version) and second (minor version)
  let formatted_version = format!("{}.{}", major_version, minor_version);
  if line.contains(&formatted_version) {
    let splitted_line = line.split(" ").collect::<Vec<&str>>();
    let parsed_line = splitted_line[2];
    desired_version_state.add("madison_pulled_full_version", parsed_line)	
  }
} 
