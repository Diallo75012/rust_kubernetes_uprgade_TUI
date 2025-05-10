// core_ui/src/cmd.rs
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt,BufReader};
use tokio::sync::mpsc::Sender;
use std::time::Duration;
use tokio::time::timeout;
use shared_fn::write_debug_steps::write_step_cmd_debug;


/// Streams stdout and stderr of a spawned command, line-by-line, and sends to TUI log channel
/// Also returns early if the command exceeds the timeout limit
pub async fn stream_child(
    step: &'static str,
    mut child: tokio::process::Child,
    tx: Sender<String>,
  ) -> Result<()> {
  // Take the child's stdout and stderr handles
  let stdout = child.stdout.take().context("Missing stdout")?;
  let stderr = child.stderr.take().context("Missing stderr")?;

  // Set up buffered line readers. type is `Result<Option<String>>`
  // `.lines()` extension need the import `AsyncBufReadExt` from `tokio::io`
  let mut rdr_out = BufReader::new(stdout).lines();
  let mut rdr_err = BufReader::new(stderr).lines();
  let tx_clone = tx.clone();

  // Spawn a task that reads stdout/stderr in background and sends to channel
  let log_task = tokio::spawn(async move {
    loop {
      // `tokio::select!` handles the `await` so no need `line = rdr_out.next_line().await` but just `line = rdr_out.next_line()`
      tokio::select! {
        // `.next_lines()` extension need the import `AsyncBufReadExt` from `tokio::io`
        line = rdr_out.next_line() => {
          match line {
            Ok(Some(l)) => {
              // so here even if inside `tokio:;select!` globally, it is not consider as so but inside `match`
              // so `.send()` returns a `Future` therefore need an `await` (tricky). inner nested scope will have their own rules
              let _ = tx_clone.send(format!("[{}][OUT] {}\n", step, l)).await;
              write_step_cmd_debug(&format!("[{}][OUT] {}", step, l));
            },
            Ok(None) => break, // end of stream
            Err(e) => {
              let _ = tx_clone.send(format!("[{}][ERR] error reading stdout: {}", step, e)).await;
              break;
            }
          }
        }
        line = rdr_err.next_line() => {
          match line {
            Ok(Some(l)) => {
              let _ = tx_clone.send(format!("[{}][ERR] {}", step, l)).await;
            }
            Ok(None) => break,
            Err(e) => {
              let _ = tx_clone.send(format!("[{}][ERR] error reading stderr: {}", step, e)).await;
              break;
            }
          }
        }
      }
    }
  });

  // Wait for the process to finish with a timeout
  let status = timeout(Duration::from_secs(10), child.wait())
    .await
    .context(format!("Timeout waiting for step `{}`", step))??;

  if !status.success() {
    return Err(anyhow::anyhow!("Command exited with status: {}", status));
  }

  // Wait for the log task to complete
  log_task.await?;

  Ok(())
}
