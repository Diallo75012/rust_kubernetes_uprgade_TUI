use core_ui::state::{
  PipelineState,
  ClusterNodeType,
  NodeUpdateTrackerState,
};
use shared_fn::write_debug_steps::write_step_cmd_debug;


pub fn state_updater_for_ui_good_display(
  step &'static str,
  line: &str,
  // the function signature will borrow by reference and not value so that when we make changed here it will be reflected in the state no need to return it
  shared_state: &mut PipelineState,
  node_update_tracker_state: &mut NodeUpdateTrackerState
  ) {
  if "Discover Nodes" = step {
    // in this step we want to update the initialized `PipelineState` fields `log.node_name` and `log.node_type`
    // using implemented functions `fn update_shared_state_node_type(&mut self, node_role: ClusterNodeType)`
    // and `fn update_shared_state_info(&mut self, k: &str, v: &str)`

    // we delete all previous entry by replacing the previous `vec` by a mew one 
    let mut node_update_tracker_state.discovered_node = Vec::new();
    // push those lines in the vector
    node_update_tracker_state.discovered_node.push(line.to_string());
    // check to get rid from the vector what has already been updated
    for elem in node_update_tracker_state.node_already_updated.iter() {
      if node_update_tracker_state.discovered_node.contains(elem) {
        // we keep only what hasn't been updated yet in `node_update_tracker_state.discover_node`
     	node_update_tracker_state.discovered_node.iter().retain(|x| x != elem);
      }
    }
    // now we update the field `node_name` in `shared_state` `PipelineState` taking the first index from `node_update_tracker_state`
    let _ = shared_state.update_shared_state_info("node_name", &node_update_tracker_state.discovered_node[0]);
     
    // then we update the `node_type` field of `shared_state` PipelineState`
    if node_update_tracker_state.discovered_node[0].contains("controller") {
      // update the `node_type` and the `status`
      let _ = shared_state.update_shared_state_node_type(ClusterNodeType::Controller);
      let _ = shared_statee.update_shared_state_status(UpgradeStatus::InProcess);
    } else {
      // update the `node_type` and the `status`
      let _ = shared_state.update_shared_state_node_type(ClusterNodeType::Worker);
      let _ = shared_state.update_shared_state_status(UpgradeStatus::InProcess);	
    }
  }
  } else if "Pull Repo Key" = step {
 
  } else if "Madison Version" = step  {
  	
  } else if "Upgrade Plan" = step {
  	
  } else if "Upgrade Apply" = step {
  	
  } else if "Upgrade Node" = step {
  	
  } else if "Veryfy Core DNS Proxy = step {
  	
  }	
}
