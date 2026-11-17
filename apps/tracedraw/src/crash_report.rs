//! CDRCraft: automatic crash reports.
//!
//! A panic already leaves `crash.log` in the settings folder
//! (`startup_log`), and the log of that run is kept as `previous.log`.
//! At the next start, if the user allowed it, both go to a crash-report
//! service that speaks the Sentry protocol (sentry.io or a self-hosted
//! GlitchTip). The report holds the error, the version, the system and
//! the log with the user's home folder hidden; never a drawing.
//!
//! The service's address (a "DSN": public, it can only receive reports)
//! is set when the program is built, with the environment variable
//! `CDRCRAFT_CRASH_DSN`, or at run time with the same variable (tests,
//! a different server). Without one nothing is sent and nothing is asked.
//!
//! The first time there is a report to send, the user is asked once;
//! the answer is kept in the settings (`crash_reports`).

use std::path::{Path, PathBuf};

/// The address set at build time, if any.
const BUILT_DSN: Option<&str> = option_env!("CDRCRAFT_CRASH_DSN");

/// The most of the log sent, from its end (the lines before the crash).
const MAX_LOG: usize = 64 * 1024;

/// The service address in use: the run-time variable, else the built one.
pub fn dsn() -> Option<String> {
    std::env::var("CDRCRAFT_CRASH_DSN")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| BUILT_DSN.map(str::to_string))
        .filter(|s| !s.trim().is_empty())
}

/// A parsed DSN: where envelopes go and the public key.
#[derive(Debug, PartialEq)]
pub struct Target {
    pub envelope_url: String,
    pub key: String,
    pub dsn: String,
}

/// `scheme://key@host[:port][/path]/project` to the envelope endpoint
/// `scheme://host[:port][/path]/api/project/envelope/`.
pub fn parse_dsn(dsn: &str) -> Option<Target> {
    let dsn = dsn.trim();
    let (scheme, rest) = dsn.split_once("://")?;
    if scheme != "https" && scheme != "http" {
        return None;
    }
    let (key, rest) = rest.split_once('@')?;
    let key = key.split(':').next()?.to_string();
    let (host_path, project) = rest.trim_end_matches('/').rsplit_once('/')?;
    if key.is_empty() || host_path.is_empty() || project.is_empty() {
        return None;
    }
    Some(Target {
        envelope_url: format!("{scheme}://{host_path}/api/{project}/envelope/"),
        key,
        dsn: dsn.to_string(),
    })
}

/// The report file a panic leaves in `dir`.
fn pending(dir: &Path) -> PathBuf {
    crate::startup_log::crash_path(dir)
}

/// Where the log of the run that crashed is kept.
pub fn previous_log_path(dir: &Path) -> PathBuf {
    dir.join("previous.log")
}

/// Hide the user's home folder and name in `text`.
pub fn scrub(text: &str, home: Option<&str>, user: Option<&str>) -> String {
    let mut t = text.to_string();
    if let Some(h) = home.filter(|h| h.len() > 1) {
        t = t.replace(h, "~");
    }
    if let Some(u) = user.filter(|u| u.len() > 2) {
        t = t.replace(u, "<usuario>");
    }
    t
}

fn home_and_user() -> (Option<String>, Option<String>) {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok();
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .ok();
    (home, user)
}

fn now_secs() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 32 hex digits, different for every report.
fn event_id() -> String {
    let nanos = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mixed = nanos
        ^ ((std::process::id() as u128) << 64)
        ^ 0x9e37_79b9_7f4a_7c15_f39c_c060_5ced_c834u128;
    format!("{mixed:032x}")
}

/// The Sentry envelope for one crash: header, event item.
pub fn envelope(target: &Target, report: &str, log: &str, id: &str, timestamp: u64) -> String {
    let first = report.lines().next().unwrap_or("crash");
    // "CDRCraft 0.47.0 (linux x86_64) stopped: WHAT at PLACE"
    let what = first
        .split_once("stopped: ")
        .map(|(_, w)| w)
        .unwrap_or(first);
    let log_tail = if log.len() > MAX_LOG {
        let mut start = log.len() - MAX_LOG;
        while !log.is_char_boundary(start) {
            start += 1;
        }
        &log[start..]
    } else {
        log
    };
    let event = serde_json::json!({
        "event_id": id,
        "timestamp": timestamp,
        "platform": "native",
        "level": "fatal",
        "logger": "panic",
        "release": format!("cdrcraft@{}", env!("CARGO_PKG_VERSION")),
        "environment": if cfg!(debug_assertions) { "debug" } else { "release" },
        "message": { "formatted": first },
        "exception": { "values": [ {
            "type": "panic",
            "value": what,
            "mechanism": { "type": "panic", "handled": false },
        } ] },
        "contexts": {
            "os": { "name": std::env::consts::OS },
            "device": { "arch": std::env::consts::ARCH },
        },
        "extra": {
            "report": report,
            "log": log_tail,
        },
    });
    let header = serde_json::json!({ "event_id": id, "dsn": target.dsn });
    let body = event.to_string();
    let item = serde_json::json!({ "type": "event", "length": body.len() });
    format!("{header}\n{item}\n{body}\n")
}

/// Send one envelope; true when the service took it.
fn send(target: &Target, envelope: &str) -> bool {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(15)))
        .http_status_as_error(false)
        .build()
        .into();
    let auth = format!(
        "Sentry sentry_version=7, sentry_key={}, sentry_client=cdrcraft/{}",
        target.key,
        env!("CARGO_PKG_VERSION")
    );
    match agent
        .post(&target.envelope_url)
        .header("X-Sentry-Auth", &auth)
        .header("Content-Type", "application/x-sentry-envelope")
        .send(envelope)
    {
        Ok(r) => {
            let ok = r.status().is_success();
            if !ok {
                log::warn!("crash report: the service answered {}", r.status());
            }
            ok
        }
        Err(e) => {
            log::warn!("crash report not sent: {e}");
            false
        }
    }
}

/// Ask the user once whether reports may be sent.
fn ask() -> bool {
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    {
        let answer = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Info)
            .set_title("CDRCraft")
            .set_description(crate::i18n::tr("dialog.crash_reports_question"))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        matches!(answer, rfd::MessageDialogResult::Yes)
    }
    #[cfg(any(target_os = "android", target_arch = "wasm32"))]
    {
        false
    }
}

/// What a start does with a report left by the last run.
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Nothing,
    Declined,
    Sending,
}

/// At start: if the last run left a report, ask (once) and send it in
/// the background. A report sent, or declined, is kept as
/// `crash-AAAAMMDD-HHMMSS.log` beside the settings and not sent again.
pub fn on_start(settings: &mut crate::settings::Settings) -> Outcome {
    let Some(dir) = crate::settings::config_dir() else {
        return Outcome::Nothing;
    };
    on_start_in(&dir, settings, dsn(), ask)
}

pub fn on_start_in(
    dir: &Path,
    settings: &mut crate::settings::Settings,
    dsn: Option<String>,
    ask: impl FnOnce() -> bool,
) -> Outcome {
    let report_path = pending(dir);
    let Ok(report) = std::fs::read_to_string(&report_path) else {
        return Outcome::Nothing;
    };
    let Some(target) = dsn.as_deref().and_then(parse_dsn) else {
        return Outcome::Nothing;
    };
    let allowed = match settings.crash_reports {
        Some(a) => a,
        None => {
            let a = ask();
            settings.crash_reports = Some(a);
            settings.save();
            a
        }
    };
    let kept = dir.join(format!("crash-{}.log", now_secs()));
    let log = std::fs::read_to_string(previous_log_path(dir)).unwrap_or_default();
    if !allowed {
        let _ = std::fs::rename(&report_path, &kept);
        return Outcome::Declined;
    }
    let (home, user) = home_and_user();
    let report = scrub(&report, home.as_deref(), user.as_deref());
    let log = scrub(&log, home.as_deref(), user.as_deref());
    let body = envelope(&target, &report, &log, &event_id(), now_secs());
    // Taken out of the way first, so a crash while sending does not send
    // it again at every start.
    let _ = std::fs::rename(&report_path, &kept);
    std::thread::spawn(move || {
        if send(&target, &body) {
            log::info!("crash report sent");
        } else {
            // Put it back to try at the next start.
            let _ = std::fs::rename(&kept, &report_path);
        }
    });
    Outcome::Sending
}

/// `CDRCRAFT_TEST_CRASH=1`: stop on purpose a few seconds after the
/// window opens, to check the reports end to end.
pub fn test_crash_requested() -> bool {
    std::env::var("CDRCRAFT_TEST_CRASH").is_ok_and(|v| v == "1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dsn_gives_the_envelope_address_and_key() {
        let t = parse_dsn("https://abc123@o42.ingest.sentry.io/4567").unwrap();
        assert_eq!(t.envelope_url, "https://o42.ingest.sentry.io/api/4567/envelope/");
        assert_eq!(t.key, "abc123");
        let t = parse_dsn("http://k:secret@localhost:8000/glitch/7").unwrap();
        assert_eq!(t.envelope_url, "http://localhost:8000/glitch/api/7/envelope/");
        assert_eq!(t.key, "k");
        assert!(parse_dsn("ftp://k@h/1").is_none());
        assert!(parse_dsn("https://h/1").is_none());
        assert!(parse_dsn("").is_none());
    }

    #[test]
    fn the_home_folder_and_user_name_are_hidden() {
        let s = scrub(
            "open /home/maria/Dibujos/a.cdr by maria",
            Some("/home/maria"),
            Some("maria"),
        );
        assert_eq!(s, "open ~/Dibujos/a.cdr by <usuario>");
    }

    #[test]
    fn the_envelope_has_a_header_an_item_and_the_event() {
        let t = parse_dsn("https://k@h/1").unwrap();
        let env = envelope(
            &t,
            "CDRCraft 1 (linux x86_64) stopped: boom at a.rs:1\nbacktrace",
            &"x".repeat(MAX_LOG + 10),
            "0123456789abcdef0123456789abcdef",
            10,
        );
        let lines: Vec<&str> = env.lines().collect();
        assert_eq!(lines.len(), 3);
        let item: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(item["length"].as_u64().unwrap() as usize, lines[2].len());
        let ev: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(ev["exception"]["values"][0]["value"], "boom at a.rs:1");
        assert_eq!(ev["level"], "fatal");
        assert_eq!(ev["extra"]["log"].as_str().unwrap().len(), MAX_LOG);
    }

    #[test]
    fn without_a_service_or_a_report_nothing_is_asked() {
        let dir = std::env::temp_dir().join(format!("cdrcraft-crash-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let mut s = crate::settings::Settings::default();
        let asked = std::cell::Cell::new(false);
        // No report.
        assert_eq!(
            on_start_in(&dir, &mut s, Some("https://k@h/1".into()), || {
                asked.set(true);
                true
            }),
            Outcome::Nothing
        );
        // A report but no service.
        std::fs::write(pending(&dir), "x").unwrap();
        assert_eq!(on_start_in(&dir, &mut s, None, || { asked.set(true); true }), Outcome::Nothing);
        assert!(!asked.get());
        assert!(pending(&dir).exists());
        // Declined: kept aside, not sent, not asked again.
        s.crash_reports = Some(false);
        assert_eq!(
            on_start_in(&dir, &mut s, Some("https://k@h/1".into()), || panic!("asked")),
            Outcome::Declined
        );
        assert!(!pending(&dir).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
