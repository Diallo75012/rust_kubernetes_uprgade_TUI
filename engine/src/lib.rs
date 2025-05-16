//#![allow(unused_imports)]
use anyhow::Result;
// use tokio::{sync::mpsc};
use tokio::time::{
  // sleep,
  Duration
};
use std::io::stdout;
use ratatui::{prelude::{CrosstermBackend, Terminal}};
use crossterm::{
  event::{self, Event, KeyCode, poll,  DisableMouseCapture, EnableMouseCapture},
  execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
// the `TUI` manager drawer/painter
use core_ui::{
  state::{
    AppState,
    PipelineState,
    //NodeDiscoveryInfo,
    // StepColor,
    //ClusterNodeType,
    //UpgradeStatus,
    NodeUpdateTrackerState,
    ComponentsVersions,
    DesiredVersions,
  },
  ui::run_input_prompt,
  // parse_lines::{
  //   state_updater_for_ui_good_display,
  //   madison_get_full_version_for_kubeadm_upgrade_saved_to_state,
  //   check_upgrade_plan_version_and_update_shared_state_versions,
  //   check_upgrade_plan_output_available_next_version,
  //   check_version_upgrade_apply_on_controller,
  //   check_worker_update_node_on_worker,
  //   check_node_upgrade_state_and_kubeproxy_version,
  // },
};
use core_ui::ui::draw_ui;
// crates from engine/src
mod upgrade_steps_runner;
use crate::upgrade_steps_runner::run_upgrade_steps;
// the common helper `trait` shared between `steps`
// use shared_traits::step_traits::Step;
use shared_fn::debug_to_file::print_debug_log_file;


pub async fn run() -> Result<()> {

  // 1. Static list of step names used to initialize UI state
  let step_names = [
    "Discover Nodes",
    "Pull Repo Key",
    "Madison Version",
    "Cordon",
    "Drain",
    "Upgrade Plan",
    "Upgrade Apply CTL",
    "Uprgade Node",
    "Uncordon",
    "Restart Services",
    "Verify Core DNS Proxy",
  ];

  /* 2. state, terminal, single log channel ------------------------------ */
  let (mut state, _tx_state, _rx_state) = AppState::new(&step_names);
  // `mut` for `pipeline_state` as we want to mutate the `color` field in this function
  let (mut pipeline_state, _tx_pipeline_state, _rx_pipeline_state) = PipelineState::new(/* PipelineState */); // we initialize a Shared State
  let (mut node_update_tracker_state, _tx_node_update_state, _rx_node_update_state) = NodeUpdateTrackerState::new(/* NodeUpdateState */); // we initialize Node update Tracker State
  let (mut components_versions, _tx_components_versions, _rx_components_versions) = ComponentsVersions::new(/* ComponentsVersions */); // we initalize Components Versions State
  let mut desired_versions = DesiredVersions::new(); // initializing `DesiredVersions` state
  /* ******************  initialize all other states trhat we might need ******************* */


  /* 3. We get user input versions */
  // we setup the `backend` which is `Crossterm` in `ratatui` (but we used generics so we can swap it for anything eled)
  // `stdout` imported from `std::io`
  let backend = CrosstermBackend::new(stdout());
  let mut term  = Terminal::new(backend)?;
  // might need to get the state that will save user input to this as parameter
  // so we have access to it there and will be the state that we have initialized here and not another state
  execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
  crossterm::terminal::enable_raw_mode()?;
  run_input_prompt(&mut term, &mut desired_versions, false)?;
  run_input_prompt(&mut term, &mut desired_versions, true)?;
  crossterm::terminal::disable_raw_mode()?;
  execute!(term.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
  
  let _ = print_debug_log_file(
    "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
    "Desired Versions State: ",
    &format!("kube desired version: {}\ncontaienrd desired version: {}", desired_versions.target_kube_versions, desired_versions.target_containerd_version)
  );

  /* 4. we clear the terminal */
  // we clear the backend again here as we have done it before for the popup management to get user input.
  // so for the steps running it will be new clear `backend`
  term.clear()?;

  /* 5. We run steps update ont he first discovered node and feed the state with all other nodes so that later we can upgrade the other discovered nodes */
  // call the loop function
  match run_upgrade_steps(
    &mut term,
    &mut state,
    &mut pipeline_state,
    &mut desired_versions,
    &mut node_update_tracker_state,
    &mut components_versions,
  ).await {
    // steps done without issue
  	Ok(()) => {
  	  // we just log it and it can keep going and will enter next whule loop to run other steps
  	  let _ = print_debug_log_file(
  	    "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt",
  	    "SUCCESS STEP ROUND",
  	    "Step Went Well!"
  	  );
  	},
  	// if any error.. we stop explicitly the sequence
  	Err(e) => {
  	  // this for log written to file
  	  let _ = print_debug_log_file(
  	    "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt",
  	    "FAILED ROUND: ",
  	    &format!("{}", e)
  	  );
  	  return Err(e); // stop the process explicitly so that no other steps rounds
  	}  	
  }

  /* 6. Here we run steps on other discovered nodes in a loop that will update the node tracker count and will exit the loop to end finish */
  // so the here we just check onthe state `node_update_tracker_state` which is a `Vec<String>` and keep upgrading the other nodes present in it
  // there is already a function in `core/src/parsed_lines.rs` that will update the state
  // and eliminate what is already done to put it in another `Vec<String>: `node_update_tracker_state.node_already_updated`
  while !node_update_tracker_state.discovered_node.is_empty() {
    /* ** Here we are going to reinitialize all other states other than `node_update_tracker_state` and `desired_versions`
       which are the only ones that we need to live longer ** */
    let (mut state, _tx_state, _rx_state) = AppState::new(&step_names);
    // `mut` for `pipeline_state` as we want to mutate the `color` field in this function
    let (mut pipeline_state, _tx_pipeline_state, _rx_pipeline_state) = PipelineState::new(/* PipelineState */); // we initialize a Shared State
    let (mut components_versions, _tx_components_versions, _rx_components_versions) = ComponentsVersions::new(/* ComponentsVersions */); // we initalize Components Versions State
  
    let _ = print_debug_log_file(
      "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
      "Inside While Loop For Next Round (node update tracker state: discovered_node):\n",
      &node_update_tracker_state.discovered_node.iter().cloned().collect::<Vec<_>>().join("\n")
    );
    let _ = print_debug_log_file(
      "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
      "Inside While Loop For Next Round (node update tracker state: node_already_updated):\n",
      &node_update_tracker_state.node_already_updated.iter().cloned().collect::<Vec<_>>().join("\n")
    );
    // call the loop function
    match run_upgrade_steps(
      // initialized
      &mut term,
      // initialized
      &mut state,
      // initialized
      &mut pipeline_state,
      // not initialized as needed for the full app lifetimuuu
      &mut desired_versions,
      // not initialized as needed for the full app lifetimuuu
      &mut node_update_tracker_state,
      // initialized
      &mut components_versions,
    ).await {
      // steps done without issue
      Ok(()) => {
        // we just log it and it can keep going and will enter next whule loop to run other steps
  	    let _ = print_debug_log_file(
  	      "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt",
  	      "SUCCESS STEP ROUND",
  	      "Step Went Well!"
  	    );
      },
      // if any error.. we stop explicitly the sequence
  	  Err(e) => {
  	    // this for log written to file
  	    let _ = print_debug_log_file(
  	      "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt",
  	      "FAILED ROUND: ",
  	      &format!("{}", e)
  	    );
  	    return Err(e); // stop the process explicitly so that no other steps rounds
      }  	
    } // end of match
  } // end of while loop


  /* 7. final paint ------------------------------------------------------- */
  // this is the last print when the  loop is done so steps are done
  state.log.push("All Steps Are Done! Press 'q' to quit".to_string());
  term.draw(|f| draw_ui(f, &mut state, &mut pipeline_state, &mut desired_versions))?;

  /* 8. we wait here for user to press `q` to quit the app */
  // puting thos here to have the chance to get those user keytrokes captured (so we put it after the `for loop
  // as inside it won't be in same scope so not working)
  execute!(stdout(), EnableMouseCapture)?;
  crossterm::terminal::enable_raw_mode()?;
  loop {
    if poll(Duration::from_millis(10))? {
      if let Event::Key(key) = event::read()? {
        match key.code {
          KeyCode::Char('q') => {
            let _ = print_debug_log_file(
              "/home/creditizens/kubernetes_upgrade_rust_tui/debugging/shared_state_logs.txt",
              "Engine/lib.rs Quit Pressed: ",
              "True"
            );
            crossterm::terminal::disable_raw_mode()?;
            execute!(stdout(), LeaveAlternateScreen,  DisableMouseCapture)?;
            return Ok(())
          }, // quit
          // here for scrolling but might get rid of this code as i have changed my mind and would show in TUI only the last lines of logs so no scrolling
          KeyCode::Up | KeyCode::Char('k') => {
            state.log_scroll_offset = state.log_scroll_offset.saturating_sub(1);
          },
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
  }
  // crossterm::terminal::disable_raw_mode()?;
  // execute!(stdout(), LeaveAlternateScreen,  DisableMouseCapture)?;
  // Ok()
}
