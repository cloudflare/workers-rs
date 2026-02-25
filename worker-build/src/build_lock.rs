//! Build lock using a `.tmp` staging directory with mtime-based heartbeat.
//!
//! When multiple `worker-build` processes run concurrently (e.g. wrangler
//! triggers a rebuild while the previous one is still running), they can
//! stomp on each other's files in `build/`. This module provides a lock
//! mechanism:
//!
//! 1. All build output goes into `build/.tmp/` instead of `build/` directly.
//! 2. A background thread bumps `.tmp`'s mtime every second as a heartbeat.
//! 3. On startup, if `.tmp` exists with mtime < 5s ago, we know another build
//!    is active and wait for it to go stale before proceeding.
//! 4. On completion, entries are moved from `.tmp/` into `build/`, and `.tmp/`
//!    is removed.

use anyhow::{bail, Context, Result};
use filetime::FileTime;
use log::warn;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

/// How often the heartbeat thread touches `.tmp` (seconds).
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);

/// If `.tmp` mtime is older than this, it's considered stale / abandoned.
const STALE_THRESHOLD: Duration = Duration::from_secs(5);

/// Maximum time to wait for a concurrent build before giving up.
const MAX_WAIT: Duration = Duration::from_secs(300);

pub struct BuildLock {
    /// The real output directory, e.g. `<crate>/build`.
    out_dir: PathBuf,
    /// The staging directory: `<crate>/build/.tmp`.
    tmp_dir: PathBuf,
    /// Signal to stop the heartbeat thread.
    stop: Arc<AtomicBool>,
    /// Join handle for the heartbeat thread.
    heartbeat_handle: Option<thread::JoinHandle<()>>,
}

impl BuildLock {
    /// Acquire the build lock. This will:
    /// - Wait for any active concurrent build to finish (stale `.tmp`)
    /// - Clean up any stale `.tmp` into `.oldtmp` and remove it
    /// - Create a fresh `.tmp` directory
    /// - Start the heartbeat thread
    ///
    /// Returns the path to the `.tmp` staging directory for building into.
    pub fn acquire(out_dir: &Path) -> Result<Self> {
        let tmp_dir = out_dir.join(".tmp");
        let oldtmp_dir = out_dir.join(".oldtmp");

        // Ensure the parent out_dir exists
        fs::create_dir_all(out_dir)?;

        // Wait for any active build to finish
        Self::wait_for_stale(&tmp_dir)?;

        // Clean up stale .tmp → .oldtmp → delete
        if tmp_dir.exists() {
            // Move to .oldtmp so new .tmp can be created immediately
            if oldtmp_dir.exists() {
                if let Err(e) = fs::remove_dir_all(&oldtmp_dir) {
                    warn!("Failed to remove old .oldtmp directory: {e}");
                }
            }
            fs::rename(&tmp_dir, &oldtmp_dir)
                .or_else(|_| fs::remove_dir_all(&tmp_dir))
                .context("Failed to clean up stale .tmp directory")?;
            if let Err(e) = fs::remove_dir_all(&oldtmp_dir) {
                warn!("Failed to remove .oldtmp directory: {e}");
            }
        }

        // Create fresh .tmp
        fs::create_dir_all(&tmp_dir)?;

        // Start heartbeat
        let stop = Arc::new(AtomicBool::new(false));
        let heartbeat_handle = {
            let stop = Arc::clone(&stop);
            let tmp_dir = tmp_dir.clone();
            thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    let now = FileTime::now();
                    if let Err(e) = filetime::set_file_mtime(&tmp_dir, now) {
                        warn!("Failed to update heartbeat mtime: {e}");
                    }
                    thread::sleep(HEARTBEAT_INTERVAL);
                }
            })
        };

        Ok(Self {
            out_dir: out_dir.to_path_buf(),
            tmp_dir,
            stop,
            heartbeat_handle: Some(heartbeat_handle),
        })
    }

    /// Returns the staging directory path to build into.
    pub fn staging_dir(&self) -> &Path {
        &self.tmp_dir
    }

    /// Finish the build: stop heartbeat, move entries from `.tmp/` into the
    /// parent `build/` directory, and remove `.tmp/`.
    pub fn finish(mut self) -> Result<()> {
        // Stop heartbeat
        self.stop_heartbeat();

        // Move each entry from .tmp/ into out_dir/
        for entry in fs::read_dir(&self.tmp_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let dest = self.out_dir.join(&name);

            // Remove existing entry at destination
            if dest.is_dir() {
                if let Err(e) = fs::remove_dir_all(&dest) {
                    warn!(
                        "Failed to remove existing directory {}: {e}",
                        dest.display()
                    );
                }
            } else if dest.exists() {
                if let Err(e) = fs::remove_file(&dest) {
                    warn!("Failed to remove existing file {}: {e}", dest.display());
                }
            }

            fs::rename(entry.path(), &dest).with_context(|| {
                format!(
                    "Failed to move {} from staging to output",
                    name.to_string_lossy()
                )
            })?;
        }

        // Remove the now-empty .tmp directory
        if let Err(e) = fs::remove_dir(&self.tmp_dir) {
            warn!("Failed to remove .tmp staging directory: {e}");
        }

        Ok(())
    }

    /// Wait until `.tmp` either doesn't exist or is stale (mtime > threshold).
    fn wait_for_stale(tmp_dir: &Path) -> Result<()> {
        let deadline = SystemTime::now() + MAX_WAIT;
        loop {
            if !tmp_dir.exists() {
                return Ok(());
            }

            let metadata = match fs::metadata(tmp_dir) {
                Ok(m) => m,
                Err(_) => return Ok(()), // disappeared
            };

            let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let age = SystemTime::now()
                .duration_since(mtime)
                .unwrap_or(Duration::ZERO);

            if age >= STALE_THRESHOLD {
                return Ok(());
            }

            if SystemTime::now() >= deadline {
                bail!(
                    "Timed out waiting for concurrent worker-build to finish \
                     (build/.tmp is still being updated). If this is stale, \
                     remove the build/.tmp directory manually."
                );
            }

            thread::sleep(Duration::from_millis(500));
        }
    }

    fn stop_heartbeat(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.heartbeat_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for BuildLock {
    fn drop(&mut self) {
        // Ensure heartbeat is stopped even on error/panic
        self.stop_heartbeat();
    }
}
