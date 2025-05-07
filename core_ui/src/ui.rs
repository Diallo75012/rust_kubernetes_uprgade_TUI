use ratatui::prelude::{Frame, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::state::{AppState, PipelineState, StepColor};

pub fn draw_ui(f: &mut Frame, state: &AppState, shared_state: &PipelineState) {
  // using `ratutui` `Layout` grid helper
  let rects = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(6),     // header
      Constraint::Min(1),        // body (will be split later on to -> sidebar + log)
      Constraint::Length(6),     // footer
    ])
    // instead of `.size()` which is deprecated in `ratatui` use `.area()`
    .split(f.area());

  // making the header[0,1] split from the `rects[0]`
  let header = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Min(1),
      Constraint::Min(10),
    ])
    .split(rects[0]);
  f.render_widget(Paragraph::new("Rust K8s Upgrade – Creditizens - v0.1.0"), header[0]);
  // so here will probably need to get the value from the `PipelineState` and inject to &str
  f.render_widget(Paragraph::new("Upgrade State<...>"), header[1]);

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
      Constraint::Length(10), // footer[0]
      Constraint::Min(1),  // footer[1]
      Constraint::Min(20) // footer[2]
    ])
    .split(rects[2]);
  f.render_widget(Paragraph::new("q: quit"), footer[0]);
  // so here will probably need to get the value from the `PipelineState` and inject to &str
  f.render_widget(Paragraph::new("Kubeadm<...>; Kubectl<...>; <Kubelet<...>\nContainerd<..>"), footer[1]);
  f.render_widget(Paragraph::new("Node name:<...>\nNode role:<...>"), footer[2]);
}

// function to redraw the UI : This is a more reusable version using `generics`
// as `ratatui` `Backend` accepts `CrosstermBackend` and `TermionBackend` (if want to change backend for example)
pub fn redraw_ui<B: Backend>(term: &mut Terminal<B>, s: &AppState, s_s: &PipelineState) -> anyhow::Result<()> {
    term.draw(|f| draw_ui(f, s, s_s))?;
    Ok(())
}
