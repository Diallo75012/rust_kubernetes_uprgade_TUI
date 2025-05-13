use crossterm::event::{self, Event, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;
use crate::state::{
  AppState,
  PipelineState,
  StepColor,
  //ClusterNodeType,
  //UpgradeStatus,
  //NodeUpdateTrackerState,
  DesiredVersions,
};


pub fn draw_ui(f: &mut Frame, state: &mut AppState, shared_state: &mut PipelineState) {
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

  let log_upgrade_status = shared_state.log.clone().shared_state_iter("upgrade_status")[0].clone();

  f.render_widget(Paragraph::new("Rust K8s Upgrade – Creditizens - v0.1.0"), header[1]);
  // so here will probably need to get the value from the `PipelineState` and inject to &str
  // f.render_widget(Paragraph::new("Upgrade State<...>"), header[3]);
  f.render_widget(Paragraph::new(log_upgrade_status).style(Style::default().fg(Color::Green)), header[3]);

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
  //let log_text = state.log.iter().cloned().collect::<Vec<_>>().join("\n");
  let log_area_height = body[1].height.saturating_sub(2) as usize; // minus borders

  let log_lines: Vec<_> = state.log.iter().cloned().collect();
  let log_view = log_lines
    .iter()
    .skip(state.log_scroll_offset)
    .take(log_area_height)
    .cloned()
    .collect::<Vec<String>>()
    .join("\n");

  // styling the text output
  let log = Paragraph::new(log_view).block(Block::default().title("Log").borders(Borders::ALL));
  // paints the commands output to the TUI (now we can see it)
  f.render_widget(log, body[1]);

  let footer = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Length(3), // footer[0] space margin from left eadge
      Constraint::Length(18), // footer[1]
      Constraint::Length(25),  // footer[2] to be cut for each version to fit on their own space
      Constraint::Length(25),
      Constraint::Length(25),
      Constraint::Min(1),
      Constraint::Length(30) // footer[6]
    ])
    .split(rects[2]);
  f.render_widget(Paragraph::new("q: quit"), footer[1]);
  /************ Here have a conditional that checks if the state version updated is equal to desired state add effect to paint in green  **************/
  let log_kubeadm_v = shared_state.log.clone().shared_state_iter("kubeadm_v")[0].clone();
  let log_kubeadm = Paragraph::new(format!("Kubeadm v{}", log_kubeadm_v));
  let log_kubelet_v = shared_state.log.clone().shared_state_iter("kubelet_v")[0].clone();
  let log_kubelet = Paragraph::new(format!("Kubelet v{}", log_kubelet_v));
  let log_kubectl_v = shared_state.log.clone().shared_state_iter("kubectl_v")[0].clone();
  let log_kubectl = Paragraph::new(format!("Kubectl v{}", log_kubectl_v));
  let log_containerd_v = shared_state.log.clone().shared_state_iter("containerd_v")[0].clone();
  let log_containerd = Paragraph::new(format!("Containerd v{}", log_containerd_v));
  f.render_widget(log_kubeadm, footer[2]);
  f.render_widget(log_kubelet, footer[3]);
  f.render_widget(log_kubectl, footer[4]);
  f.render_widget(log_containerd, footer[5]);
  // we are not gonna do here the logic of tracking `nodeupdatetrackerstate` but just use the field of `shared_state` and pull those info
  // in the `shared_fn` specifc to tracker `node update tracker state` we will there update `PipelineState` (shared_state).
  // so logic stays in its module, here we just paint to the `tui`
  let log_node_name = shared_state.log.clone().shared_state_iter("node_name")[0].clone();
  let log_node_role = shared_state.log.clone().shared_state_iter("node_role")[0].clone();
  let node_processed_info = List::new(
    Vec::from(
      [
        ListItem::new(log_node_name).style(Style::default().fg(Color::Green)),
        ListItem::new(log_node_role).style(Style::default().fg(Color::Green)),
      ]
    )
  ).block(Block::default());
  // f.render_widget(Paragraph::new("Node name:<...>\nNode role:<...>"), footer[6]);
  f.render_widget(node_processed_info, footer[6]);

}

// function to redraw the UI : This is a more reusable version using `generics`
// as `ratatui` `Backend` accepts `CrosstermBackend` and `TermionBackend` (if want to change backend for example)
pub fn redraw_ui<B: Backend>(term: &mut Terminal<B>, s: &mut AppState, s_s: &mut PipelineState) -> anyhow::Result<()> {
  term.draw(|f| draw_ui(f, s, s_s))?;
  Ok(())
}


/* This is for the Pop-up Taht Captured User desired versions of kube components and containerd*/

// Draws the actual popup window
pub fn draw_version_prompt(f: &mut Frame, input: &str, is_containerd: bool) {

  let title = if is_containerd {
    "Enter Containerd Version"
  } else {
    "Enter Kubernetes Version"
  };

  let block = Block::default()
    .title(title)
    .borders(Borders::ALL);

  let paragraph = Paragraph::new(input.to_string())
    .block(block)
    .style(Style::default().fg(Color::Cyan));

  let rects = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(20),
      Constraint::Min(1),
      Constraint::Length(20),
    ])
    .split(f.area());

  let input_area = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Length(60),
      Constraint::Min(1),
      Constraint::Length(60)
    ])
    .split(rects[1]);

  // place the cursor
  let cursor_x = input_area[1].x + input.len() as u16 + 1; // +1 for padding
  let cursor_y = input_area[1].y + 1; // +1 if your box has a border
  f.set_cursor(cursor_x, cursor_y); // top-left corner (x. y)

  if is_containerd {
    let tutorial = "Please see the right version of Containerd,if needed at: https://containerd.io/releases/.\n(You might not even need to upgrade it, then indicate the version that you have now. It is mandatory to provide a version number.)".to_string();
    let tutorial_text = Paragraph::new(tutorial)
      .block(Block::default().title("Extra Info").borders(Borders::ALL))
      .style(Style::default().fg(Color::Green));
    f.render_widget(tutorial_text, rects[0]);
  } else {
  	let tutorial = "Please get more information about Kubernetes Version Upgrade Here: https://kubernetes.io/docs/tasks/administer-cluster/kubeadm/kubeadm-upgrade/\n(Rule of Thumb is Only One Minor Version At a Time 1.2x.xx , Eg.: if you have 1.28.xx, next upgrade can only be 1.29.xx)\n\n INDICATE YOUR VERSION LIKE THAT, Eg: 1.29 or 1.30 or 1.31 (ONLY MINOR FOR KUBERNETES KUBe COMPONENTS)\n\nThe App will search for the latest statble version available in that range 1.xx".to_string();
  	let tutorial_text = Paragraph::new(tutorial)
      .block(Block::default().title("Extra Info").borders(Borders::ALL))
  	  .style(Style::default().fg(Color::Green));
  	  f.render_widget(tutorial_text, rects[0]);
  }

  f.render_widget(paragraph, input_area[1]);
}

// Captures keyboard input and stores it into `input`
pub fn run_input_prompt<B: Backend>(
  term: &mut Terminal<B>,
  desired_versions_state: &mut DesiredVersions,
  is_containerd: bool,
  // mith ned to ad an input pramater for the state initialized in engine/src/lib/rs that will store the `DesiredVersions`
) -> anyhow::Result<()> {

  /******************************************************
  This function might need to set a variable as buffer input to get to accumulate user keystroke by adding char and deleting
  but when enter is pressed it would be saved to the state

  ********************************************************/

  let mut input_buffer = "".to_string();

  loop {
  
    term.draw(|f| {
      draw_version_prompt(f, &input_buffer, is_containerd); // so here `input` would use the buffer that we would have created here
    })?;

    if event::poll(std::time::Duration::from_millis(500))? {
      if let Event::Key(key) = event::read()? {
        match key.code {
        // so here to have an input_buffer var that will pop characters or push those... and then save it as String to state
          KeyCode::Char(c) => input_buffer.push(c),
          KeyCode::Enter => {
            let input_not_empty = !input_buffer.is_empty();
            let input_is_not_alphabetic = !input_buffer.chars().all(char::is_alphabetic);
            if input_not_empty && input_is_not_alphabetic {
              if is_containerd {
                desired_versions_state.add("target_containerd_version", &input_buffer);
              } else {
                desired_versions_state.add("target_kube_versions", &input_buffer);
              }
            }
            break;
          },
          KeyCode::Backspace => {
            input_buffer.pop();
          },
          KeyCode::Esc => {
            input_buffer.clear();
            break;
          },
          _ => {}
        }
      }
    }
  }
  Ok(())
}
