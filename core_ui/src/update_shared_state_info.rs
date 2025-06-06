use crate::state::{
  PipelineState,
  ClusterNodeType,
  UpgradeStatus,
  NodeUpdateTrackerState,
  ComponentsVersions,
};
use shared_fn::debug_to_file::print_debug_log_file;


pub fn state_updater_for_ui_good_display(
  step: &'static str,
  line: &str,
  // the function signature will borrow by reference and not value so that when we make changed here it will be reflected in the state no need to return it
  shared_state: &mut PipelineState,
  node_update_tracker_state: &mut NodeUpdateTrackerState,
  components_versions: &mut ComponentsVersions,
  ) {
  if "Discover Nodes" == step {
    // in this step we want to update the initialized `PipelineState` fields `log.node_name` and `log.node_type`
    // using implemented functions `fn update_shared_state_node_type(&mut self, node_role: ClusterNodeType)`
    // and `fn update_shared_state_info(&mut self, k: &str, v: &str)`

    // push those lines in the vector
    match line {
      l if l.contains("kubeadm") => { /* we will update the state by adding the `kube_versions`*/
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "Line:    ",
            l
          );
          let parse_kubeadm_versions = l.split(" ").collect::<Vec<&str>>(); // expect: ["kubead", "1.29..."]
          components_versions.add("kube_versions", parse_kubeadm_versions[3]); // expect "1.29..."
          shared_state.update_shared_state_info("kubeadm_v", parse_kubeadm_versions[3]); // update shared state kube versions
          shared_state.update_shared_state_info("kubelet_v", parse_kubeadm_versions[3]);
          shared_state.update_shared_state_info("kubectl_v", parse_kubeadm_versions[3]);
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "Shared State Log keys/values:\n",
            &format!("{}", shared_state.log)
          );
        },
      l if l.contains("containerd") => { /* we will update the state by adding the `containerd_version`*/
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "Line: ",
            l
          );
          let parse_containerd_version = l.split(" ").collect::<Vec<&str>>(); // expect: ["container", "1.7..."]
          components_versions.add("containerd_version", parse_containerd_version[3]); // expect "1.7..."
          shared_state.update_shared_state_info("containerd_v", parse_containerd_version[3]); // update shared state containerd version
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "Line index[3]: ",
            parse_containerd_version[3]
          );
        },
      l => {
        // this is to parse
        node_update_tracker_state.discovered_node.push(l.to_string());
        // check to get rid from the vector what has already been updated
        for elem in node_update_tracker_state.node_already_updated.iter() {
          if node_update_tracker_state.discovered_node.contains(elem) {
            // we keep only what hasn't been updated yet in `node_update_tracker_state.discover_node`
            node_update_tracker_state.discovered_node.retain(|x| x != elem);
          }
        }
        // now we update the field `node_name` in `shared_state` `PipelineState` taking the first index from `node_update_tracker_state`
        let name = &node_update_tracker_state.discovered_node[0].to_string();
        let list_part_name_to_parse = name.split(" ").collect::<Vec<&str>>();
        let mut parsed_name = String::new();
        for i in 0..list_part_name_to_parse.len() {
          if i == list_part_name_to_parse.len()-1 {
            parsed_name += list_part_name_to_parse[i]
          }
        }
        let debug_var = node_update_tracker_state.discovered_node.iter().map(|x| x.to_string()).collect::<String>();
        shared_state.update_shared_state_info("node_name", &parsed_name);
    
        // debug to see node name
        let _ =  print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "discover node [0]",
          &parsed_name
        );
        let _ =  print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "check what inside vec discover nodde:",
          &debug_var
        );    
      
        // then we update the `node_type` field of `shared_state` PipelineState`
        if node_update_tracker_state.discovered_node[0].contains("controller") {
          // update the `node_type` and the `status`
          shared_state.update_shared_state_node_type(ClusterNodeType::Controller);
          shared_state.update_shared_state_status(UpgradeStatus::InProcess);
        } else {
          // update the `node_type` and the `status`
          shared_state.update_shared_state_node_type(ClusterNodeType::Worker);
          shared_state.update_shared_state_status(UpgradeStatus::InProcess);	
        }
      },
    }
  /* Have completely forgotten that I had already planned that wor each step so that i can call it in the end of `engine/src/lib.rs
     So there for each step have already created functions pulled from `core_ui/src/parsed_lines.rs`
     So can more all here and only call one function there `engine/src/lib/rs`
    ************* TO DO **************
  `*/
  } else if "Pull Repo Key" == step {
 
  } else if "Madison Version" == step  {
  	
  } else if "Upgrade Plan" == step {
  	
  } else if "Upgrade Apply" == step {
  	
  } else if "Upgrade Node" == step {
  	
  } else if "Veryfy Core DNS Proxy" == step {
  	
  }	
}
