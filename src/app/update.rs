//! Checking for a newer release, like the other Craft apps. CncCraft asks GitHub for the latest
//! release of its repository, compares the tag with the running version, and — if a newer one
//! exists — offers a link to the download page in the About window. It never replaces the binary
//! itself (that is platform-specific and risky); it points the person at the release, which is the
//! safe, predictable behaviour.
//!
//! The request is made in a background thread with `ureq` (pure-Rust HTTP + rustls TLS, no system
//! library and no external program), so the UI never blocks. The result arrives over a channel and
//! is polled each frame.

use std::sync::mpsc::Receiver;

/// Where the releases live.
const REPO: &str = "freegroup/CncCraft";

/// What a check has found.
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    /// No check has run yet.
    Idle,
    /// A check is in flight.
    Checking,
    /// The running version is the latest.
    UpToDate,
    /// A newer release exists: its version tag and the page to download it from.
    Newer { version: String, url: String },
    /// The check could not be made (offline, curl missing, GitHub unreachable).
    Failed(String),
}

/// The update checker: its current status and the channel a running check reports back on.
pub struct Update {
    pub status: Status,
    rx: Option<Receiver<Status>>,
}

impl Default for Update {
    fn default() -> Self {
        Update { status: Status::Idle, rx: None }
    }
}

impl Update {
    /// Start a check in the background (does nothing if one is already running). Safe to call at
    /// startup; the result is picked up later by [`Update::poll`].
    pub fn check(&mut self) {
        if self.status == Status::Checking {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.status = Status::Checking;
        self.rx = Some(rx);
        std::thread::Builder::new()
            .name("cnccraft-update".into())
            .spawn(move || {
                let _ = tx.send(fetch());
            })
            .ok();
    }

    /// Pick up a finished check, if any. Returns whether the status changed (so the UI repaints).
    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.rx else { return false };
        match rx.try_recv() {
            Ok(status) => {
                self.status = status;
                self.rx = None;
                true
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => false,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.status = Status::Failed("Prüfung abgebrochen".into());
                self.rx = None;
                true
            }
        }
    }
}

/// Ask GitHub for the latest release and compare it with the running version.
fn fetch() -> Status {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body = match ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "CncCraft")
        .timeout(std::time::Duration::from_secs(15))
        .call()
    {
        Ok(resp) => match resp.into_string() {
            Ok(s) => s,
            Err(_) => return Status::Failed("Antwort nicht lesbar".into()),
        },
        Err(ureq::Error::Status(404, _)) => return Status::Failed("Noch keine Veröffentlichung".into()),
        Err(_) => return Status::Failed("GitHub nicht erreichbar".into()),
    };
    let json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return Status::Failed("Antwort nicht lesbar".into()),
    };
    let tag = json.get("tag_name").and_then(|v| v.as_str()).unwrap_or_default();
    if tag.is_empty() {
        // no release published yet (the API returns a "message" instead)
        return Status::Failed("Noch keine Veröffentlichung".into());
    }
    let url = json.get("html_url").and_then(|v| v.as_str()).map(str::to_string).unwrap_or_else(|| format!("https://github.com/{REPO}/releases"));
    if is_newer(tag, env!("CARGO_PKG_VERSION")) {
        Status::Newer { version: tag.trim_start_matches('v').to_string(), url }
    } else {
        Status::UpToDate
    }
}

/// Whether release tag `latest` is a newer semantic version than `current` (both like "1.2.3",
/// `latest` may have a leading "v"). Non-numeric parts are ignored; missing parts count as 0.
fn is_newer(latest: &str, current: &str) -> bool {
    parts(latest) > parts(current)
}

fn parts(v: &str) -> (u64, u64, u64) {
    let v = v.trim().trim_start_matches('v');
    let mut it = v.split(['.', '-', '+']).map(|p| p.parse::<u64>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("v1.0.0", "0.9.9"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
        assert!(!is_newer("v0.1.0", "0.1.0"), "same version with a v prefix is not newer");
    }
}
