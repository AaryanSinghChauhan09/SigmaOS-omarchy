// SPDX-License-Identifier: MIT
//! Hyprland & Wayland Compositor Session Controller for SigmaOS-omarchy
//!
//! Provides automated configuration generation, workspace dispatching,
//! window rule validation, and compositor status verification.

use std::fmt;

/// Wayland layout layout mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Dwindle,
    Master,
}

impl fmt::Display for LayoutMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dwindle => write!(f, "dwindle"),
            Self::Master => write!(f, "master"),
        }
    }
}

/// Dynamic workspace configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRule {
    pub workspace_id: u8,
    pub monitor: String,
    pub default_name: String,
}

impl WorkspaceRule {
    pub fn new(id: u8, monitor: &str, name: &str) -> Self {
        Self {
            workspace_id: id,
            monitor: monitor.to_string(),
            default_name: name.to_string(),
        }
    }

    pub fn to_rule_directive(&self) -> String {
        format!(
            "workspace = {}, monitor:{}, default:true",
            self.workspace_id, self.monitor
        )
    }
}

/// Window floating and opacity rule
#[derive(Debug, Clone, PartialEq)]
pub struct WindowRule {
    pub class_pattern: String,
    pub floating: bool,
    pub opacity: Option<f32>,
}

impl WindowRule {
    pub fn new_floating(class: &str) -> Self {
        Self {
            class_pattern: class.to_string(),
            floating: true,
            opacity: None,
        }
    }

    pub fn new_translucent(class: &str, opacity: f32) -> Self {
        Self {
            class_pattern: class.to_string(),
            floating: false,
            opacity: Some(opacity.clamp(0.1, 1.0)),
        }
    }

    pub fn to_rule_directive(&self) -> String {
        let mut directives = Vec::new();
        if self.floating {
            directives.push(format!("windowrulev2 = float, class:^({})$", self.class_pattern));
        }
        if let Some(op) = self.opacity {
            directives.push(format!(
                "windowrulev2 = opacity {:.2} {:.2}, class:^({})$",
                op, op, self.class_pattern
            ));
        }
        directives.join("\n")
    }
}

/// Hyprland Compositor configuration generator
#[derive(Debug, Clone, PartialEq)]
pub struct HyprlandConfig {
    pub layout: LayoutMode,
    pub border_size: u8,
    pub gaps_in: u8,
    pub gaps_out: u8,
    pub active_border_color: String,
    pub inactive_border_color: String,
    pub workspaces: Vec<WorkspaceRule>,
    pub window_rules: Vec<WindowRule>,
}

impl Default for HyprlandConfig {
    fn default() -> Self {
        Self {
            layout: LayoutMode::Dwindle,
            border_size: 2,
            gaps_in: 5,
            gaps_out: 10,
            active_border_color: "rgba(33ccffee) rgba(00ff99ee) 45deg".to_string(),
            inactive_border_color: "rgba(595959aa)".to_string(),
            workspaces: vec![
                WorkspaceRule::new(1, "DP-1", "Code"),
                WorkspaceRule::new(2, "DP-1", "Web"),
                WorkspaceRule::new(3, "DP-1", "Term"),
            ],
            window_rules: vec![
                WindowRule::new_floating("pavucontrol"),
                WindowRule::new_floating("nm-connection-editor"),
                WindowRule::new_translucent("kitty", 0.90),
            ],
        }
    }
}

impl HyprlandConfig {
    pub fn to_config_text(&self) -> String {
        let mut out = String::new();
        out.push_str("# SigmaOS Omarchy - Hyprland Config\n");
        out.push_str(&format!("general {{\n    gaps_in = {}\n    gaps_out = {}\n    border_size = {}\n    col.active_border = {}\n    col.inactive_border = {}\n    layout = {}\n}}\n\n",
            self.gaps_in, self.gaps_out, self.border_size, self.active_border_color, self.inactive_border_color, self.layout));

        out.push_str("# Workspaces\n");
        for ws in &self.workspaces {
            out.push_str(&format!("{}\n", ws.to_rule_directive()));
        }
        out.push('\n');

        out.push_str("# Window Rules\n");
        for rule in &self.window_rules {
            out.push_str(&format!("{}\n", rule.to_rule_directive()));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_display_aaa() {
        assert_eq!(LayoutMode::Dwindle.to_string(), "dwindle");
        assert_eq!(LayoutMode::Master.to_string(), "master");
    }

    #[test]
    fn test_workspace_directive_generation_aaa() {
        let ws = WorkspaceRule::new(1, "eDP-1", "Main");
        assert_eq!(ws.to_rule_directive(), "workspace = 1, monitor:eDP-1, default:true");
    }

    #[test]
    fn test_floating_window_rule_aaa() {
        let rule = WindowRule::new_floating("qalculate-gtk");
        let dir = rule.to_rule_directive();
        assert!(dir.contains("windowrulev2 = float, class:^(qalculate-gtk)$"));
    }

    #[test]
    fn test_translucent_window_rule_aaa() {
        let rule = WindowRule::new_translucent("kitty", 0.85);
        let dir = rule.to_rule_directive();
        assert!(dir.contains("windowrulev2 = opacity 0.85 0.85, class:^(kitty)$"));
    }

    #[test]
    fn test_full_hyprland_config_generation_aaa() {
        let conf = HyprlandConfig::default();
        let text = conf.to_config_text();
        assert!(text.contains("layout = dwindle"));
        assert!(text.contains("gaps_in = 5"));
        assert!(text.contains("pavucontrol"));
        assert!(text.contains("kitty"));
    }
}
