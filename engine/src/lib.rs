use anyhow::Result;
use core_ui::state::{AppState, StepColor};
use core_ui::ui::draw_ui;
use ratatui::{prelude::{CrosstermBackend, Terminal}};
use std::io;
use tokio::{sync::{mpsc, watch}, task::JoinHandle};
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
use shared_traits::step_traits::Step;


pub async fn run() -> Result<()> {
  // 1. build state
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
    "Verify Core DNS Proxy"
  ];
  let (mut state, tx_state, mut rx_state) = AppState::new(&step_names);
  // we here clone state as it is going to be moved by the `async move` inside the `tokio::spawn`
  // So that we can reuse `state` later on this function
  let mut state_cloned = state.clone();


  // 2. channel for log lines
  let (tx_log, mut rx_log) = mpsc::channel::<String>(1024);

  // 3. UI task – owns the terminal
  // so here we yse in `tokio::spawn` and `async move` so ownership is moved so `AppState` is owned
  let ui_handle: JoinHandle<Result<()>> = tokio::spawn(
    async move {

      // terminal init
      let backend = CrosstermBackend::new(io::stdout());
      let mut term = Terminal::new(backend)?;
      term.clear()?;

      loop {
        // if state was updated
        if rx_state.has_changed().is_ok() {
          // `rx_state` updates the view state
          state_cloned = rx_state.borrow().clone();
        }

        // apply incoming log lines
        while let Ok(line) = rx_log.try_recv() {
          state_cloned.log.push(line);
        }
        // redraw
        term.draw(|f| draw_ui(f, &state_cloned))?;
        // stop when rx_state closed
        if rx_state.has_changed().is_err() { break; }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
      }

      Ok(())
    }
  );

  // 4. sequentially run steps
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

  for step in &steps {
    // mark running
    // now here we can consume the `state` as we arriving to the end of the lifetime of the function
    if let Some(s) = state.steps.iter_mut().find(|s| s.name == step.name()) {
      s.color = StepColor::Green;   // running (green)
    }
    tx_state.send(state.clone()).ok();

    if let Err(e) = step.run(tx_log.clone(), tx_state.clone()).await {
      eprintln!("step failed: {e}");
      break;
    }
  }
  // all done → paint blue
  for s in &mut state.steps { s.color = StepColor::Blue; }
  tx_state.send(state.clone()).ok();

  ui_handle.await??;
  Ok(())
}
