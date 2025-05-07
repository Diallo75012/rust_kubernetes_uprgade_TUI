use std::collections::VecDeque;
use tokio::sync::watch;
use std::collections::HashMap;


/* Here to Color `TUI` */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepColor { Grey, Green, Blue, Red }

/* Here State of Upgrade */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpgradeStatus { Upgraded, InProcess, Waiting, Error }

#[derive(Debug, Clone)]
pub struct StepInfo {
  pub name: &'static str,
  pub color: StepColor,
}


/* Here To Manage `App` State */
// keep last N log lines, drop oldest automatically
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
  // `VecDeque` is like `[now, next]`, eg. if only 2 inside: new push to replace `next` which become `now`
  buf: VecDeque<T>, // defaulted to `5000`
  // we make sure it has fix size
  cap: usize,
}
impl<T> RingBuffer<T> {
  pub fn new(cap: usize) -> Self { Self { buf: VecDeque::with_capacity(cap), cap } }
  pub fn push(&mut self, v: T) {
    // we make sure it keeps it length and just swap by getting rid of the front `.pop_front()` and pushing one new value at the back `.push_back()`
    if self.buf.len() == self.cap { self.buf.pop_front(); }
    self.buf.push_back(v);
  }
  pub fn iter(&self) -> impl Iterator<Item=&T> { self.buf.iter() }
}

#[derive(Debug, Clone)]
pub struct AppState {
  pub steps: Vec<StepInfo>,
  pub log: RingBuffer<String>,
}
impl AppState {
  pub fn new(step_names: &[&'static str]) -> (Self, watch::Sender<AppState>, watch::Receiver<AppState>) {
    //let steps = step_names.iter().map(|&n| StepInfo { name: n, color: StepColor::Grey }).collect();
    let state = AppState {
      // this will be the `Vec<StepInfo>`
      steps: step_names.iter().map(|&step_name| StepInfo {
        name: step_name,
        color: StepColor::Grey,
      }).collect(),
      // this will be the `RingBuffer<String>` limits the buffer if the output is too long
      log: RingBuffer::new(5000), // here is were we default it to `5000`
    };
    let (tx, rx) = watch::channel(state.clone());
    //(Self { steps, log: RingBuffer::new(5000) }, tx, rx)
    (state, tx, rx)
  }
}

/* Here To Manage Shared State Between Stream Steps */

/// so here we add some shared fields
// and in implementation there are the  functions for those specific shared fields
#[derive(Debug, Clone)]
pub struct PipelineState {
  /* GENERAL ONES */
  pub color: StepColor,
  // we need this to store the state and be able to update `tui` from what is inside `This`
  pub log: RingBuffer<String>,
  // name and kind of the nodes
  // {name:role} (controller/worker)
  pub node_roles: HashMap<String, String>,
  // Upgrade status state
  pub upgrade_status: UpgradeStatus,
  // versions
  pub kubeadm_version: String,
  pub kubelet_version: String,
  pub kubectl_version: String,
  pub containerd_version: String,
}

// here are the functions that will enable the fields of shared state
// to be store in state and to be rendered to the 'tui'
impl PipelineState {
  pub fn new() -> (Self, watch::Sender<PipelineState>, watch::Receiver<PipelineState>) {
    let state_pipeline = PipelineState {
      color: StepColor::Grey,
      log: RingBuffer::new(5000),
      // rects[2].footer[2]
      node_roles: HashMap::from([("waiting for Node name...".to_string(), "Role will be updated...".to_string())]),
      // rects[0].header[1]
      upgrade_status: UpgradeStatus::Waiting,
      // rects[2].footer[1]
      kubeadm_version: "Waiting For Update...".to_string(),
      kubelet_version: "Waiting For Update...".to_string(),
      kubectl_version: "Waiting For Update...".to_string(),
      containerd_version: "Waiting For Update...".to_string(),
    };
    let (tx, rx) = watch::channel(state_pipeline.clone());
    (state_pipeline, tx, rx)
  }

  // this will add to the hashmap so we will be able to have the `tui` updated with that when drawing/painting to it
  pub fn add_node(&mut self, name: &str, role: &str) {
    self.node_roles.insert(name.to_string(), role.to_string());
  }

  // ... more functions
}
