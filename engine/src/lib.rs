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


pub async fn run() -> Result<()> {
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

    let (mut state, tx_state, _rx_state) = AppState::new(&step_names);
    let backend = CrosstermBackend::new(stdout());
    let mut term = Terminal::new(backend)?;
    term.clear()?;

    let (tx_log, mut rx_log) = mpsc::channel::<String>(1024);

    // Main loop over steps
    for (idx, step) in steps.into_iter().enumerate() {
        // Mark current step as running
        state.steps[idx].color = StepColor::Green;
        redraw_ui(&mut term, &state)?;

        // Non-blocking: drain any accumulated log messages
        while let Ok(line) = rx_log.try_recv() {
            state.log.push(line);
        }

        // Run the step, await completion
        match step.run(tx_log.clone(), tx_state.clone()).await {
            Ok(()) => state.steps[idx].color = StepColor::Blue,
            Err(e) => {
                eprintln!("step failed: {e}");
                break;
            }
        }

        // After step completes, drain remaining log lines
        while let Ok(line) = rx_log.try_recv() {
            state.log.push(line);
        }

        redraw_ui(&mut term, &state)?;
    }

    // Final UI draw
    term.draw(|f| draw_ui(f, &state))?;
    Ok(())
}
