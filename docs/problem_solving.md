**Kubernetes Upgrade TUI Application - Debugging & Resolution Summary**

**Project Context:**
The project is a Rust-based Terminal UI (TUI) manager to automate Kubernetes upgrades across nodes. It captures user inputs for desired versions, pulls repo keys, performs upgrades, and shows live logs with interactive feedback. The app is modular, composed of multiple `Step` implementations, shared state structs (`PipelineState`, `DesiredVersions`, `AppState`, etc.), and an event-driven UI using Ratatui and Crossterm.

---

### Key Problems Encountered & Solutions

#### 1. **User Input Buffer Not Updating**

* **Problem:** Input was overwritten on each keystroke.
* **Root Cause:** `input_buffer` was not used correctly in the drawing function.
* **Fix:** Replaced usage of `input` with `input_buffer` and ensured it was passed into `draw_version_prompt()` on each `term.draw()`.

#### 2. **GRUB Recovery Mode Inaccessibility & `sudo` Lockout**

* **Problem:** Misconfigured `visudo` entry locked out the user from `sudo`, preventing fixing the issue.
* **Fix:** Rebooted in recovery mode, dropped into root shell, remounted with `mount -o remount,rw /`, and corrected the `/etc/sudoers` file.

#### 3. **`kubeadm upgrade apply` Failed Due to NotReady Controller**

* **Problem:** `kubeadm upgrade apply` failed with: `FATAL: there are NotReady control-planes`.
* **Root Cause:** Controller was cordoned (not reachable for plan/apply).
* **Fix:** Skipped `cordon`/`drain` on controller nodes by checking `node_role`. This logic was conditionally applied in each step's `run()`.

#### 4. **Persistent Shared State Access**

* **Problem:** Needed persistent version state (`DesiredVersions`) across steps without constantly passing it.
* **Fix:** Passed `&mut DesiredVersions` to each `Step` instead of relying on global statics. Confirmed persistence by debug-logging state after each update.

#### 5. **`visudo` and NOPASSWD Configuration**

* **Problem:** Commands like `apt`, `tee`, and `curl` still prompted for password.
* **Fix:** Properly configured the `sudoers` entry (no typos, no wildcards, full binary paths). Ensured non-interactive mode with `sudo -n` and `-y`.

#### 6. **Command Timeout Logic**

* **Problem:** Fixed timeout value (10s) caused premature command termination.
* **Fix:** Increased timeout to 60s for long-running steps and considered future implementation of adaptive timeout estimation based on command.

#### 7. **AWK/Format Macro Errors Due to `{{` and `}}`**

* **Problem:** `format!()` clashed with AWK block syntax (`{}` inside strings).
* **Fix:** Used Rust raw strings (`r#"..."#`) and double-braced AWK (`{{ }}`) where needed to escape ambiguity.

#### 8. **Controller Node Not Upgraded While `kubectl get nodes` Shows Ready**

* **Problem:** Manual `kubectl` showed controller was Ready, but `kubeadm upgrade apply` failed.
* **Root Cause:** Short time between restart and upgrade caused `kubelet` to not fully initialize.
* **Fix:** Added `sleep` before upgrade or retry logic. Also monitored logs and `systemctl status kubelet` to validate state.

#### 9. **Command Execution via SSH**

* **Problem:** `ssh node1 'ls ~/home/user'` failed.
* **Fix:** Removed `~` in nested path (`~/home/...` becomes `/home/user/home/...`, which doesn't exist). Used absolute path.

#### 10. **Log Body Scrolling and TUI Exit**

* **Problem:** Terminal log area didn't scroll; app exited too quickly.
* **Fix:** Added `state.log_scroll_offset` logic with keystroke matching for `PageUp/PageDown`, `j/k`. App now waits for `q` to quit.

#### 11. **Madison Parsing Not Persisting to State**

* **Problem:** `madison_parsed_upgrade_apply_version` not printed in logs or used.
* **Fix:** Confirmed presence of value with debug logs. Used correct field key in `.add()` method. Confirmed state update worked by log traces.

#### 12. **Macro Variable Scope Confusion**

* **Problem:** Custom macro used `&str` in place of `String` slices, causing closure mismatch.
* **Fix:** Matched types explicitly in macro inputs. Used `.as_str()` or `String::from()` where needed.

#### 13. **Version Check & Comparison**

* **Problem:** Needed to verify `kubelet`, `kubectl`, and `containerd` were upgraded correctly.
* **Fix:** Used `awk` on CLI output in step logic. Parsed version into major/minor and compared with user input.

#### 14. **Misuse of `return` and Block Expressions in `if` Statements**

* **Problem:** Used `return` inconsistently in blocks.
* **Fix:** Cleaned up logic using early return and `if` pattern. Learned Rust's rule: `return` is optional for final expression, but required if used before.

#### 15. **`This` Interpreted as a Bash Command**

* **Problem:** String starting with `This is ...` was interpreted as a command.
* **Fix:** Prefixed with `echo` to avoid shell interpretation. Explanation: any unquoted `This` is interpreted by Bash as a binary name.

---

### Additional Improvements

* **TUI Responsiveness:** Implemented rotating loading icon during step execution.
* **Multi-line Command Structuring:** Broke commands into named variables for readability and maintainability.
* **Version Match Validation:** Added strict checking in `check_upgrade_plan_version_and_update_shared_state_versions()`.

---

### Recommendations

* Add retry logic for upgrade steps where node readiness is flaky.
* Centralize all version parsing into a `parser.rs` module.
* Future: Implement global step timeout estimator using average step execution time.

---

### Conclusion

The debugging process revealed edge cases with TUI rendering, async step management, and shell integration. Each issue was solved methodically with minimal code changes and logging as the primary observability tool. The app now handles full Kubernetes upgrade orchestration with persistent shared state, visual feedback, and command safety in mind.
