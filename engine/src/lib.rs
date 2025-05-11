//#![allow(unused_imports)]
use anyhow::Result;
use tokio::{sync::mpsc};
use tokio::time::{sleep, Duration};
use std::io::stdout;
use ratatui::{prelude::{CrosstermBackend, Terminal}};
use crossterm::event::{self, Event, KeyCode};
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
  },
  update_shared_state_info::state_updater_for_ui_good_display,  
};
use core_ui::ui::{draw_ui, redraw_ui};
// all the `steps`
use step_discover_nodes::DiscoverNodes;
/*
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
*/
// the common helper `trait` shared between `steps`
use shared_traits::step_traits::Step;
use shared_fn::debug_to_file::print_debug_log_file;


pub async fn run() -> Result<()> {

  // 1. Static list of step names used to initialize UI state
  let step_names = [
    "Discover Nodes",
  ];
  /*
    "Pull Repo Key",
    "Madison Version",
    "Cordon",
    "Drain",
    "Upgrade Plan",
    "Uprgade Apply CTL",
    "Uprgade Node",
    "Uncordon",
    "Restart Services",
    "Verify Core DNS Proxy",
  ];
  */

  // 2. Instantiate all step implementations, boxed as trait objects
  // `Send`: This means the type can be safely sent between threads.
  // `Sync`: This means it can be safely shared between threads.
  let steps: Vec<Box<dyn Step + Send + Sync>> = vec![
    Box::new(DiscoverNodes),
  ];
  /*
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
  */

  /* 2. state, terminal, single log channel ------------------------------ */
  let (mut state, _tx_state, _rx_state) = AppState::new(&step_names);
  // `mut` for `pipeline_state` as we want to mutate the `color` field in this function
  let (mut pipeline_state, _tx_pipeline_state, _rx_pipeline_state) = PipelineState::new(/* PipelineState */); // we initialize a Shared State
  let (mut node_update_tracker_state, _tx_node_update_state, _rx_node_update_state) = NodeUpdateTrackerState::new(/* NodeUpdateState */); // we initialize Node update Tracker State
  let (mut components_versions, _tx_components_versions, _rx_components_versions) = ComponentsVersions::new(/* ComponentsVersions */); // we initalize Components Versions State
  // `stdout` imported from `std::io`
  let backend = CrosstermBackend::new(stdout());
  let mut term  = Terminal::new(backend)?;
  // start with a clear sheet
  term.clear()?;

  // will be following the order in which data is sent (`send`) to channel and received (`recv`) in order
  // transmitter `tx_log` and receiver `rx_log`
  let (tx_log, mut rx_log) = mpsc::channel::<String>(1024);
  //let (pipeline_tx_log, mut pipeline_rx_log) = mpsc::channel::<String>(1024);
  //let (node_update_tracker_tx_log, mut node_update_tracker_rx_log) = mpsc::channel::<String>(1024);
  
  /* Or Maybe here in the loop action specific function to specific step and update the `PipelineState` which will call `redraw( calling `draw_ui`) */
  /* 3. engine loop ------------------------------------------------------- */
  // `.enumerate()` like in Python to get `index` and `value`
  for (idx, mut step) in steps.into_iter().enumerate() {

    /* 3.1 mark running (green) (ratatui tui coloring stuff)*/
    state.steps[idx].color = StepColor::Green;
    pipeline_state.color = StepColor::Blue;
    // no color for `node update tracker`: we will do it on render if needed

    // we repaint the tui to get that green colored step out there
    redraw_ui(&mut term, &mut state, &mut pipeline_state)?;

    // this is custom function made to get some logs as the `tui` doesn't permit to see `println/eprintln` so we write to a file.
    let _ = print_debug_log_file("/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt", "WILL STARTooo" , step.name());

    /* 3.2 run the step – this awaits until its child process ends */
    // we borrow `tx_log` (transmitter buffer/output)
    match step.run(&tx_log).await {
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
      		state_updater_for_ui_good_display(step.name(), &line, &mut pipeline_state, &mut node_update_tracker_state, &mut components_versions);
      	},
      	/*
        "Pull Repo Key"  => {},
      	"Madison Version"=> {},
      	"Upgrade Plan"   => {},
        "Upgrade Apply"  => {}, 
 	    "Upgrade Node"   => {}, 
      	"Veryfy Core DNS Proxy" => {},
      	*/
      	_ => {},
      }
      
      // if need can write `line` to a debug file. Line is borrowed above and here moved so bye bye `line`!
      state.log.push(line);
    }

    /* 3.4 redraw with updated colours + new log */
    redraw_ui(&mut term, &mut state, &mut pipeline_state)?;
    // just simulating some processing waiting a bit... will be replaced by real command duration....
    sleep(Duration::from_secs(10)).await;
  }

  /* 4. final paint ------------------------------------------------------- */
  term.draw(|f| draw_ui(f, &mut state, &mut pipeline_state))?;
  Ok(())
}
