use ratatui::{prelude::*, widgets::*};
use crate::state::{AppState, StepColor};

pub fn draw_ui(f: &mut Frame, state: &AppState) {
  let rects = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(1),     // header
      Constraint::Min(1),        // body (split → sidebar + log)
      Constraint::Length(1),     // footer
    ])
    // instead of `.size()` which is deprecated in `ratatui` use `.area()`
    .split(f.area());

  // header
  f.render_widget(Paragraph::new("Rust K8s Upgrade – demo"), rects[0]);

  // body -> split
  let body = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Length(20),     // sidebar width
      Constraint::Min(1),
    ])
    .split(rects[1]);

  // sidebar – list steps with colour
  let items: Vec<ListItem> = state.steps.iter().map(|s| {
    let style = match s.color {
      StepColor::Grey  => Style::default().fg(Color::DarkGray),
      StepColor::Green => Style::default().fg(Color::Green),
      StepColor::Blue  => Style::default().fg(Color::Blue),
    };
    ListItem::new(s.name).style(style)
  }).collect();
  let sidebar = List::new(items).block(Block::default().title("Steps").borders(Borders::ALL));
  f.render_widget(sidebar, body[0]);

  // log pane
  let log_text = state.log.iter().cloned().collect::<Vec<_>>().join("\n");
  let log = Paragraph::new(log_text).block(Block::default().title("Log").borders(Borders::ALL));
  f.render_widget(log, body[1]);

  // footer
  f.render_widget(Paragraph::new("q: quit"), rects[2]);
}
