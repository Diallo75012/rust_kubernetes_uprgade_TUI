mod error;
mod state;
mod cli;

use error::AppError;
use state::App;

use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
  execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
// use ratatui::Terminal;  // part of `use ratatui::prelude::*;`
// all explicit import instead of `*` for `ratatui`
//use ratatui::{
//    prelude::{Frame, Terminal, CrosstermBackend, Alignment, Constraint, Direction, Layout, Rect},
//    style::{Color, Modifier, Style},
//    widgets::Paragraph,
//};

#[allow(unused_imports)]
use std::{env, fs, io::{self, Stdout}, path::PathBuf};

use cli::get_path_from_cli;


fn main() -> Result<(), AppError> {
  // 1. load a file the user passed (or fallback to self‑code)
  // `?` is a deterministic error propagation if `Ok(x)` return `x`
  // otherwise `Erre(e)` returns `e` and as the function is returning an `AppError` it is valid wya of handling production grade code error
  let path = get_path_from_cli()?;
  // so here we use `&path` to look cleaner but it is cheaper to use directly `path` as we don't need it after and avoid cloning with `to_owned`
  let lines = fs::read_to_string(&path)?
    // iterator fo `&str` and splits on `\n` or `\r\n`
    .lines()
    // each &str -> String (cloned data) breaking `&str` lifetime tie
    // or `|s| s.to_owned()` so `lines` still exists
    .map(str::to_owned)
    .collect::<Vec<String>>();

   /*
   // example of the cheaper what of doing it but we will not use
   let buffer = fs::read_to_string(path)?;            // path is *moved*, no &
   let mut app = {
     let lines: Vec<&str> = buffer.lines().collect(); // borrow slices
     App::new(lines)                                  // needs App to store Vec<&'a str>
   };                                                 // buffer lives as long as app
   */

  // we put the `lines` in the `App` and `offset` starts at `0` by default in fn `new()` impl of struct `App`
  let mut app = App::new(lines);

  // 2. set up terminal
  // we prepare the `stdout` to be managed by `crosstermBackend::new()`
  let mut stdout = io::stdout();
  // we start the process interactive `TUI`
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  crossterm::terminal::enable_raw_mode()?;
  // we load the `stdout` in `CrosstermBackend`
  let backend = CrosstermBackend::new(stdout);
  // we load the `backend` holding on the `stdout` in the `Terminal` to get terminal output, type `nut term`: `nut Terminal<CrosstermBackend<Stdout>>`
  // `Terminal is from `ratatui` constructor: `use ratatui::Terminal;` or `use ratatui::prelude::*;` (the one we using to get all)
  let mut terminal = Terminal::new(backend)?;

  // 3. run loop
  // looping over `&mut app` type `Vec<String>` through the terminal `mut Terminal<CrosstermBackend<Stdout>>` using their `&` (borrow)
  let res = run(&mut terminal, &mut app);

  // 4. restore tty
  // we stop the process
  crossterm::terminal::disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;

  res
}


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



// DISPLAYS TEXT AND OUTPUT OF FILE LINE STORED IN `app`(struct App) (which have `state` `lines` and `offset`) TO TUI
fn ui(f: &mut Frame, app: &App) {
    // chunks split buffer in header/body/footer so len(3) (0, 1, and 2)
    // each `chunks[n]` will be type `Rect (4 u16s)` so as it implements `Copy` no need to use `&chunk[0]` for example
    // the value will be copied and is cheap to pass it by value
    // Rect is a struct from `Ratatui` that stores `x, y, width, height` as four `u16` values
    // it marks the area on screen where a widget should render
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // header
            Constraint::Min(1),      // body
            Constraint::Length(1),   // footer
        ])
        // split the buffer Frame `f`
        //.split(f.size()); // warning on compiler suggest to use `.area()`
        .split(f.area());

    // header layout formatting for static header no scroll
    let header = Paragraph::new(
      format!(
        "File viewer – lines {}‑{}",
        app.offset + 1,
        //(app.offset + f.size().height as usize).min(app.lines.len())))
        (app.offset + f.area().height as usize).min(app.lines.len())))
      .style(Style::default().add_modifier(Modifier::BOLD))
      .alignment(Alignment::Center);
    // rendering this `header` frame `chunk[0]` of `chunks`
    f.render_widget(header, chunks[0]);

    // body : dont forget that `app` is from `App<Vec<String>>`
    let visible = app.lines.iter()
        .skip(app.offset)
        .take(chunks[1].height as usize)
        // `.cloned()` is a trick to: &String → String
        // but skip the clone and use instead `.map(|s| s.as_str())`       
        .cloned()
        // Vec<String> can be used but here we learn a way to infer:
        // makes a Vec<T> where T is whatever the iterator is yielding... here for sure a `String` type as we used `.cloned()`
        .collect::<Vec<_>>()
        // join needs an owned collection of Strings or an iterator of &str
        .join("\n");
    // `body``
    let body = Paragraph::new(visible);
    // render frame for `chunk[1]` in `body`
    f.render_widget(body, chunks[1]);

    // footer
    let footer = Paragraph::new("↑/k ↓/j PgUp PgDn  q: quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    // render frame `chunk[2]` for footer
    f.render_widget(footer, chunks[2]);
}
