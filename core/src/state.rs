use std::collections::VecDeque;
use tokio::sync::watch;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepColor { Grey, Green, Blue }

#[derive(Debug)]
pub struct StepInfo {
  pub name: &'static str,
  pub color: StepColor,
}

// keep last N log lines, drop oldest automatically
#[derive(Debug)]
pub struct RingBuffer<T> {
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

#[derive(Debug)]
pub struct AppState {
  pub steps: Vec<StepInfo>,
  pub log: RingBuffer<String>,
}
impl AppState {
  pub fn new(step_names: &[&'static str]) -> (Self, watch::Sender<()>, watch::Receiver<()>) {
    let steps = step_names.iter().map(|&n| StepInfo { name: n, color: StepColor::Grey }).collect();
    let (tx, rx) = watch::channel(());
    (Self { steps, log: RingBuffer::new(5000) }, tx, rx)
  }
}
