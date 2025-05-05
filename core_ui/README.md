# `core_ui/` – Shared Types and Utilities

This is the foundation of the project. Every other crate depends on this.
It contains:
- state.rs: holds AppState, shared between engine and UI
- cmd.rs: managing commands stdout and stderr
- style.rs: defines colors for UI steps (grey/green/blue...)
- ui.rs: drawing logic for header/sidebar/footer and log area

```figma
┌──────────┐
│  core/   │◀────────┐
└──────────┘         │
       ▲             ▼
     engine     steps/*
       │             │
       ▼             ▼
     app/ ←──────────┘
```
