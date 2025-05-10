use crate::state::NodeUpdateTrackerState;

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
