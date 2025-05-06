#![allow(unused_imports)]
use anyhow::Result;
use tokio::{sync::{mpsc, watch}};
use std::io::stdout;
use ratatui::{prelude::{CrosstermBackend, Terminal}};
// the `TUI` manager drawer/painter
use core_ui::state::{AppState, StepColor};
use core_ui::ui::{draw_ui, redraw_ui};
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
// sleep to simulate delay and see if receiver will fail or buffer be exploded
use tokio::time::{sleep, Duration};


pub async fn run() -> Result<()> {

  // 1. Static list of step names used to initialize UI state
  let step_names = [
    "Discover Nodes",
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

  /* 2. state, terminal, single log channel ------------------------------ */
  let (mut state, _tx_state, _rx_state) = AppState::new(&step_names);
  let backend = CrosstermBackend::new(stdout());
  let mut term   = Terminal::new(backend)?;
  term.clear()?;

  let (tx_log, mut rx_log) = mpsc::channel::<String>(1024);

  /* 3. engine loop ------------------------------------------------------- */
  for (idx, mut step) in steps.into_iter().enumerate() {
    /* 3.1 mark running (green) */
    state.steps[idx].color = StepColor::Green;
    redraw_ui(&mut term, &state)?;

    let _ = print_debug_log_file("/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt", "WILL STARTooo" , step.name());
    /* 3.2 run the step – this awaits until its child process ends */
    match step.run(&tx_log).await {
      Ok(()) => {
        state.steps[idx].color = StepColor::Blue;
        let _ = print_debug_log_file("/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt", "SUCCESS" , step.name());
      }
      Err(e) => {
        let _ = print_debug_log_file("/home/creditizens/kubernetes_upgrade_rust_tui/debugging/debugging_logs.txt", "FAILED" , step.name());
        state.steps[idx].color = StepColor::Red;
        let err_msg = format!("Step '{}' failed: {e}", step.name());
        state.log.push(err_msg.clone());
        eprintln!("step failed: {e}");
        return Err(e.into()); // stop the process explicitly so that no other step runs
      }
    }

    /* 3.3 drain any log lines produced during the step */
    while let Ok(line) = rx_log.try_recv() {
      state.log.push(line);
    }

    /* 3.4 redraw with updated colours + new log */
    redraw_ui(&mut term, &state)?;
    sleep(Duration::from_secs(10)).await;
  }

  /* 4. final paint ------------------------------------------------------- */
  term.draw(|f| draw_ui(f, &state))?;
  Ok(())
}
