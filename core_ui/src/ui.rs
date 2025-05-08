use ratatui::prelude::{Frame, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::state::{AppState, PipelineState, StepColor, ClusterNodeType, UpgradeStatus};

pub fn draw_ui(f: &mut Frame, state: &AppState, shared_state: &PipelineState) {
  // using `ratutui` `Layout` grid helper
  let rects = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(1),     // header 1 line
      Constraint::Min(1),        // body (will be split later on to -> sidebar + log) the rest of space
      Constraint::Length(2),     // footer 2 lines
    ])
    // instead of `.size()` which is deprecated in `ratatui` use `.area()`
    .split(f.area());

  // making the header[0,1] split from the `rects[0]`
  let header = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Length(3),
      Constraint::Length(17),
      Constraint::Min(1),
      Constraint::Length(30),
    ])
    .split(rects[0]);
  f.render_widget(Paragraph::new("Rust K8s Upgrade – Creditizens - v0.1.0"), header[1]);
  // so here will probably need to get the value from the `PipelineState` and inject to &str
  f.render_widget(Paragraph::new("Upgrade State<...>"), header[3]);

  // Here we splite the `body` in `horizontal direction` body -> split
  let body = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Length(20),     // sidebar width (20 columns)
      Constraint::Min(1),         // Rest of the `body` would take the rest of the space
    ])
    // so this is concerning the second part of `vertical rects` `[0, 1, 2]` therefore `rects[1]`` 
    .split(rects[1]);

  // sidebar – list steps with colour
  // for each steps we map to a color
  let items: Vec<ListItem> = state.steps.iter().map(|s| {
    let style = match s.color {
      // using the `enum` `stepColor` from `state.rs` to style the items
      // using match pattern in the closure iterable mapped to `s` representing each items to check
      StepColor::Grey  => Style::default().fg(Color::DarkGray), // `ratatui` styling
      StepColor::Green => Style::default().fg(Color::Green),
      StepColor::Blue  => Style::default().fg(Color::Blue),
      StepColor::Red  => Style::default().fg(Color::Red),
    };
    // after we are creating a new `ListItem` with the right colors for each
    ListItem::new(s.name).style(style)
    // after here we collect to a `Vec<ListItem>` as defined above so will infer it (no need to `.collect<Vec<ListItem>>()`)
  }).collect();
  // some styling to the list of items presented on the sidebar: `block`, `title`, `borders`
  let sidebar = List::new(items).block(Block::default().title("Steps").borders(Borders::ALL));
  // we put the content of the sidebar in the `body` horizontal split which is located at `body[0]`
  // using `ratatui` `.render_widget()` 'painter' (so actually writing to the layout created TUI)
  f.render_widget(sidebar, body[0]);

  // This is the `body` log pane so center part that will display the commands output
  // `state.log` is a `RingBuffer<VecQue, usize>`
  //`.iter()` will iterate orver `&String` (type inside the defined `VecQue`)
  // `.clone()` will make those `&String` transform to `String` and then collected to `Vec`
  // then `joined` on `&str` `"\n"` to be printable in one block returning at the line for each lines
  let log_text = state.log.iter().cloned().collect::<Vec<_>>().join("\n");
  // styling the text output
  let log = Paragraph::new(log_text).block(Block::default().title("Log").borders(Borders::ALL));
  // paints the commands output to the TUI (now we can see it)
  f.render_widget(log, body[1]);

  let footer = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Length(3), // footer[0] space margin from left eadge
      Constraint::Length(18), // footer[1]
      Constraint::Length(30),  // footer[2] to be cut for each version to fit on their own space
      Constraint::Length(30),
      Constraint::Length(30),
      Constraint::Min(1),
      Constraint::Length(20) // footer[6]
    ])
    .split(rects[2]);
  f.render_widget(Paragraph::new("q: quit"), footer[1]);
  let log_kubeadm_v = shared_state.log.clone().shared_state_iter("kubeadm_v")[0].clone();
  let log_kubeadm = Paragraph::new(log_kubeadm_v);
  let log_kubelet_v = shared_state.log.clone().shared_state_iter("kubelet_v")[0].clone();
  let log_kubelet = Paragraph::new(log_kubelet_v);
  let log_kubectl_v = shared_state.log.clone().shared_state_iter("kubectl_v")[0].clone();
  let log_kubectl = Paragraph::new(log_kubectl_v);
  let log_containerd_v = shared_state.log.clone().shared_state_iter("containerd_v")[0].clone();
  let log_containerd = Paragraph::new(log_containerd_v);
  f.render_widget(log_kubeadm, footer[2]);
  f.render_widget(log_kubelet, footer[3]);
  f.render_widget(log_kubectl, footer[4]);
  f.render_widget(log_containerd, footer[5]);
  f.render_widget(Paragraph::new("Node name:<...>\nNode role:<...>"), footer[6]);
}

// function to redraw the UI : This is a more reusable version using `generics`
// as `ratatui` `Backend` accepts `CrosstermBackend` and `TermionBackend` (if want to change backend for example)
pub fn redraw_ui<B: Backend>(term: &mut Terminal<B>, s: &AppState, s_s: &PipelineState) -> anyhow::Result<()> {
    term.draw(|f| draw_ui(f, s, s_s))?;
    Ok(())
}

//PipelineState: 
//  color (StepColor:
//    Grey, Green, Blue, Red
//  ),
//  log (SharedState: 
//    buff(Hashmap(keys:
//      kubeadm_v,
//      kubelet_v,
//      kubectl_v,
//      containterd_v,
//      node_name,
//      node_role(ClusterNodeType:
//        Controller, Worker, Undefined
//      ),
//      upgrade_status(UpgradeStatus:
//        Upgraded, InProcess, Waiting, Error
//      ),
//    )
//  ),


/*
  let items: Vec<ListItem> = state.steps.iter().map(|s| {
    let style = match s.color {
      // using the `enum` `stepColor` from `state.rs` to style the items
      // using match pattern in the closure iterable mapped to `s` representing each items to check
      StepColor::Grey  => Style::default().fg(Color::DarkGray), // `ratatui` styling
      StepColor::Green => Style::default().fg(Color::Green),
      StepColor::Blue  => Style::default().fg(Color::Blue),
      StepColor::Red  => Style::default().fg(Color::Red),
    };
    // after we are creating a new `ListItem` with the right colors for each
    ListItem::new(s.name).style(style)
    // after here we collect to a `Vec<ListItem>` as defined above so will infer it (no need to `.collect<Vec<ListItem>>()`)
  }).collect();
  // some styling to the list of items presented on the sidebar: `block`, `title`, `borders`
  let sidebar = List::new(items).block(Block::default().title("Steps").borders(Borders::ALL));
  // we put the content of the sidebar in the `body` horizontal split which is located at `body[0]`
  // using `ratatui` `.render_widget()` 'painter' (so actually writing to the layout created TUI)
  f.render_widget(sidebar, body[0]);


let share_state_style = shared_state.color;
let log_shared_state_text: HashMap<String, String> = shared_state.log.buf.into_iter().cloned().collect();

// rects[2].footer[2]
node_role
node_name
// rects[0].header[1]
let upgrade_status = log_shared_state_text.upgrade_
// rects[2].footer[1]
let kubeadm_version = log_shared_state_text.kubeadm_v;
let kubelet_version = log_shared_state_text.kubelet_v;
let kubectl_version = log_shared_state_text.kubectl_v;
let containerd_version = log_shared_state_text.containerd_v;
*/


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
