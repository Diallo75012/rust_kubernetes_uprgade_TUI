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
├── Cargo.toml                     # [workspace] source: (doc workspace)[https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html]
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
- [x] Ratatui quick‑start — draw a static layout, then refactor to listen to a watch::Receiver<AppState>. (hep blog)[https://raysuliteanu.medium.com/creating-a-tui-in-rust-e284d31983b3?utm_source=chatgpt.com] 
- [x] Tokio process streaming — stream child stdout/stderr without blocking UI. Rust Users thread has code for capturing println! output. (forum helper)[https://users.rust-lang.org/t/how-to-intercept-stdout-to-display-inside-tui-layout/78823?utm_source=chatgpt.com]
- [ ] kube‑rs Controller — follow the “Application controller” guide to reconcile upgrades. (check `Kube.rs`)[https://kube.rs/controllers/application/?utm_source=chatgpt.com]
- [ ] Spawn & stream: write cmd_exec so kubectl get nodes streams cleanly.
- [ ] Parser TDD: write #[test] cases for the madison parser using captured sample output.
- [ ] State machine: represent every high‑level step as enum Phase; drive it with loop { match phase { … } }.
- [ ] Ratatui basics: build a static layout; then refactor to redraw only when AppState.version_panel changes.
- [ ] Error bubbles: simulate a failed kubectl drain; show step → red, log detail, and exit gracefully.

# 11. Nexts
- [x] make `enum` for node role `worker` or `controller`
- [x] do the UI layout and break parts until getting what needed with boilerplate sentences, that will later be feeded with dynamic data (`core_ui/src/ui.rs`)
- [x] put in the boilerplate rendered `core_ui/src/lib/rs` fields not only plain text but try to populate with initialized `shared_state` `Pipeline` values
- [x] do a `shared_fn` that will do conditional on each steps to analyze line from `while` loop in `engine/src/lib.rs` and update the share state `PipelineState`
- [x] in `core_ui/src/ui.rs` make the Shared State field derived from `PipelineState` field update
- [x] change state logic to in order to be updated in `engine/src/lib.rs`, no need to do it inside the stream, do not overcomplicate.
- [x] add a small snippet in `engine/src/lib.rs` to capture user press of keystroke `q` to quit the app gently.
- [x] before steps logic creation, create a blocking pop-up at the beginning to ask user input desired version of kube (kubelet/kubeadm/kubectl) and containerd
- [x] put tutorial display on the screens about what is expected to enter in the unser input fields with link to the place where they can find compatibility
- [x] add `user input` versions desired when the app launches so that it can run smoothly and store in a new state
- [x] for step `Upgrade Plan` add a special state capture from lines of `upgrade plan` command and store the versions, thenmake a comparison to fail app no good
- [ ] activate next step that now we have the repeatable patterns and do step by step starting with `Pull Repo Key`
- [x] add in each steps `lib.rs` a `command` with `ssh` version of the command to run it on `worker` node so need to check `node_role` for all steps
- [x] do next steps to the end and make sure to check how to get output of ssh command and what is ran from control plane and what is ran using ssh
- [ ] add function that checks if `Kube DNS Proxy version matched` and `draw a last sentence` to say that the `upgrade is done` user can exit with `q`

# 12. State logic updates of shared_state decision
```markdown
Step prints output ─▶ stream_child sends line ─▶ engine receives it ─▶ updates state ─▶ redraws TUI
```
/*
// functions available PipelineState
new(mut self)
update_shared_state_info(&mut self, k: &str, v: &str)
update_shared_state_status(&mut self, status: UpgradeStatus)
update_shared_state_node_type(&mut self, node_role: ClusterNodeType)

// PiplelineState field `buf` available functions
fn new(
    kubeadm_v: String,
    kubelet_v: String,
    kubectl_v: String,
    containerd_v: String,
    node_name: String,
    _node_role: ClusterNodeType,
    upgrade_status: UpgradeStatus
  )

// for NodeDiscoveryInfo available functions
new(node_name: &str)
fn add_node_info(&mut self, node_name: &str, node_type: ClusterNodeType)
*/
/*
// TO PLAN THOSE AT EACH STEP
  "Discover Nodes" (here we put all nodes in state so we can update PipelineState with the right node name (stored in state NodeDiscoveryInfo)),
  "Pull Repo Key" (here we can get the versions of the different components and start displaying the PipelineState fields to the `tui`),
  "Madison Version" (here we might create another state of add it to existing NodeDiscovery to save the future version number that will be used for upgrade)
  "Upgrade Plan" (Here we will upgrade Kubeadm, Kubelet, Kubectl and can display Versions On Worker/OR/Controller node .. maybe some ssh commands if worker...)
  "Upgrade Apply" (here we can update PipelineState (kubeadm) version if Controller),
  "Upgrade Node" (Here we can update PipelineState (kubeadm) version if  Worker),
  "Veryfy Core DNS Proxy" (Here after this is confirmed matching the state targeted version from `Madison Version` step 
                           where we stored in state the upgrade target version, we will update the PipelineSatte status for 'tui to show `Upgraded`')
*/

# Extra Notes

## we have 6 columns and we need the 3rd column version number the biggest and 5th column v.1.<...> using regex and comparing to user input state 1.<...> just as safe validation
We might need here to use: awk command and print $3 and $5 like: `cat test.txt | awk '{print $3,$5}'` with conditions and more bash stuff
or even better so no need to have those written to a file and do it on the fly `sudo apt-cache madison kubeadm | awk '{print $3,$5}'` and then add conditions 
kubeadm | 1.29.15-1.1 | https://pkgs.k8s.io/core:/stable:/v1.29/deb  Packages

## Tokio sequential steps function
```rust
async fn upgrade(mut state: State) -> Result<()> {
    discover_nodes(&mut state).await?;
    pull_repo_key(&mut state).await?;
    // …next step
    Ok(())
}
```

## project decision `threads spawn` or `Tokio asyn/await`
```bash
Sequential steps + want live log → use Tokio |> .await each command
CPU work only                    → std::thread::spawn
```

## Rust `VecDeque`
source: (Rust queue bouble-ended VecDeque)[https://doc.rust-lang.org/alloc/collections/vec_deque/struct.VecDeque.html]
VecDeque is a growable ring buffer, which can be used as a double-ended queue `[<>,<>]` efficiently.
The "default" usage of this type as a queue is to use:
  - push_back to add to the queue,
  - pop_front to remove from the queue
  - extend and append push onto the back in this manner
  - iterating over VecDeque goes front to back


## Rust `tokio::watch::chanel`
(doc tokio crate channel)[https://docs.rs/tokio/latest/tokio/sync/watch/fn.channel.html]
`pub fn channel<T>(init: T) -> (Sender<T>, Receiver<T>)`
eg.:
```rust
use tokio::sync::watch;
use tokio::time::{Duration, sleep};

let (tx, mut rx) = watch::channel("hello");

tokio::spawn(async move {
    // Use the equivalent of a "do-while" loop so the initial value is
    // processed before awaiting the `changed()` future.
    loop {
        println!("{}! ", *rx.borrow_and_update());
        if rx.changed().await.is_err() {
            break;
        }
    }
});

sleep(Duration::from_millis(100)).await;
tx.send("world")?;
```

## Rust `mpsc channel`
`pub fn channel<T>() -> (Sender<T>, Receiver<T>)`
{doc mspc channel}[https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html]
eg.:
```rust
use std::sync::mpsc::channel;
use std::thread;

let (sender, receiver) = channel();

// Spawn off an expensive computation
thread::spawn(move || {
    sender.send(expensive_computation()).unwrap();
});

// Do some useful work for awhile

// Let's see what that answer was
println!("{:?}", receiver.recv().unwrap());
```

## Rust `CrosstermBackend`/`Layout`/`Constraint`...
from original `tui` where `ratutui` is derived from
(doc tui with `CrosstermBackend` included)[https://docs.rs/tui/latest/tui/]
eg.:
```rust
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    Ok(())
}
```
bigegr eg.:
```rust
use std::{io, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders},
    layout::{Layout, Constraint, Direction},
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Block")
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    thread::sleep(Duration::from_millis(5000));

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
```

## Rust `Display` implementation
(display documentation by example)[https://doc.rust-lang.org/rust-by-example/hello/print/print_display.html]
Eg. from the doc:
```rust
// Import (via `use`) the `fmt` module to make it available.
use std::fmt;

// Define a structure for which `fmt::Display` will be implemented. This is
// a tuple struct named `Structure` that contains an `i32`.
struct Structure(i32);

// To use the `{}` marker, the trait `fmt::Display` must be implemented
// manually for the type.
impl fmt::Display for Structure {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", self.0)
    }
}
```
Eg.: we are not gonna use it but for documentation we can see an example closer to our project needs with our own step types
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
//pub enum StepKind { // this is to make it public if want to transport it to a `shared_...` folder
enum StepKind {
  DiscoverNodes,
  PullRepoKey,
  MadisonVersion,
  Cordon,
  Drain,
  UpgradePlan,
  UpgradeApplyCtl,
  UpgradeNode,
  Uncordon,
  RestartServices,
  VerifyCoreDnsProxy,
}
// implementing `Display` for the `enum` `StepKind` and need to import `use std::fmt;`
impl fmt::Display for StepKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let name = match self {
      StepKind::DiscoverNodes => "Discover Nodes",
      StepKind::PullRepoKey => "Pull Repo Key",
      StepKind::MadisonVersion => "Madison Version",
      StepKind::Cordon => "Cordon",
      StepKind::Drain => "Drain",
      StepKind::UpgradePlan => "Upgrade Plan",
      StepKind::UpgradeApplyCtl => "Upgrade Apply CTL",
      StepKind::UpgradeNode => "Upgrade Node",
      StepKind::Uncordon => "Uncordon",
      StepKind::RestartServices => "Restart Services",
      StepKind::VerifyCoreDnsProxy => "Verify Core DNS Proxy",
    };
    write!(f, "{}", name)
  }
}
```

## Rust `VecDeque` method
(documentation VecDeque (use sidebar to see doc of each method, trait..etc..))[https://doc.rust-lang.org/std/collections/struct.VecDeque.html#method.push_back]

Eg.: push an element at the back of the `Deque` [1,2,3,...,n] <- `new_value`
```rust
use std::collections::VecDeque;

let mut buf = VecDeque::new();
buf.push_back(1);
buf.push_back(3);
assert_eq!(3, *buf.back().unwrap());
```

## Shell Command Parsing using `Awk`
I put here some examples so that we can put those in the code and adapt those for our desired output parsing
```bash
# from normal output
k get nodes
NAME                         STATUS     ROLES           AGE    VERSION
controller.creditizens.net   Ready      control-plane   669d   v1.29.15
node1.creditizens.net        NotReady   <none>          669d   v1.29.15
node2.creditizens.net        NotReady   <none>          669d   v1.29.15

# initalizing `v` and `NR` (Record Number)
# printing first column {print $1}
# adding if statement `/^N/`(meaning if starts with `N` to target `NAME` header)
# we get the Record Number +2
k get nodes | awk 'v && NR==n{ print $1 }/^N/{ v=$1;  n=NR+2; print v }  '
NAME
node1.creditizens.net
# but this was just playing from what i found online here is a shorter version that I doung to to the job
# coming fromt he `man awk | grep "N"
NR        current record number in the total input stream.
# get Record Number `2` from the output, so here `print $1` gets the first column and then we go through the rows using `NR`
k get nodes | awk 'NR==2{ print $1 }'
controller.creditizens.net
# print the first row
k get nodes | awk 'NR==1{ print $1 }'
NAME
# print the third row
k get nodes | awk 'NR==3{ print $1 }'
node1.creditizens.net
# and here the fourth row
k get nodes | awk 'NR==4{ print $1 }'
node2.creditizens.net
```
Therefore we might need to loop over to get all nodes names or to get the full output and then split on `/n` and collect in a `Vec`

- OR like this we get a string space separated easier maybe in rust to `Vectorize`:
```bash
nodes=""; for elem in $(kubectl get nodes --no-headers | awk '{print $1}'); do nodes+="$elem "; done; echo "$nodes" | xargs
Outputs:
controller.creditizens.net node1.creditizens.net node2.creditizens.net
```

## Rust Splitting on any whitespace and at the same time any return line to parse `.split_whitepsace`
Eg: my test from (rust playground)[https://play.rust-lang.org/?version=stable&mode=debug&edition=2024]
```rust
#![allow(unused)]

#[derive(Debug)]
enum NodeTypes {
    Controller,
    Worker
}

#[derive(Debug)]
struct Node {
    name: String,
    node_type: NodeTypes,
}

fn check_type(a: &str) -> Result<(), std::io::Error> {
    println!("Original A: {:?}", a);
    let splitted_a_vec: Vec<&str> = a.split_whitespace().collect();
    println!("splitted vec of A: {:?}", splitted_a_vec);
    for elem in splitted_a_vec.iter() {
       println!("Inside Loop elem checked: {:?}", elem);
        if *elem == "controller" {
            let node: Node = Node {
                name: elem.to_string(),
                node_type: NodeTypes::Controller,
            };
            println!("{:?}", node)
        } else if *elem == "worker" {
             let node: Node = Node {
                name: elem.to_string(),
                node_type: NodeTypes::Worker,
            };
            println!("{:?}", node)
        } else {
            println!("Elem: {:?} - Is Not Controller Nor Worker", elem);
        }
    }
    Ok(())
}


fn main() {
  let a: &str = "junko controller\nand worker 109";
  //let v = 
  check_type(a); 
  //println!("{:?}", v)
}
Outputs:
Original A: "junko controller\nand worker 109"
splitted vec of A: ["junko", "controller", "and", "worker", "109"]
Inside Loop elem checked: "junko"
Elem: "junko" - Is Not Controller Nor Worker
Inside Loop elem checked: "controller"
Node { name: "controller", node_type: Controller }
Inside Loop elem checked: "and"
Elem: "and" - Is Not Controller Nor Worker
Inside Loop elem checked: "worker"
Node { name: "worker", node_type: Worker }
Inside Loop elem checked: "109"
Elem: "109" - Is Not Controller Nor Worker
```

**so we can use this kind of logic to parse what we need keep along the way to change some part of the display information
or to have some steps being able to get information from a common state where all informations are shared between steps**

## Rust `Format` output from `HashMap` (for `Shared_state` to `tui`)
we might need to have a function to format some of those output to be painted in two lines, see eg.:
```rust
use std::collections::HashMap;

fn main() {
    let a = HashMap::from([("location", "Harajuku"),("Building", "109")]);
    let b = a.into_iter().map(|(k, v)| {
        let d = format!("{}-{}\n", k.to_string(), v.to_string());
        d
    }).collect::<String>();
    println!("{b}");
}
Outputs;
location-Harajuku
Building-109
```

## Rust Test Our State Update:
```rust
fn main() {
  let a = "controller";
  let mut b = NodeDiscoveryInfo::new(a);
  NodeDiscoveryInfo::add_node_info(&mut b, "junko", ClusterNodeType::Worker);

  println!("{:?}", b);
  
  ()
}
Outputs:
NodeDiscoveryInfo { buf: {"junko": Worker, "controller": Undefined} }
```

## Rust Ratatui Styling & Key bindings
Source: (Doc `Ratatui` Paragraph Used As Example)[https://docs.rs/ratatui/latest/ratatui/widgets/struct.Paragraph.html]
Source: (Doc `Ratatui` where we find some key bindings)[https://ratatui.rs/examples/widgets/list/#_top]
We will reuse code that we have used when running our `test/learning` of `ratatui` and bind some `keys` to some actions `events`
```rust
fn run(term: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<(), AppError> {
  loop {
    // UI interactions and render every frame `f` `mut Frame`
    term.draw(|f| ui(f, app))?;

    match event::read()? {
      // matching on the event of pressing `up` (`k`)
      Event::Key(k) => match k.code {
        // here using `vim-alike` command for nomal directions and quit
        KeyCode::Char('q')                 => return Err(AppError::Exit),
        KeyCode::Up | KeyCode::Char('k')   => app.scroll_up(1),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(1),
        KeyCode::PageUp                    => app.scroll_up(10),
        KeyCode::PageDown                  => app.scroll_down(10),
        _ => {}
      },
      _ => {}
    }
  }
}

let res = run(&mut terminal, &mut app);

// Very close to what we have to draw in the UI, but we have one more `App` State as argument called `PipelineState`
// so here i out `redraw_ui` which is using `draw_ui` main `ui.rs` function which is the same
pub fn redraw_ui<B: Backend>(term: &mut Terminal<B>, s: &AppState, s_s: &PipelineState) -> anyhow::Result<()> {
    term.draw(|f| draw_ui(f, s, s_s))?;
    Ok(())
}
```


```rust
PipelineState:
 
  color (StepColor:
    Grey, Green, Blue, Red
  ),

  log (SharedState: 
    buff(Hashmap(keys:
      kubeadm_v,
      kubelet_v,
      kubectl_v,
      containterd_v,
      node_name,
      node_role(ClusterNodeType:
        Controller, Worker, Undefined
      ),
      upgrade_status(UpgradeStatus:
        Upgraded, InProcess, Waiting, Error
      ),
    )
  ),
let log_kubeadm_v = shared_state.log.buff.kubeadm_v; // type should be already `String` as it is stored like that
let log = Paragraph::new(log_kubeadm_v).block(Block::default().title("Log").borders(Borders::ALL));
f.render_widget(log, <area>[area_index]);
```

## Rust `Ratatui` widgets
We might need some more visual effects like a `spinner` for example so that user waits, see doc:
source: (Spinner widget `ratatui`)[https://ratatui.rs/showcase/third-party-widgets/]


## Rust Iter over `&mut Vec<String>` and match values to get rid of those (`No pop()`)
So here there is two ways of doing that:
- using `retain` that keeps all excepts the value
- using `position` to get index and `remove` that takes the index to remove value.

```rust
#[derive(Debug, Clone)]
struct Test {
  d: Vec<String>,
  e: Vec<String>,
}


fn main() {
  let b = &mut Test { 
    d: Vec::from(["naha".to_string(), "kobe".to_string(), "Tokyo".to_string()]), 
    e: Vec::from(["Tokyo".to_string()]),
  };
  for elem in b.e.iter() {
      if b.d.contains(elem) {
          // first way using `.retain(||)` closure
          //b.d.retain(|x| x != elem);
          // second way using `Option` `if let` and `position(||)` closure
          //if let Some(pos) = b.d.iter().position(|x| x == elem) {
              //b.d.remove(pos);
          //}
      }
  }
  println!("{:?}", b);
  ()
}
Outputs:
Test { d: ["naha", "kobe"], e: ["Tokyo"] }
```

# Rust `|` is for boolean not for `OR`, and what to use for `OR`
so do not use `|` or `||` for comparison to check if something is matching, like `"a" | "b" | "c" == var`
use it only for `bool` `true/false`

How to check in `Rust` comparing and having this `OR`:
- use `matched!()` macro
- use `.contains()`
```rust
// `.matches!()`
if matches!(var, "a" | "b" | "c") { do something }
// `.contains()`
let my_selection = ["a", "b", "c"];
if var.contains(&my_selection) { do something }
```

# Get Version `bash`
We will use those in the step `Pull Repository Key` or even from the beginning when `tui` starts so that we can see the version of the different components.
and update in `engine/src/lib.rs` when receiving lines in the correst step using a special function or `crate` for that
Eg: cheching output for each so that we can plan how to parse what we need to show in `TUI`, we will try to parse from the `cmd` passed in the stream directly
to get an easy line reception when the `stream` `thread` `spawned` message is received.
```bash
# kubeadm
kubeadm version
Outputs:
kubeadm version: &version.Info{Major:"1", Minor:"29", GitVersion:"v1.29.15", GitCommit:"0d0f172cdf9fd42d6feee3467374b58d3e168df0", GitTreeState:"clean", BuildDate:"2025-03-11T17:46:36Z", GoVersion:"go1.23.6", Compiler:"gc", Platform:"linux/amd64"}
# kubectl
kubectl version
Outputs:
Client Version: v1.29.15
Kustomize Version: v5.0.4-0.20230601165947-6ce0bf390ce3
Server Version: v1.29.15
# kubelet
kubelet --version
Outputs:
Kubernetes v1.29.15
# containerd
containerd --version
Outputs:
containerd containerd.io 1.7.25 bcc810d6b9066471b0b6fa75f557a15a1cbf31bb
```
- therefore, here to get the versions of different kubernetes components we have the commands:
**Kubeadm**
```bash
# kubeadm: macth on `line.contains("kubeadm")`
kubeadm version | awk '{split($0,a,"\""); print a[6]}' | awk -F "[v]" '{ print $1 $NF}'
outputs:
1.29.15
```

**Kubectl**
```bash
# kubectl: macth on `line.contains("Client")`
kubectl version | awk 'NR==1{ print $3}' | awk -F "[v]" '{ print $1 $NF}'
Outputs:
1.29.15
```

**Kubelet**
```bash
# kubelet: macth on `line.contains("Kubernetes")`
kubelet --version | awk '{ print $2}' | awk -F "[v]" '{ print $1 $NF}'
Outputs:
1.29.15
```

**Containerd**
```bash
# containerd: macth on `line.contains("containerd")`
containerd --version | awk '{ print $3 }'
Outputs:
1.7.25
```

## `Git` commmand to see all previous `commit` and their **full** `commit message`
Try any of those, the last one is with some formatting but same same but the same
```bash
git log --oneline --decorate --graph --all
git log --pretty=full
git log --pretty=fuller
git log --pretty=format:"%C(auto)%h %d%nAuthor: %an <%ae>%nDate: %ad%n%n%s%n%b%n------------------------" --date=short
```

## `ssh` commands to remote server for when we will need to do steps in the remote server
- beforehands need to setup ssh connection so that the app runs smoothly and don't get password asked, so port open + ssh connection validated.
  Admin user should be able to connect just using username@server (thereofore `server` need to be mapped in `/etc/hosts` to the `ip` of the `worker node`)
```bash
ssh creditizens@node1 'bash -c command'
ssh creditizens@node1 'bash -c commad && other_command'
ssh creditizens@node1 'command; other_command; some_more_commands'
```

## Prevent command to ask for user `password` by allowing for that user in a certain scope
Here we are trying to solve the issue that the app can face while running commands and being prompted for password.
Like for `ssh` which need to be setup on the server beforehands, here we need beforehands to have the user password accepting non-interactive command
so that it is not prompted, there are different ways but we could use the one that selects for which binary it can be accepted,
so keeping a kind of least priviledged sor that user only and for those binaries only like `apt`/`apt-get`/`kubeadm`/`kubectl`
```bash
# we need to update the `sudo` configs
sudo visudo

# then use the method here to select the user + to restrict this effect only to those binaries path (add the line at the end)
<admin username> ALL=(ALL) NOPASSWD: /usr/bin/apt, /usr/bin/apt-get, /bin/sh, /bin/bash
# this method here is to allow for all binaries, straight forward but less secure
<admin username> ALL=(ALL) NOPASSWD: ALL

# then run the command in the script using `-n` option for non-interactive
sudo -n apt update`
```
**Note**: use `%<user group> if it is for a group instead of a single user`

## `Madison` command to get the latest version available
Here we will use the state `DesiredVersion` and get the minor version by parsing and formatting user saved version and then we will
inject it to this command which will fetch all lines of the output having that same version number and the first line which will be the latest
```bash
# `$0` prints the full line and the `grep` is targeting the version and the `NR==1` get the first row
sudo apt-cache madison kubectl | grep "1:29" | awk 'NR==1{print $0}'
```

## Rust command affinement for `apt`
To avoid any warning when using `apt` command which is a wrapper and avoid the `warning` which print to our `error` leg in the `cmd.rs` match pattern,
we can use instead `apt-get` and `apt-cache` which are more gently and will do the job without the warning as it is not recommended to use `apt` in scripts.

## Output of `Upgrade Plan` to study to validate step with line fetching
We need to actually check that the version when doing upgrade plan is same as the user desired version proof of correct version installation
and madison fetching.
```bash
sudo kubeadm upgrade plan
Outputs:
[upgrade/config] Making sure the configuration is correct:
[upgrade/config] Reading configuration from the cluster...
[upgrade/config] FYI: You can look at this config file with 'kubectl -n kube-system get cm kubeadm-config -o yaml'
[preflight] Running pre-flight checks.
[upgrade] Running cluster health checks
[upgrade] Fetching available versions to upgrade to
[upgrade/versions] Cluster version: v1.29.15  # so here probably cretae a state just for this step to check version difference, split on "v"
[upgrade/versions] kubeadm version: v1.29.15  # so here same as above and also check same as desired version
I0514 03:23:23.386070  293426 version.go:256] remote version is much newer: v1.33.0; falling back to: stable-1.29
[upgrade/versions] Target version: v1.29.15
[upgrade/versions] Latest version in the v1.29 series: v1.29.15
```

Real line to format as app adds in its sentence prefix from the `cmd.rs`'s `stream_child` function,
strategy of splitting on `v` will do the job and `if line.contains("[upgrade/versions] Cluster version: v")`:
```bash
[Upgrade Plan][OUT] [upgrade/versions] Cluster version: v1.29.15
[Upgrade Plan][OUT] [upgrade/versions] kubeadm version: v1.29.15
```
so version of cluster has to be != to version of line `[upgrade/versions] kubeadm version` and split on `v` get index `[1]`
and if different we check that it "version of line `[upgrade/versions] kubeadm version`" is equal to user `desired version`
here will stop the app steps raising an error if not or if any of those conditions fail.

**NOTE:**
**As we are testing we can't fail when it is different as we are using same version as the one already installed to make sure all run smoothly so**
**what we are going to do is to just check if the line output `Target version` is the same as desired version, and we will split on it fully  until the `v`.**
**It is enough to make the app fail returning an `Err` `anyhow::anyhow!`**


## Upgrade Apply for `Contoller` analysis output
we need to get the lines saying `SUCCESS` and have an `OR`ed with `Enjoy!` also with `successful`:
  - those are all in same line normally and we can check on the version which is also present "v1.29.15"
```bash
sudo kubeadm upgrade apply v1.29.15 --yes
[upgrade/config] Making sure the configuration is correct:
[upgrade/config] Reading configuration from the cluster...
[upgrade/config] FYI: You can look at this config file with 'kubectl -n kube-system get cm kubeadm-config -o yaml'
[preflight] Running pre-flight checks.
[upgrade] Running cluster health checks
[upgrade/version] You have chosen to change the cluster version to "v1.29.15"
[upgrade/versions] Cluster version: v1.29.15
[upgrade/versions] kubeadm version: v1.29.15
[upgrade/prepull] Pulling images required for setting up a Kubernetes cluster
[upgrade/prepull] This might take a minute or two, depending on the speed of your internet connection
[upgrade/prepull] You can also perform this action in beforehand using 'kubeadm config images pull'
[upgrade/apply] Upgrading your Static Pod-hosted control plane to version "v1.29.15" (timeout: 5m0s)...
[upgrade/etcd] Upgrading to TLS for etcd
[upgrade/staticpods] Preparing for "etcd" upgrade
[upgrade/staticpods] Current and new manifests of etcd are equal, skipping upgrade
[upgrade/etcd] Waiting for etcd to become available
[upgrade/staticpods] Writing new Static Pod manifests to "/etc/kubernetes/tmp/kubeadm-upgraded-manifests85768002"
[upgrade/staticpods] Preparing for "kube-apiserver" upgrade
[upgrade/staticpods] Renewing apiserver certificate
[upgrade/staticpods] Renewing apiserver-kubelet-client certificate
[upgrade/staticpods] Renewing front-proxy-client certificate
[upgrade/staticpods] Renewing apiserver-etcd-client certificate
[upgrade/staticpods] Moved new manifest to "/etc/kubernetes/manifests/kube-apiserver.yaml" and backed up old manifest to "/etc/kubernetes/tmp/kubeadm-backup-manifests-2025-05-14-17-51-24/kube-apiserver.yaml"
[upgrade/staticpods] Waiting for the kubelet to restart the component
[upgrade/staticpods] This might take a minute or longer depending on the component/version gap (timeout 5m0s)
[apiclient] Found 1 Pods for label selector component=kube-apiserver
[upgrade/staticpods] Component "kube-apiserver" upgraded successfully!
[upgrade/staticpods] Preparing for "kube-controller-manager" upgrade
[upgrade/staticpods] Current and new manifests of kube-controller-manager are equal, skipping upgrade
[upgrade/staticpods] Preparing for "kube-scheduler" upgrade
[upgrade/staticpods] Current and new manifests of kube-scheduler are equal, skipping upgrade
[upload-config] Storing the configuration used in ConfigMap "kubeadm-config" in the "kube-system" Namespace
[kubelet] Creating a ConfigMap "kubelet-config" in namespace kube-system with the configuration for the kubelets in the cluster
[upgrade] Backing up kubelet config file to /etc/kubernetes/tmp/kubeadm-kubelet-config715180320/config.yaml
[kubelet-start] Writing kubelet configuration to file "/var/lib/kubelet/config.yaml"
[kubeconfig] Writing "admin.conf" kubeconfig file
[kubeconfig] Writing "super-admin.conf" kubeconfig file
[bootstrap-token] Configured RBAC rules to allow Node Bootstrap tokens to get nodes
[bootstrap-token] Configured RBAC rules to allow Node Bootstrap tokens to post CSRs in order for nodes to get long term certificate credentials
[bootstrap-token] Configured RBAC rules to allow the csrapprover controller automatically approve CSRs from a Node Bootstrap Token
[bootstrap-token] Configured RBAC rules to allow certificate rotation for all node client certificates in the cluster
[addons] Applied essential addon: CoreDNS
[addons] Applied essential addon: kube-proxy

[upgrade/successful] SUCCESS! Your cluster was upgraded to "v1.29.15". Enjoy!

[upgrade/kubelet] Now that your control plane is upgraded, please proceed with upgrading your kubelets if you haven't already done so.

```

## Rust Check if variable contains substrings `AND`ed
```rust
let substrings = ["a", "version number"];
if substrings.iter().all(|s| variable.contains(s)) {
    // all substrings found
}
```
OR
```rust
if variable.contains("a") && variable.contains("version number") {
    // both substrings are present
}
```

## Upgrade `Worker` through `ssh`
Here you need first to do the manip on the `/etc/sudoers` or `sudo visudo` to not be prompted for password on `worker nodes`
After you can from the app send command and get the output from the controller in the controller's output terminal
```bash
ssh node1.creditizens.net 'sudo kubeadm upgrade node'
Outputs:
[upgrade] Reading configuration from the cluster...
[upgrade] FYI: You can look at this config file with 'kubectl -n kube-system get cm kubeadm-config -o yaml'
[preflight] Running pre-flight checks
[preflight] Skipping prepull. Not a control plane node.
[upgrade] Skipping phase. Not a control plane node.
[upgrade] Backing up kubelet config file to /etc/kubernetes/tmp/kubeadm-kubelet-config1844232366/config.yaml
[kubelet-start] Writing kubelet configuration to file "/var/lib/kubelet/config.yaml"
[upgrade] The configuration for this node was successfully updated!
[upgrade] Now you should go ahead and upgrade the kubelet package using your package manager.
```
We can try to get `successfully` keyword from lines output to validate the upgrade status of the worker node

## LAst Step Analysis Output
```bash
kubectl get daemonset kube-proxy -n kube-system -o=jsonpath='{.spec.template.spec.containers[0].image}'   
Outputs:
registry.k8s.io/kube-proxy:v1.29.15
```
We will use previous command tweeked and get the version formated with keyword `"kubeproxy "`
```bash
kubectl get daemonset kube-proxy -n kube-system -o=jsonpath='{.spec.template.spec.containers[0].image}' | awk '{split($0,a,"v"); print a[2]}' | awk -F "[v]" '{ print "kubeproxy "$2 $NF}'
Outputs:
kubeproxy 1.29.15
```

## Rust `if/else` rules
As i try to make some `if` statements without `else` I get sometimes some compiling errors forcing me to add and `else` statement.
After some search i foudn thsoe rules that explain more why:
```markdown
- If you have an if without an else, the body must always evaluate to unit, no exceptions.
- If you have an if and an else, their blocks must evaluate to the same type.
- If you return the result of an if/else expression from a function,
  the return type of the function must match the type of the expressions in the if and else blocks.
```
