# `engine/` – The Orchestrator

This is the brain of the upgrade process. It:
- Imports all Step crates
- Runs each one sequentially
- Sends updates to the TUI via an mpsc channel

```figma
┌────────────┐
│  engine/   │
└─────┬──────┘
      │
┌─────▼─────┐
│ core::AppState
│ core::Action
└─────┬─────┘
      │
┌─────▼───────┐     ┌──────────────┐
│ run_upgrade │───▶│ steps::StepN │
└─────────────┘     └──────────────┘
```
