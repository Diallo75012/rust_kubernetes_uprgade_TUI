//#![allow(unused_imports)]
use anyhow::Result;
use tokio::{sync::mpsc};
use tokio::time::{
  sleep,
  Duration
};
use std::io::stdout;
use ratatui::{
  prelude::{
    // CrosstermBackend,
    Terminal
  },
  backend::Backend,
};
use crossterm::{
  event::{self, Event, KeyCode, poll,  DisableMouseCapture, EnableMouseCapture},
  execute,
};
// the `TUI` manager drawer/painter
use core_ui::{
  state::{
    AppState,
    PipelineState,
    //NodeDiscoveryInfo,
    StepColor,
    //ClusterNodeType,
    //UpgradeStatus,
    NodeUpdateTrackerState,
    ComponentsVersions,
    DesiredVersions,
  },
  parse_lines::{
    state_updater_for_ui_good_display,
    madison_get_full_version_for_kubeadm_upgrade_saved_to_state,
    check_upgrade_plan_version_and_update_shared_state_versions,
    check_upgrade_plan_output_available_next_version,
    check_version_upgrade_apply_on_controller,
    check_worker_update_node_on_worker,
    check_node_upgrade_state_and_kubeproxy_version,
  },
  ui::redraw_ui,
};
// all the `steps`
use step_discover_nodes::DiscoverNodes;
use step_pull_repo_key::PullRepoKey;
use step_madison_version::MadisonVersion;
use step_cordon::Cordon;
use step_drain::Drain;
use step_upgrade_plan::UpgradePlan;
use step_upgrade_apply_ctl::UpgradeApplyCtl;
use step_upgrade_node::UpgradeNode;
use step_uncordon::Uncordon;
use step_restart_services::RestartServices;
use step_verify_coredns_proxy::VerifyCoreDnsProxy;
// the common helper `trait` shared between `steps`
use shared_traits::step_traits::Step;
use shared_fn::debug_to_file::print_debug_log_file;


pub async fn run_upgrade_steps<B: Backend>(
    term: &mut Terminal<B>,
    state: &mut AppState,
    pipeline_state: &mut PipelineState,
    desired_versions: &mut DesiredVersions,
    node_update_tracker_state: &mut NodeUpdateTrackerState,
    components_versions: &mut ComponentsVersions,
  // This returned `Result` is an `anyhow::Result`
  ) -> Result<()> {

  // 2. Instantiate all step implementations, boxed as trait objects
  // `Send`: This means the type can be safely sent between threads.
  // `Sync`: This means it can be safely shared between threads.
  let steps: Vec<Box<dyn Step + Send + Sync>> = vec![
    Box::new(DiscoverNodes),
    Box::new(PullRepoKey),
    Box::new(MadisonVersion),
    Box::new(Cordon),
    Box::new(Drain),
    Box::new(UpgradePlan),
    Box::new(UpgradeApplyCtl),
    Box::new(UpgradeNode),
    Box::new(Uncordon),
    Box::new(RestartServices),
    Box::new(VerifyCoreDnsProxy),
  ];

  // we clear the backend again here as we have done it before for the popup management to get user input.
  // so for the steps running it will be new clear `backend`
  term.clear()?;
  
  // will be following the order in which data is sent (`send`) to channel and received (`recv`) in order
  // transmitter `tx_log` and receiver `rx_log`
  let (tx_log, mut rx_log) = mpsc::channel::<String>(1024);
  //let (pipeline_tx_log, mut pipeline_rx_log) = mpsc::channel::<String>(1024);
  //let (node_update_tracker_tx_log, mut node_update_tracker_rx_log) = mpsc::channel::<String>(1024);
  
  /* Or Maybe here in the loop action specific function to specific step and update the `PipelineState` which will call `redraw( calling `draw_ui`) */
  /* 3. engine loop ------------------------------------------------------- */
  // `.enumerate()` like in Python to get `index` and `value`
  // maybe here wrapp this in a separate function with arguments &steps, &mut term, &mut state, &mut pipeline_state, &mut desired_versions, &mut node_tracker

  for (idx, mut step) in steps.into_iter().enumerate() {

    execute!(stdout(), EnableMouseCapture)?;
    crossterm::terminal::enable_raw_mode()?;

    /* 3.1 mark running (green) (ratatui tui coloring stuff)*/
    state.steps[idx].color = StepColor::Green;
    pipeline_state.color = StepColor::Blue;
    // no color for `node update tracker`: we will do it on render if needed

    // we repaint the tui to get that green colored step out there
    redraw_ui(term, state, pipeline_state, desired_versions)?;

    // this is custom function made to get some logs as the `tui` doesn't permit to see `println/eprintln` so we write to a file.
    let _ = print_debug_log_file("/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt", "WILL STARTooo" , step.name());

    /* 3.2 run the step – this awaits until its child process ends */
    // we borrow `tx_log` (transmitter buffer/output)
    match step.run(&tx_log, desired_versions, pipeline_state, node_update_tracker_state).await {
      // step done without issue
      Ok(()) => {
        // we paint the sidebar step in blue
        state.steps[idx].color = StepColor::Blue; 
        
        // this only for logs writtten to file so we use `_`
        let _ = print_debug_log_file("/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt", "SUCCESS" , step.name());
      }
      // if any error.. we stop explicitly the sequence
      Err(e) => {
        // this for log written to file
        let _ = print_debug_log_file("/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt", "FAILED" , step.name());
        // we color the step on the sidebar to red
        state.steps[idx].color = StepColor::Red;
        // we use format to have a `String`
        let err_msg = format!("Step '{}' failed: {e}", step.name());
        // as we are stopping the sequence by returning explicitly we don't need to `.clone()` we can consume this `String`
        state.log.push(err_msg);
        // this one because of the `tui` wont work... or is covered by it, can't see it anyways..
        eprintln!("step failed: {e}");
        return Err(e.into()); // stop the process explicitly so that no other step runs
      }
    }

    /* 3.3 drain any log lines produced during the step */
    while let Ok(line) = rx_log.try_recv() {
      /******************************************************************************************************************************
      // create the function not here in the `shared_fn` and then import it here to do the filtering and update of that state on the fly
      // so i can capture the `step` and `l` (line) in a function that will have the full logic of updating the shared state `PipelineState`
      **********************************************************************************************************************************/

      match step.name() {
      	"Discover Nodes" => {
          // we log the state 'node_update_tracker_state' which should have been cleaned up in previous round last step `Verify Core DNS Proxy`
          let _ = print_debug_log_file(
            "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
            "Inside While Loop Discovery Nodes Step Before CleanUp (node update tracker state: discovered_node):\n",
            &node_update_tracker_state.discovered_node.to_vec().join("\n")
          );
      	  // we update states
          state_updater_for_ui_good_display(step.name(), &line, pipeline_state, node_update_tracker_state, components_versions);
      	},
        // create a line to capture the version matching with the `DesiredVersion` (need one more field in the state for that) (if .contains()) and then split(" ") and get [2]
      	"Madison Version" => {
      	  let node_type_in_step = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
      	  if node_type_in_step != "Worker" && line.contains(&desired_versions.target_kube_versions) {
            // if the line has the version we update the state to get madison full version
            // (it is like a double check aa if this fails it meand that the madison command parsing whas wrong)
            madison_get_full_version_for_kubeadm_upgrade_saved_to_state(&line, desired_versions);
            let _ = print_debug_log_file(
              "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
              "Desired Versions Full Version: ",
              &desired_versions.madison_pulled_full_version
            );
          }
      	},
      	"Upgrade Plan" => {
      	  let node_type_in_step = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
      	  if node_type_in_step != "Worker" {
            let _ = check_upgrade_plan_version_and_update_shared_state_versions(&line, desired_versions, pipeline_state);
            let _ = check_upgrade_plan_output_available_next_version(&line, desired_versions);
     	  }
     	  // we sleep a bit to give the time for the node to be ready as i get frequently node not ready state in the next step `Upgrade Apply cTL`
     	  sleep(Duration::from_secs(15)).await;
      	},
      	"Upgrade Apply CTL" => {
      	  let node_type_in_step = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
          if node_type_in_step != "Worker" {
            // put here the funciton that is going to check
            let _ = check_version_upgrade_apply_on_controller(&line, desired_versions, pipeline_state);
          }
      	},
      	"Upgrade Node" => {
      	  let node_type_in_step = pipeline_state.log.clone().shared_state_iter("node_role")[0].clone();
          if node_type_in_step != "Controller" {
      	    // put here the funciton that is going to check
      	    // here we just check to upgrate the state to `Upgraded` and next step will check and invalidate if the state is not `Upgraded`
      	    let _ = check_worker_update_node_on_worker(&line, pipeline_state);
      	  }
      	},
        "Verify Core DNS Proxy" => {
          // put here the funciton that is going to check keyword: `"kubeproxy "`
          let node_name = pipeline_state.log.clone().shared_state_iter("node_name")[0].clone();
          match check_node_upgrade_state_and_kubeproxy_version(&line, desired_versions, pipeline_state, node_update_tracker_state, &node_name) {
            // just want to block the app and stop if any issues
          	Ok(_) => (),
          	Err(e) => return Err(e),
          }
      	},
      	_ => {},
      }
      
      // if need can write `line` to a debug file. Line is borrowed above and here moved so bye bye `line`!
      state.log.push(line);
      // AUTO-SCROLL if at bottom (e.g., near latest lines)
      let log_len = state.log.len();
      // here we use 18 lines so that is fitting our `body` space in the `tui` and we always see last logs
      if state.log_scroll_offset + 18 >= log_len.saturating_sub(1) {
        state.log_scroll_offset = log_len.saturating_sub(18);
      }
    } // end of `while` loop for line parsing

    /* 3.4 redraw with updated colours + new log */
    redraw_ui(term, state, pipeline_state, desired_versions)?;
    // just simulating some processing waiting a bit... will be replaced by real command duration....
    // sleep(Duration::from_secs(10)).await;

    if poll(Duration::from_millis(10))? {
      if let Event::Key(key) = event::read()? {
        match key.code {
          KeyCode::Char('q') => {
            let _ = print_debug_log_file(
              "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
              "Engine/lib.rs Quit Pressed: ",
              "True"
            );
            return Ok(())
          }, // quit
          KeyCode::Up | KeyCode::Char('k') => {
            state.log_scroll_offset = state.log_scroll_offset.saturating_sub(1);
          },
         // here for scrolling but might get rid of this code as i have changed my mind and would show in TUI only the last lines of logs so no scrolling
          KeyCode::Down | KeyCode::Char('j') => {
            state.log_scroll_offset = state.log_scroll_offset.saturating_add(1);
          },
          KeyCode::PageUp => {
            state.log_scroll_offset = state.log_scroll_offset.saturating_sub(10);
          },
          KeyCode::PageDown => {
            state.log_scroll_offset = state.log_scroll_offset.saturating_add(10);
          },
          _ => {},
        }
      }
    }
    crossterm::terminal::disable_raw_mode()?;
    execute!(stdout(), DisableMouseCapture)?;

  } // enf of `for loop`
  if !node_update_tracker_state.discovered_node.is_empty() {
    state.log.push("\n\nAll Steps Are Done For This Round\n\n".to_string());
  } else {
  	state.log.push("\n\nLast Round Done, Congratualations! Cluster Is Fully Upgraded! \n\n".to_string());
  }
  redraw_ui(term, state, pipeline_state, desired_versions)?;
  Ok(())

}
