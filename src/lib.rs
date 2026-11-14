//! SigmaOS-omarchy: top-level Omarchy tool — `omarchy doctor` health
//! checks and setup-wizard state.
//!
//! Real implementation: named diagnostic checks (session, compositor,
//! panel, theme, audio, network, disk, updates) each producing
//! pass/warn/fail with remediation hints, plus a first-boot wizard
//! state machine. Replaces the generic `Tool` placeholder.

use std::fmt;

/// Severity of a check result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity { Pass, Warn, Fail }

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self { Self::Pass => "PASS", Self::Warn => "WARN", Self::Fail => "FAIL" }
    }
}

/// One diagnostic check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub name: &'static str,
    pub severity: Severity,
    pub detail: String,
    pub fix: Option<&'static str>,
}

impl CheckResult {
    fn pass(name: &'static str, detail: impl Into<String>) -> Self {
        Self { name, severity: Severity::Pass, detail: detail.into(), fix: None }
    }
    fn warn(name: &'static str, detail: impl Into<String>, fix: &'static str) -> Self {
        Self { name, severity: Severity::Warn, detail: detail.into(), fix: Some(fix) }
    }
    fn fail(name: &'static str, detail: impl Into<String>, fix: &'static str) -> Self {
        Self { name, severity: Severity::Fail, detail: detail.into(), fix: Some(fix) }
    }
}

/// System facts the checks inspect — injected so tests are deterministic.
#[derive(Debug, Clone, Default)]
pub struct SystemFacts {
    pub wayland_session: bool,
    pub hyprland_running: bool,
    pub waybar_running: bool,
    pub theme_set: bool,
    pub audio_server: bool,
    pub network_up: bool,
    pub disk_free_gib: u64,
    pub disk_total_gib: u64,
    pub updates_pending: u32,
    pub secure_boot: bool,
    pub battery_pct: Option<u8>,
}

/// Run all doctor checks against observed facts.
pub fn doctor(f: &SystemFacts) -> Vec<CheckResult> {
    let mut out = vec![];

    // session/compositor
    out.push(if f.wayland_session {
        CheckResult::pass("wayland-session", "running under Wayland")
    } else {
        CheckResult::fail("wayland-session", "not a Wayland session", "log in via Hyprland session")
    });
    out.push(if f.hyprland_running {
        CheckResult::pass("hyprland", "compositor running")
    } else {
        CheckResult::fail("hyprland", "compositor not detected", "start Hyprland or check logs")
    });
    out.push(if f.waybar_running {
        CheckResult::pass("waybar", "panel running")
    } else {
        CheckResult::warn("waybar", "panel not running", "omarchy-refresh or restart waybar")
    });

    // desktop state
    out.push(if f.theme_set {
        CheckResult::pass("theme", "theme configured")
    } else {
        CheckResult::warn("theme", "no theme selected", "omarchy theme set <name>")
    });

    // hardware/services
    out.push(if f.audio_server {
        CheckResult::pass("audio", "PipeWire/PulseAudio running")
    } else {
        CheckResult::warn("audio", "no audio server detected", "systemctl --user start pipewire")
    });
    out.push(if f.network_up {
        CheckResult::pass("network", "link up")
    } else {
        CheckResult::warn("network", "no network connectivity", "check NetworkManager")
    });

    // disk
    let used_pct = if f.disk_total_gib > 0 {
        100 - (f.disk_free_gib * 100 / f.disk_total_gib)
    } else { 0 };
    out.push(if used_pct < 85 {
        CheckResult::pass("disk", format!("{} GiB free of {}", f.disk_free_gib, f.disk_total_gib))
    } else if used_pct < 95 {
        CheckResult::warn("disk", format!("{}% full", used_pct), "clean package cache")
    } else {
        CheckResult::fail("disk", format!("{}% full", used_pct), "free space immediately")
    });

    // updates
    out.push(if f.updates_pending == 0 {
        CheckResult::pass("updates", "system up to date")
    } else {
        CheckResult::warn("updates", format!("{} packages pending", f.updates_pending),
            "omarchy update")
    });

    // security
    out.push(if f.secure_boot {
        CheckResult::pass("secure-boot", "enabled")
    } else {
        CheckResult::warn("secure-boot", "disabled", "enable in firmware for measured boot")
    });

    // battery (only if present)
    if let Some(pct) = f.battery_pct {
        out.push(if pct > 20 {
            CheckResult::pass("battery", format!("{}%", pct))
        } else {
            CheckResult::warn("battery", format!("{}% — low", pct), "plug in charger")
        });
    }

    out
}

/// Doctor summary: counts by severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DoctorSummary { pub pass: usize, pub warn: usize, pub fail: usize }

pub fn summarize(results: &[CheckResult]) -> DoctorSummary {
    let mut s = DoctorSummary::default();
    for r in results {
        match r.severity {
            Severity::Pass => s.pass += 1,
            Severity::Warn => s.warn += 1,
            Severity::Fail => s.fail += 1,
        }
    }
    s
}

/// Render doctor output like `omarchy doctor`.
pub fn render(results: &[CheckResult]) -> String {
    let mut out = String::new();
    for r in results {
        let mark = match r.severity {
            Severity::Pass => "[ ok ]", Severity::Warn => "[warn]", Severity::Fail => "[FAIL]",
        };
        out.push_str(&format!("{} {:<16} {}\n", mark, r.name, r.detail));
        if let Some(fix) = r.fix {
            out.push_str(&format!("       fix: {}\n", fix));
        }
    }
    let s = summarize(results);
    out.push_str(&format!("\n{} passed, {} warnings, {} failures\n", s.pass, s.warn, s.fail));
    out
}

/// First-boot setup wizard steps (Omarchy ships a guided first login).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WizardStep {
    Welcome,
    Locale,
    Keyboard,
    UserAccount,
    Theme,
    Network,
    Done,
}

/// Wizard state machine: linear with back/next.
#[derive(Debug, Clone)]
pub struct Wizard {
    pub step: WizardStep,
    pub answers: Vec<(WizardStep, String)>,
}

impl Wizard {
    pub fn new() -> Self {
        Self { step: WizardStep::Welcome, answers: vec![] }
    }
    pub fn next(&mut self) {
        self.step = match self.step {
            WizardStep::Welcome => WizardStep::Locale,
            WizardStep::Locale => WizardStep::Keyboard,
            WizardStep::Keyboard => WizardStep::UserAccount,
            WizardStep::UserAccount => WizardStep::Theme,
            WizardStep::Theme => WizardStep::Network,
            WizardStep::Network => WizardStep::Done,
            WizardStep::Done => WizardStep::Done,
        };
    }
    pub fn back(&mut self) {
        self.step = match self.step {
            WizardStep::Welcome => WizardStep::Welcome,
            WizardStep::Locale => WizardStep::Welcome,
            WizardStep::Keyboard => WizardStep::Locale,
            WizardStep::UserAccount => WizardStep::Keyboard,
            WizardStep::Theme => WizardStep::UserAccount,
            WizardStep::Network => WizardStep::Theme,
            WizardStep::Done => WizardStep::Network,
        };
    }
    pub fn answer(&mut self, value: impl Into<String>) {
        self.answers.retain(|(s, _)| *s != self.step);
        self.answers.push((self.step, value.into()));
        self.next();
    }
    pub fn finished(&self) -> bool { self.step == WizardStep::Done }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OmarchyError {
    WizardAlreadyDone,
    EmptyAnswer(WizardStep),
}

impl fmt::Display for OmarchyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WizardAlreadyDone => write!(f, "setup wizard already completed"),
            Self::EmptyAnswer(s) => write!(f, "empty answer for {:?}", s),
        }
    }
}

impl std::error::Error for OmarchyError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy() -> SystemFacts {
        SystemFacts {
            wayland_session: true, hyprland_running: true, waybar_running: true,
            theme_set: true, audio_server: true, network_up: true,
            disk_free_gib: 200, disk_total_gib: 500, updates_pending: 0,
            secure_boot: true, battery_pct: Some(80),
        }
    }

    #[test]
    fn test_doctor_all_pass_on_healthy_system_aaa() {
        // Arrange & Act
        let results = doctor(&healthy());
        // Assert
        assert!(results.iter().all(|r| r.severity == Severity::Pass));
        assert_eq!(summarize(&results).fail, 0);
        assert!(results.iter().any(|r| r.name == "hyprland"));
    }

    #[test]
    fn test_doctor_detects_broken_session_aaa() {
        // Arrange
        let mut f = healthy();
        f.hyprland_running = false;
        f.audio_server = false;
        f.disk_free_gib = 10; // 98% full
        // Act
        let results = doctor(&f);
        let s = summarize(&results);
        // Assert
        assert!(results.iter().any(|r| r.name == "hyprland" && r.severity == Severity::Fail));
        assert!(results.iter().any(|r| r.name == "audio" && r.severity == Severity::Warn));
        assert!(results.iter().any(|r| r.name == "disk" && r.severity == Severity::Fail));
        assert!(s.fail >= 2);
        // fixes are present for failures
        assert!(results.iter().filter(|r| r.severity != Severity::Pass)
            .all(|r| r.fix.is_some()));
    }

    #[test]
    fn test_render_output_format_aaa() {
        // Arrange
        let results = doctor(&healthy());
        // Act
        let text = render(&results);
        // Assert
        assert!(text.contains("[ ok ]"));
        assert!(text.contains("passed"));
        assert!(text.contains("hyprland"));
    }

    #[test]
    fn test_disk_threshold_boundaries_aaa() {
        // Arrange & Act & Assert: 85% boundary is warn not fail
        let mut f = healthy();
        f.disk_free_gib = 75; f.disk_total_gib = 500; // 85% used
        let r = doctor(&f).into_iter().find(|r| r.name == "disk").unwrap();
        assert_eq!(r.severity, Severity::Warn);
        f.disk_free_gib = 20; // 96%
        let r = doctor(&f).into_iter().find(|r| r.name == "disk").unwrap();
        assert_eq!(r.severity, Severity::Fail);
    }

    #[test]
    fn test_wizard_walks_all_steps_aaa() {
        // Arrange
        let mut w = Wizard::new();
        // Act: answer through every step
        for _ in 0..6 {
            let step = w.step;
            w.answer(format!("ans-for-{:?}", step));
        }
        // Assert
        assert!(w.finished());
        assert_eq!(w.answers.len(), 6);
        assert!(w.answers.iter().any(|(s, _)| *s == WizardStep::Theme));
    }

    #[test]
    fn test_wizard_back_navigation_aaa() {
        // Arrange
        let mut w = Wizard::new();
        w.next(); w.next(); // at Keyboard
        // Act
        w.back();
        // Assert
        assert_eq!(w.step, WizardStep::Locale);
        w.back(); w.back();
        assert_eq!(w.step, WizardStep::Welcome); // clamps at start
    }

    #[test]
    fn test_battery_low_warns_aaa() {
        // Arrange
        let mut f = healthy();
        f.battery_pct = Some(10);
        // Act
        let r = doctor(&f).into_iter().find(|r| r.name == "battery").unwrap();
        // Assert
        assert_eq!(r.severity, Severity::Warn);
        // desktop (no battery) → no battery check
        f.battery_pct = None;
        assert!(!doctor(&f).iter().any(|r| r.name == "battery"));
    }
}
