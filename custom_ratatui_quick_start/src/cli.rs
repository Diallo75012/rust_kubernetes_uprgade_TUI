// src/cli.rs
// use crate as we are not in `main.rs so no need `mod error;
use crate::error::AppError; 
use std::{env, path::PathBuf};


pub fn get_path_from_cli() -> Result<PathBuf, AppError> {
  match env::args().nth(1) {
    Some(arg) => Ok(arg.into()),
    None      => Err(AppError::Cli("missing file path".into())),
  }
}
