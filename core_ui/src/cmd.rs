use tokio::{io::{AsyncBufReadExt, BufReader}, process::{Command, Child}};
use anyhow::Result;
use tokio::sync::mpsc::Sender;

pub async fn stream_child(step: &'static str, mut child: Child, tx: Sender<String>) -> Result<()> {
  let stdout = child.stdout.take().expect("stdout");
  let stderr = child.stderr.take().expect("stderr");

  let mut rdr_out = BufReader::new(stdout).lines();
  let mut rdr_err = BufReader::new(stderr).lines();

  loop {
    tokio::select! {
      line = rdr_out.next_line() => {
        if let Some(l) = line? { tx.send(format!("[{}][OUT] {}", step, l)).await.ok(); }
      }
      line = rdr_err.next_line() => {
        if let Some(l) = line? { tx.send(format!("[{}][ERR] {}", step, l)).await.ok(); }
      }
      else => break,
    }
  }
  child.wait().await?;
  Ok(())
}
