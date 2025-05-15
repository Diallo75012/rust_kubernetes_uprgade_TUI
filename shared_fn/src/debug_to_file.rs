use anyhow:: Result;   // <- anyhow::Result = Result<_, anyhow::Error>
use std::io::Write;

pub fn print_debug_log_file(file_path: &str, status_or_message: &str, value_to_write: &str) -> Result<()> {
  // need to have `use std::io::write;` imported otherwise not gonna workuuu
  let mut file = std::fs::OpenOptions::new()
    .append(true)
    .create(true)
    .open(file_path)?;
  writeln!(file, "\nmessage: {}\nstep started: {}", status_or_message, value_to_write)?;
  Ok(())
 }
