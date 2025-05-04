// state.rs

pub struct App {
  pub lines: Vec<String>,
  // first visible line
  pub offset: usize,
}

impl App {
  pub fn new(lines: Vec<String>) -> Self {
    // create `App` with default `offset` starting at `0`
    Self { lines, offset: 0 }
  }

  // FUNCTIONS THAT CALUCLATED OFSSET WITH NO UNDER-FLOW
  // those scroll with `usize::saturating_sub()` are making sure it is not less `< 0` as `usize` should always be positive
  // will scroll up/down by the `n` offset but never passing `zero` so never passes the final line `index` up/down directions
  // so here the `.min()` is here to not 'over-shoot' the end (pass over)
  // we use `saturating_sub()` to do the logic but exist also its conterpart `saturating_add()`: all are safe way to substract and add
  // so as `usize= 0 - 1` panics for `under-flow` as `usize` can't be negative, we need the safe subsctraction using `saturating_sub()`
  // also be carefull as `usize` will load its largest value everytime `18 446 744 073 709 551 615`, so it need to be limited to its range `[0 … len‑1]`
  // `right hand side = rhs`, `left ahnd side  = lhs`: result = if lhs >= rhs { lhs - rhs } else { 0 }
  // thereofre the `saturating_sub()` will safely return `0` not `negative`(`panic` for `usize`) if  `rhs > lhs`
  pub fn scroll_up(&mut self, n: usize) { self.offset = self.offset.saturating_sub(n); }
  pub fn scroll_down(&mut self, n: usize) { 
    // as we as going down we just need to make sure that last line is `subsctracted`
    // that is why we use `saturatin_sub(1)` on the totla lines length `self.lines.len()``
    let last_line_idx = self.lines.len().saturating_sub(1);
    self.offset = (self.offset + n).min(last_line_idx);
  }
}
