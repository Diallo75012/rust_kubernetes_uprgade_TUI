use tokio::sync::watch;
use std::collections::{HashMap, VecDeque};



/* Here to Color `TUI` */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepColor { Grey, Green, Blue, Red }

/* Here State of Upgrade */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpgradeStatus { Upgraded, InProcess, Waiting, Error }

/* Here type of Cluster Node */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClusterNodeType { Controller, Worker, Undefined }

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
      log: RingBuffer::<String>::new(5000), // here is were we default it to `5000`
    };
    let (tx, rx) = watch::channel(state.clone());
    //(Self { steps, log: RingBuffer::new(5000) }, tx, rx)
    (state, tx, rx)
  }
}



/* Here To Manage Shared State Between Stream Steps */
/*
#[derive(Debug, Clone)]
pub struct NodeDiscoveryInfo {
  // k: name, v: ClusterNodeType
  buf: HashMap<String, ClusterNodeType>,
}
impl NodeDiscoveryInfo {
  pub fn new(node_name: &str) -> Self {
    NodeDiscoveryInfo {
  	  buf: HashMap::from(
  	    [
  	      (node_name.to_string(), ClusterNodeType::Undefined),
  	    ]
  	  ),
  	}
  }
  // check here if we return a `Result` and put `Ok()` at the end of function
  pub fn add_node_info(&mut self, node_name: &str, node_type: ClusterNodeType) {
  	self.buf.insert(node_name.to_string(), node_type);
  }
}
*/

#[derive(Debug, Clone)]
pub struct SharedState {
  // just a normal `HashMapuuuuu`
  buf: HashMap<String, String>,
}
impl SharedState {
  pub fn new(
    kubeadm_v: String,
    kubelet_v: String,
    kubectl_v: String,
    containerd_v: String,
    node_name: String,
    _node_role: ClusterNodeType,
    upgrade_status: UpgradeStatus
  ) -> Self { 
    match upgrade_status {
      UpgradeStatus::Waiting => {
        let status = "Waiting for update...".to_string();
        let node_t = "waiting for update...".to_string();
        SharedState {
          buf: HashMap::from(
            [
    	      ("kubeadm_v".to_string(), kubeadm_v),
    	      ("kubelet_v".to_string(), kubelet_v),
    	      ("kubectl_v".to_string(), kubectl_v),
    	      ("containerd_v".to_string(), containerd_v),
    	      ("node_name".to_string(), node_name),
    	      ("node_role".to_string(), node_t),
    	      ("upgrade_status".to_string(), status),
    	    ]
          )
        }
      },
      UpgradeStatus::Upgraded => {
        let status = "Upgraded!".to_string();
        let node_t = "waiting for update...".to_string();
        SharedState {
          buf: HashMap::from(
            [
    	      ("kubeadm_v".to_string(), kubeadm_v),
    	      ("kubelet_v".to_string(), kubelet_v),
    	      ("kubectl_v".to_string(), kubectl_v),
    	      ("containerd_v".to_string(), containerd_v),
    	      ("node_name".to_string(), node_name),
    	      ("node_role".to_string(), node_t),
    	      ("upgrade_status".to_string(), status),
    	    ]
          )
        }
      },
      UpgradeStatus::InProcess => {
        let status = "In Process...".to_string();
        let node_t = "waiting for update...".to_string();
        SharedState {
          buf: HashMap::from(
            [
    	      ("kubeadm_v".to_string(), kubeadm_v),
    	      ("kubelet_v".to_string(), kubelet_v),
    	      ("kubectl_v".to_string(), kubectl_v),
    	      ("containerd_v".to_string(), containerd_v),
    	      ("node_name".to_string(), node_name),
    	      ("node_role".to_string(), node_t),
    	      ("upgrade_status".to_string(), status),
    	    ]
          )
        }
      },
      UpgradeStatus::Error => {
        let status = "Error Call J...".to_string();
        let node_t = "waiting for update...".to_string();
        SharedState {
          buf: HashMap::from(
            [
    	      ("kubeadm_v".to_string(), kubeadm_v),
    	      ("kubelet_v".to_string(), kubelet_v),
    	      ("kubectl_v".to_string(), kubectl_v),
    	      ("containerd_v".to_string(), containerd_v),
    	      ("node_name".to_string(), node_name),
    	      ("node_role".to_string(), node_t),
    	      ("upgrade_status".to_string(), status),
    	    ]
          )
        }
      },
    }
  }

  pub fn shared_state_iter(self, key: &str) -> Vec<String> {
    let mut values = Vec::new();
    for (k , v) in self.buf.iter() {
      if *k == key {
         values.push(v.to_string());
      }
    }
    values
  }
 
}

/// so here we add some shared fields
// and in implementation there are the  functions for those specific shared fields
#[derive(Debug, Clone)]
pub struct PipelineState {
  /* GENERAL ONES */
  pub color: StepColor,
  // we need this to store the state and be able to update `tui` from what is inside `This`
  pub log: SharedState,
}

// here are the functions that will enable the fields of shared state
// to be store in state and to be rendered to the 'tui'
impl PipelineState {
  pub fn new() -> (Self, watch::Sender<PipelineState>, watch::Receiver<PipelineState>) {
    let pipeline_state = Self { color : StepColor::Grey,
      // "Wait for Update...".to_string()
      log : SharedState::new(
        "Wait for Update...".to_string(),
        "Wait for Update...".to_string(),
        "Wait for Update...".to_string(),
        "Wait for Update...".to_string(),
        "Wait for Update...".to_string(),
        ClusterNodeType::Undefined,
        UpgradeStatus::Waiting,
      ),
    };
    let (tx, rx) = watch::channel(pipeline_state.clone());
    (pipeline_state, tx, rx)
  }

  // this will add to the hashmap so we will be able to have the `tui` updated with that when drawing/painting to it
  // this is only for the `String` values updates
  pub fn update_shared_state_info(&mut self, k: &str, v: &str) {
    self.log.buf.insert(k.to_string(), v.to_string());
  }

  // this is only for the `UpgradeStatus` field update
  pub fn update_shared_state_status(&mut self, status: UpgradeStatus) {
    match status {
      UpgradeStatus::Waiting => {
        self.log.buf.insert("upgrade_status".to_string(), "Waiting for update...".to_string()); 	
      },
      UpgradeStatus::Upgraded => {
        self.log.buf.insert("upgrade_status".to_string(), "Upgraded!".to_string()); 
      },
      UpgradeStatus::InProcess => {
        self.log.buf.insert("upgrade_status".to_string(), "Step In Process...".to_string()); 
      },
      UpgradeStatus::Error => {
        self.log.buf.insert("upgrade_status".to_string(), "Error MangaKissa Emergency!...".to_string());       	
      },
    }
  }

  // this is only for the `UpgradeStatus` field update
  pub fn update_shared_state_node_type(&mut self, node_role: ClusterNodeType) {
    match node_role {
      ClusterNodeType::Undefined => {
        self.log.buf.insert("node_role".to_string(), "Undefined...".to_string()); 	
      },
      ClusterNodeType::Controller => {
        self.log.buf.insert("node_role".to_string(), "Controller".to_string()); 
      },
      ClusterNodeType::Worker => {
        self.log.buf.insert("node_role".to_string(), "Worker".to_string()); 
      },
    }
  }
  // ... more functions
}

/* Here will be the state managing the tracking of which nodes discovered have been updated or not to tell which will be next */
#[derive(Debug, Clone)]
struct NodeUpdateTrackerState {
  discovered_node: Vec<String>,
  node_already_updated: Vec<String>,
}
impl NodeUpdateTrackerState {
  pub fn new() -> (Self, watch::Sender<NodeUpdateTrackerState>, watch::Receiver<NodeUpdateTrackerState>) {
    let node_update_state = Self {
      discovered_node: Vec::new(),
      node_already_updated: Vec::new() ,
    };
    let (tx, rx) = watch::channel(node_update_tracker_state.clone());
    (node_update_tracker_state, tx, rx)
  }
}
