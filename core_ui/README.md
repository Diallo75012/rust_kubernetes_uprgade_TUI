# `core_ui/` – Shared Types and Utilities

This is the foundation of the project. Every other crate depends on this.
It contains:
- state.rs: holds AppState, shared between engine and UI
- cmd.rs: managing commands stdout and stderr
- ui.rs: drawing logic for header/sidebar/footer and log area
- and more...

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
