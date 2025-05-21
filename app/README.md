# `app/` – The CLI Entry Point

This is the binary crate that starts the program. It:
- Sets up the terminal with Crossterm
- Builds the list of Step objects
- Starts the async upgrade() from engine

```figma
┌────────────┐
│   app/     │
└─────┬──────┘
│
┌─────▼────────┐
│ main.rs      │
│ ├─ enable UI │
│ └─ call run()│
└──────────────┘
```
