# `steps/` – One Crate per Upgrade Step

Each subfolder is a standalone mini‑crate with one purpose:
- Wrap a real‑world upgrade step (e.g., kubectl cordon, kubeadm plan, etc.)
- Run a shell command
- Stream stdout/stderr to the UI

Each implements the Step trait, imported from core::step_trait.

```figma
┌───────────────┐
│  steps/*/     │  one per upgrade action
└─────┬─────────┘
      ▼
┌──────────────┐
│ impl Step for│
│ cordon,...etc│
└──────────────┘
      │
      ▼
log lines via mpsc::Sender<Action>
```
