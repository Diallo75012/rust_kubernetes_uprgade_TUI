use std::collections::VecDeque;
use tokio::sync::watch;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepColor { Grey, Green, Blue, Red }

#[derive(Debug, Clone)]
pub struct StepInfo {
  pub name: &'static str,
  pub color: StepColor,
}

// keep last N log lines, drop oldest automatically
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
  // `VecDeque` is like `[now, next]`, eg. if only 2 inside: new push to replace `next` which become `now`
  buf: VecDeque<T>,
  cap: usize,
}
impl<T> RingBuffer<T> {
  pub fn new(cap: usize) -> Self { Self { buf: VecDeque::with_capacity(cap), cap } }
  pub fn push(&mut self, v: T) {
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
      log: RingBuffer::new(5000),
    };
    let (tx, rx) = watch::channel(state.clone());
    //(Self { steps, log: RingBuffer::new(5000) }, tx, rx)
    (state, tx, rx)
  }
}
