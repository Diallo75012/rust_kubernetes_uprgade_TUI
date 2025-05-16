use crate::state::{
  PipelineState,
  DesiredVersions,
  ClusterNodeType,
  UpgradeStatus,
  NodeUpdateTrackerState,
  ComponentsVersions,
};
use shared_fn::debug_to_file::print_debug_log_file;

/*** Discovery Node Parsers  ***/

// this one will parse the line to get only the name of the node
pub fn line_parser(field: &str) -> String {
  // improved version of this function
  let parsed_name = match field.split_whitespace().last() {
    Some(name) => name.to_string(),
    None => "".to_string(),
  };
  // let name = field.to_string();
  // let list_part_name_to_parse = name.split(" ").collect::<Vec<&str>>();
  // let mut parsed_name = String::new();
  // for i in 0..list_part_name_to_parse.len() {
  //   if i == list_part_name_to_parse.len()-1 {
  //     parsed_name += list_part_name_to_parse[i];
  //   }
  // }
  parsed_name
}

// this one would check if the node haven't been already done, if yes, pop it from the list of ndoes to update
// pub fn discover_nodes_state_filter(state_node_tracking: &mut NodeUpdateTrackerState) -> anyhow::Result<()> {
//   for elem in state_node_tracking.node_already_updated.iter() {
//     if state_node_tracking.discovered_node.contains(elem) {
//       // we keep only what hasn't been updated yet in `node_update_tracker_state.discover_node`
//       state_node_tracking.discovered_node.retain(|x| x != elem);
//     }
//   }
//   Ok(())
// }
pub fn discover_nodes_state_filter(state_node_tracking: &mut NodeUpdateTrackerState) -> anyhow::Result<()> {
  // Clean discovered_node to keep only items not in node_already_updated
  state_node_tracking.discovered_node.retain(|x| !state_node_tracking.node_already_updated.contains(x));
  Ok(())
}



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
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "Shared State Log keys/values after containerd version state update:\n",
            &format!("{}", shared_state.log)
          );
        },
      l => {
          // log the line to see how it looks
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "Line: ",
            l
          );
        // // this is to parse
        // node_update_tracker_state.discovered_node.push(l.to_string());
        // // now we update the field `node_name` in `shared_state` `PipelineState` taking the first index from `node_update_tracker_state`
        // // if coming for new round should get new node as we have filtered above what was already done out of the list of nodes to do `discovered_node`
        // let name = &node_update_tracker_state.discovered_node[0].to_string();
        // let list_part_name_to_parse = name.split(" ").collect::<Vec<&str>>();
        // let mut parsed_name = String::new();
        // for i in 0..list_part_name_to_parse.len() {
        //   if i == list_part_name_to_parse.len()-1 {
        //     parsed_name += list_part_name_to_parse[i]
        //   }
        // }
        // let debug_var = node_update_tracker_state.discovered_node.iter().map(|x| x.to_string()).collect::<String>();
        let parsed_name = line_parser(l);
        shared_state.update_shared_state_info("node_name", &parsed_name);
        // debug to see node name
        let _ =  print_debug_log_file(
          "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
          "PARSED NAME NODE DISCOVERED:",
          &parsed_name
        );
        // we add the node name to the discovered nodes
        node_update_tracker_state.discovered_node.push(parsed_name);
        // let _ =  print_debug_log_file(
        //   "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
        //   "check what inside vec discover nodde:",
        //   &debug_var
        // );    
      
      }, // end of last match leg
    } // end of match pattern
    let _ =  print_debug_log_file(
      "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
      "EVOLUTION OF discover node [0]",
      &node_update_tracker_state.discovered_node[0].clone()
    );
    // then we update the `node_type` field of `shared_state` PipelineState`
    if node_update_tracker_state.discovered_node[0].contains("controller") {
      // update the `node_type` and the `status`
      shared_state.update_shared_state_node_type(ClusterNodeType::Controller);
      shared_state.update_shared_state_status(UpgradeStatus::InProcess);
      let _ =  print_debug_log_file(
        "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
        "IS CONTROLLER discover node [0]",
        &node_update_tracker_state.discovered_node[0].clone()
      );
    } else {
      // update the `node_type` and the `status`
      shared_state.update_shared_state_node_type(ClusterNodeType::Worker);
      shared_state.update_shared_state_status(UpgradeStatus::InProcess);
      let _ =  print_debug_log_file(
        "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
        "IS WORKER discover node [0]",
        &node_update_tracker_state.discovered_node[0].clone()
      );	
    }
    // we update now node name:
    let name = node_update_tracker_state.discovered_node[0].clone();
    shared_state.update_shared_state_info("node_name", &name);
    // we mark this step already done so for next round, it will be skipped 
    node_update_tracker_state.discovery_already_done = true;
  }	
}

/* this function is for subsequent round runs to just get the node name and role */
// versions will be pulled and updated accordingly when the step after madison is going to update the TUI with newest version
// could add logic to get actual versions, but will not do for the moment, it has to work in the simple form
pub fn next_rounds_node_state_information_update(
  shared_state: &mut PipelineState,
  node_update_tracker_state: &mut NodeUpdateTrackerState,
  ) {
 
  // we get first node available in the TO DO list of nodes to upgrade and parse it properly
  let name = node_update_tracker_state.discovered_node[0].clone();
  // update the shared state infos
  shared_state.update_shared_state_info("node_name", &name);
  // debug to see node name
  let _ =  print_debug_log_file(
    "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
    "NEW ROUND discover node [0]",
    &name
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

}

/** Madison Parser **/
// this  will parse the version that we got from our `apt-cache madison` command to get the full version for the next steps upgrades of `kubeadm`
pub fn madison_get_full_version_for_kubeadm_upgrade_saved_to_state(line: &str, desired_version_state: &mut DesiredVersions) {
  let _ = print_debug_log_file(
    "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
    "Inside Parser Madison (full line): ",
    line
  );

  let desired_version_clone = desired_version_state.target_kube_versions.clone();
  let user_desired_kube_components_version = desired_version_clone.split(".").collect::<Vec<&str>>();
  let major_version = user_desired_kube_components_version[0];
  let minor_version = user_desired_kube_components_version[1];
  // if user play to much and give use a version like `1.29.34.45.etc...`, we want just the first (major version) and second (minor version)
  let formatted_version = format!("{}.{}", major_version, minor_version);
  let _ = print_debug_log_file(
    "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
    "Inside Parser Madison (formatted version): ",
    &formatted_version
  );
  if line.contains(&formatted_version) {
    let splitted_line = line.split(" ").collect::<Vec<&str>>();
    // ["[Madison", "Version][OUT]", "", "", "", "kubectl", "|", "1.29.15-1.1", "|", "https://pkgs.k8s.io/core:/stable:/v1.29/deb", "", "Packages"]
    let parsed_line = splitted_line[7];
    // here we parse again but to get the version that is shorter for `kubedam apply` command
    // should be 1.29.15 without the `-1.1` from the madison version needed to upgrade kube components
    let parsed_upgrade_apply_version = parsed_line.split("-").collect::<Vec<&str>>()[0].to_string();
    let _ = print_debug_log_file(
      "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
      "Inside Parser Madison (parsed line): ",
      parsed_line
    );
    let _ = print_debug_log_file(
      "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
      "Inside Parser Madison (parsed_upgrade_apply_version): ",
      &parsed_upgrade_apply_version
    );
    // here add to state using the implemented function
    desired_version_state.add("madison_pulled_full_version", parsed_line);
    desired_version_state.add("madison_parsed_upgrade_apply_version", &parsed_upgrade_apply_version);
  }

  // log the 'madison_parsed_upgrade_apply_version' line as it seems that it is empty in later steps
  let madison_upgrade_apply_version = desired_version_state.madison_parsed_upgrade_apply_version.clone();
  let _ = print_debug_log_file(
    "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
    "***********************************************************Inside Parser Madison (parsed line): ",
    &madison_upgrade_apply_version
  );
} 

// make a function that would after upgrade plan get the state pipeline_state (shared state) update the kube component versions
// and the containerd version as it has to match what user wanted
pub fn check_upgrade_plan_output_available_next_version(
    line: &str,
    desired_version_state: &mut DesiredVersions,	
  ) -> anyhow::Result<()> {
  // this will check the content of the output of `upgrade plan` which will confirm that our new `kubeadm` version is available and we can apply
  // we make this function Fail the app and stop at that step if it is not present
  if line.contains("[upgrade/versions] Target version: v") {
    if let Some(version) = line.split("[upgrade/versions] Target version: v").nth(1) {
      let version = version.trim(); // clean any whitespace
      if version != desired_version_state.target_kube_versions {
        Err(anyhow::anyhow!(
          "{} mismatch: expected `{}`, got `{}`",
          "[upgrade/versions] Target version: v", desired_version_state.target_kube_versions, version
        ))
      } else { Ok(()) }
    } else { Ok(()) }
  } else { Ok(()) }
}

pub fn check_upgrade_plan_version_and_update_shared_state_versions(
    line: &str,
    desired_version_state: &mut DesiredVersions,
    shared_state: &mut PipelineState,
  ) -> anyhow::Result<()> {
  // Defining a helper macro to avoid repeating code
  // we create a macro that will apply only locally to this function and use `{{ }}` to avoid any issues of interpretation
  macro_rules! check_and_update {
    ($prefix:expr, $desired:expr, $key:expr) => {{
      if line.contains($prefix) {
        if let Some(version) = line.split($prefix).nth(1) {
          let version = version.trim(); // clean any whitespace
          if version != $desired {
            return Err(anyhow::anyhow!(
              "{} mismatch: expected `{}`, got `{}`",
              $key, $desired, version
            ));
          } else {
            shared_state.update_shared_state_info($key, version);
          }
        }
      }
    }};
  }

  // Now use the macro for each component
  check_and_update!(
    "kubeadm_plan ",
    &desired_version_state.target_kube_versions,
    "kubeadm_v"
  );
  check_and_update!(
    "kubelet_plan ",
    &desired_version_state.target_kube_versions,
    "kubelet_v"
  );
  check_and_update!(
    "kubectl_plan ",
    &desired_version_state.target_kube_versions,
    "kubectl_v"
  );
  check_and_update!(
    "containerd_plan ",
    &desired_version_state.target_containerd_version,
    "containerd_v"
  );

  Ok(())
}

/* This for upgrade apply on controller node */
// we want just to fails if the versions are different.. but should be fine
// also we want to upgrade the state of the cluster from processing to upgraded
pub fn check_version_upgrade_apply_on_controller(
    line: &str,
    desired_version_state: &mut DesiredVersions,
    pipeline_state: &mut PipelineState,	
  ) -> anyhow::Result<()> {
  // this will check the content of the output of `upgrade plan` which will confirm that our new `kubeadm` version is available and we can apply
  // we make this function Fail the app and stop at that step if it is not present
  let version = format!("v{}", desired_version_state.madison_parsed_upgrade_apply_version.clone()); 
  if line.contains("[upgrade/successful]") && line.contains(&version) {
    let line_vec = line.split("\"").collect::<Vec<&str>>();
    let line_v = line_vec[1].split("v").collect::<Vec<&str>>();
    if line_v[1].trim() != version {
      Err(anyhow::anyhow!(
        "{} mismatch: expected `{}`, got `{}`",
        "[upgrade/versions] Target version: v", &desired_version_state.madison_parsed_upgrade_apply_version, version
      ))
    } else {
    	pipeline_state.update_shared_state_status(UpgradeStatus::Upgraded);
    	Ok(())
    }
  } else { Ok(()) }
}

/*  This to check `Worker` node has been updated properly*/
// here we are trying to invalidate, just checking to update the state, next step will check that field and if not `Upgraded` it will fail
pub fn check_worker_update_node_on_worker(
    line: &str,
    pipeline_state: &mut PipelineState,	
  ) -> anyhow::Result<()> {

  if line.contains("successfully") && line.contains("updated") {
    // just upgrade the state of the `worker` node upgrade status
  	pipeline_state.update_shared_state_status(UpgradeStatus::Upgraded);
  	Ok(())
  } else if line.contains("[upgrade/health] FATAL") {
  	  Err(anyhow::anyhow!("Upgrade Apply failed: {}", line))
  } else {
  	Ok(())
  }
  
}

/*  This to check `Worker` node has been updated properly*/
// here we are trying to invalidate, just checking to update the state, next step will check that field and if not `Upgraded` it will fail
pub fn check_node_upgrade_state_and_kubeproxy_version(
    line: &str,
    desired_version_state: &mut DesiredVersions,
    pipeline_state: &mut PipelineState,
    node_tracker: &mut NodeUpdateTrackerState,
    node_name: &str,
  ) -> anyhow::Result<()> {

  // First we check if the upgrade has been done by checking state of the upgrade which should have been changed by the previous step
  // if it is not we invalidate step and return error
  let node_status = pipeline_state.log.clone().shared_state_iter("upgrade_status")[0].clone();
  if node_status != "Upgraded!" {
  	return Err(anyhow::anyhow!(
  	  "{} mismatch: expected `{}`, got `{}`",
  	  "Node Status Of Upgrade", "Upgraded!", node_status
  	))
  }

  // now we can check on the `kube proxy` version upgrade detecting the key in the returned line `kubeproxy `
  if line.contains("kubeproxy ") {
  	// we parse the line
  	let line_vec = line.split("kubeproxy ").collect::<Vec<&str>>();
  	let parsed_line_kube_proxy_actual_version = line_vec[1];
  	let desired_version = desired_version_state.madison_parsed_upgrade_apply_version.clone();
  	// if statement expect return type Result so use `return` keyworkd here otherwise it will be just a bare statement for `rust compiler`
  	// and it will get what its want and not complain also that no `else` if given. Need after to put `Ok(())` so that also it doesn't cry
  	if parsed_line_kube_proxy_actual_version != desired_version {
      return Err(anyhow::anyhow!(
        "{} mismatch: expected `{}`, got `{}`",
        "[upgrade/versions] Target version: v", desired_version, parsed_line_kube_proxy_actual_version
      ))
  	}
  } else {
  	return Err(anyhow::anyhow!("Line mismatch: expected `kubeproxy ` in line, but got: `{}`", line))
  }

  // we add the name of the node already upgraded to the list of the node DONE. `node_name` is already an `&str`
  node_tracker.add_node_already_updated(node_name);
  // we delete the node already done from the list of `dicovered_node` if there is any left inside of it
  if !node_tracker.discovered_node.is_empty() {
    node_tracker.discovered_node.remove(0);
  }
  // we log state to check if update is done properly
  let _ = print_debug_log_file(
     "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
     "NODE TRACKER TO_DO:\n",
     &node_tracker.discovered_node.iter().cloned().collect::<Vec<_>>().join("\n")
   );
   let _ = print_debug_log_file(
       "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
       "NODE TRACKER DONE:\n",
       &node_tracker.node_already_updated.iter().cloned().collect::<Vec<_>>().join("\n")
     );
  let name = node_name.to_string();
  if node_tracker.discovered_node.contains(&name) {
  	return Err(anyhow::anyhow!("Node name is still inside the list of nodes TO_DO.. Nazeeeeeee?!"));
  }
  Ok(())
}
