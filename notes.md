# Kubernetes Upgrade TUI Manager
- This project is done after having upgraded manually my Kubernetes cluster twice so I had created a bash script to handle that but as I am learning Rust
  I found here an amazing occasion to make something nice, a `TUI` with less dependencies as possible (for my level of coding) and I will learn by trying
  and making mini modules along the way and have `ChatGPT` as my `Senior Enginer Not GateKeeper` guidng me along the way.

# App Desired Flow:
- user enters a intermediary version or Kuberneter like `1.<...>` in the `TUI` and then the backend would do the full upgrade of the cluster
  - cordon/uncordon
  - pull key/repo
  - upgrade `kubect`/`kueadm`/`kubelet` and optionally `containerD` (as it doesn't need updates as frequently as kubernetes does)
  - parsers, checkers of version
  - `TUI` having all steps listed. And changing color of steps when validated and done. And it is `sequential` steps.
  - `TUI` will display all the time the versions of `kubectl`, `kubelet`, `kubeadm` and `containrd` and change those colors if they changed from start version avaialble
  - `TUI` would be simple but dynamic, focusing on each steps. thereofore, streaming `stdout`/`stderr` to display what is going on
  - `TUI` won't put notification service to user to make it easy willjust change the screen states to say if it successful or stop with red color if it is failed
  - `TUI` steps of upgrade will be greyed out and become `green` when validated and done, `orange` when being performed, `red` which means error so stop at that step
  - `TUI` when it is `red` step and stops, will display error message. User can only re-enter the version to restart all, or user need to contact `devs` to fix app



# PLAN

## 1 · Workspace map (v2)
```markdown
rust‑k8s‑upgrade‑tui/
│
├── Cargo.toml                     # [workspace]
│
├── core/                          # cross‑cutting helpers
│   ├── cmd_exec/                  # spawn + stream commands
│   ├── parser/                    # pure & tested output parsers
│   ├── fs_state/                  # read/write state.json
│   └── ui/                        # ratatui widgets & colour palette
│
├── steps/                         # one crate per upgrade step
│   ├── discover_nodes/            # kubectl get nodes
│   ├── pull_repo_key/             # add‑apt‑key & repo
│   ├── madison_version/           # apt-cache madison pick‑latest
│   ├── cordon/                    # kubectl cordon
│   ├── drain/                     # kubectl drain
│   ├── upgrade_plan/              # kubeadm upgrade plan
│   ├── upgrade_apply_ctl/         # kubeadm upgrade apply
│   ├── upgrade_node/              # kubeadm upgrade node
│   ├── uncordon/                  # kubectl uncordon
│   ├── restart_services/          # systemctl restart kubelet/containerd
│   └── verify_coredns_proxy/      # kubectl get ds/cm …   (optional)
│
├── engine/                        # finite‑state runner
│   └── src/lib.rs
│
└── app/                           # binary entry‑point
    └── src/main.rs

```

**Why this shape?**
- The core/ cluster collects utilities every step reuses: command spawning, parsing, persistent state, and UI.
- Each folder in steps/ compiles into its own library crate exposing exactly one public async function—e.g.`pub async fn run(cfg: &Config, st: &mut AppState) -> Result<()>`
- That maps 1‑to‑1 with your Trello/Miro “cards”.
- `engine/` glues the steps together in hard‑wired order; if any `run()` returns `Err`, the engine stops and the TUI shows the failing step in red.

## 2 · How a single step crate looks
```rust
steps/cordon/src/lib.rs
```
```rust
use core::{cmd_exec::Cmd, fs_state::State, error::StepError};

pub async fn run(state: &mut State) -> Result<(), StepError> {
    state.set_status("cordon", Status::Running)?;

    let nodes = state.get_controllers();
    for n in &nodes {
        let mut child = Cmd::new("kubectl").arg("cordon").arg(n).spawn()?;
        child.stream_to_log("cordon").await?;          // live log → TUI
        child.expect_exit0().await?;                  // or StepError::CmdFailed
    }

    state.set_status("cordon", Status::Done)?;
    Ok(())
}
```
No unwraps, only Result.
The helper `stream_to_log()` pushes every stdout/stderr line into a tokio::sync::mpsc that the TUI watches.

## 3 · Engine: sequential but async‐friendly
```rust
pub async fn upgrade(mut state: State) -> anyhow::Result<()> {
    use steps::*;

    discover_nodes::run(&mut state).await?;
    pull_repo_key::run(&mut state).await?;
    madison_version::run(&mut state).await?;
    cordon::run(&mut state).await?;
    drain::run(&mut state).await?;
    upgrade_plan::run(&mut state).await?;
    upgrade_apply_ctl::run(&mut state).await?;
    upgrade_node::run(&mut state).await?;
    uncordon::run(&mut state).await?;
    restart_services::run(&mut state).await?;
    verify_coredns_proxy::run(&mut state).await?;

    Ok(())
}
```
The whole upgrade is awaited from main(), so steps execute strictly one‑after‑another, yet all inner I/O is non‑blocking.

## 4 · Persistent AppState (no env vars)
```rust
#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub version_target: SemVer,
    pub step_status: HashMap<&'static str, StepStatus>,
    pub node_inventory: Vec<Node>,     // names + roles
    pub component_versions: Components // kubeadm/kubelet/…
}
```
Saved every time set_status() mutates it to `$XDG_STATE_HOME/rk8s-tui/state.json`
If the app crashes, restarting reloads and jumps to the first incomplete step.

## 5 · Parsing success vs error
- parsers/ crate contains for each command: `enum Outcome<T> { Ok(T), Failed(String /*reason*/), Unknown }`
- Each step decides success like:
```rust
let raw = child.capture().await?;
match parser::kubectl::parse_cordon(&raw) {
    Outcome::Ok(_)       => state.set_status(step, Done),
    Outcome::Failed(e)   => Err(StepError::CmdLogic(e)),
    Outcome::Unknown     => Err(StepError::ParseFail("cordon")),
}
```
For quicker prototyping you can shell‑out to small inline awk scripts,
but long‑term a Rust‑side regex keeps the binary self‑contained and testable

## 6 · Password entry story
**tiny helper in core/cmd_exec:**
- Try sudo‑with‑stdin first (echo "$PW" | sudo -S …).
- If $PW missing, detect the sudo: prompt in stderr and ask the user once via a hidden text‑input box in the TUI (ratatui can read raw keys).
- Cache it inside State for the remainder of the session.

## 7 · TUI widget mapping
UI area 							| Feeds from State 			| Updates when
Left bar – list of steps, colourised grey/orange/green/red 	| .step_status 				| dispatcher broadcasts a change
Top‑right panel – kubeadm/kubelet/kubectl/containerd versions 	| .component_versions 			| after every step
Central log pane – scrolling output 				| mpsc::Receiver<LogLine> 		| each log line

`Ratatui’s tick loop can simply poll log_rx.try_recv() & state_rx.has_changed() once every 100 ms—no extra threads.`

## 8 · Crates Needed
Needed for 			| Crate
async runtime + process 	| tokio
terminal UI 			| ratatui, crossterm
error ergonomics 		| thiserror
parsing	 			| regex
state file 			| serde, serde_json
CLI flags (optional) 		| clap


## 9. To Resume All
- core/ houses shared utilities.
- steps/ mirrors each user‑story card—one crate per atomic action.
- engine/ executes those crates in order; aborts on first error.
- State is JSON on disk + channel‑broadcast in memory; TUI repaints on change.
- Zero unwrap, strictly Result. All terminal I/O is streamed so the UI stays responsive, yet the business flow is strictly sequential.


## 10. Learning checkpoints
- [ ] Ratatui quick‑start — draw a static layout, then refactor to listen to a watch::Receiver<AppState>. (hep blog)[https://raysuliteanu.medium.com/creating-a-tui-in-rust-e284d31983b3?utm_source=chatgpt.com] 
- [ ] Tokio process streaming — stream child stdout/stderr without blocking UI. Rust Users thread has code for capturing println! output. (forum helper)[https://users.rust-lang.org/t/how-to-intercept-stdout-to-display-inside-tui-layout/78823?utm_source=chatgpt.com]
- [ ] kube‑rs Controller — follow the “Application controller” guide to reconcile upgrades. (check `Kube.rs`)[https://kube.rs/controllers/application/?utm_source=chatgpt.com]
- [ ] Spawn & stream: write cmd_exec so kubectl get nodes streams cleanly.
- [ ] Parser TDD: write #[test] cases for the madison parser using captured sample output.
- [ ] State machine: represent every high‑level step as enum Phase; drive it with loop { match phase { … } }.
- [ ] Ratatui basics: build a static layout; then refactor to redraw only when AppState.version_panel changes.
- [ ] Error bubbles: simulate a failed kubectl drain; show step → red, log detail, and exit gracefully.



# Extra Notes

## 6 columns and we need the 3rd column version number the biggest and 5th column v.1.<...> using regex and comparing to user input state 1.<...> just as safe validation
We might need here to use: awk command and print $3 and $5 like: `cat test.txt | awk '{print $3,$5}'` with conditions and more bash stuff
or even better so no need to have those written to a file and do it on the fly `sudo apt-cache madison kubeadm | awk '{print $3,$5}'` and then add conditions 
kubeadm | 1.29.15-1.1 | https://pkgs.k8s.io/core:/stable:/v1.29/deb  Packages
