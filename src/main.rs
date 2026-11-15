// SPDX-License-Identifier: MIT
// SigmaOS-omarchy: `omarchy` CLI — doctor, theme, run.
//
// Principles: SINGLE RESPONSIBILITY, FAIL-FAST, DISCOVERABILITY,
// LEAST ASTONISHMENT, COMPOSITION OVER INHERITANCE.

use sigmaos_omarchy::{doctor, render, summarize, SystemFacts};
use std::env;
use std::path::Path;

fn detect_system_facts() -> SystemFacts {
    let wayland_session =
        env::var("WAYLAND_DISPLAY").is_ok()
        || env::var("XDG_SESSION_TYPE").map(|v| v == "wayland").unwrap_or(false);

    let hyprland_running = env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();

    let waybar_running = std::process::Command::new("pgrep")
        .arg("-x").arg("waybar")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let theme_set =
        env::var("SIGMAOS_THEME").is_ok()
        || Path::new(&format!("{}/.config/sigmaos/omarchy/wallust", env::var("HOME").unwrap_or_default())).exists();

    let audio_server = std::process::Command::new("pgrep")
        .arg("-f").arg("pipewire|pulseaudio")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let network_up = Path::new("/sys/class/net").read_dir()
        .map(|mut d| d.any(|e| {
            if let Ok(entry) = e {
                entry.file_name().to_string_lossy() != "lo"
            } else { false }
        }))
        .unwrap_or(false);

    let disk_free_gib = {
        let out = std::process::Command::new("df")
            .arg("--output=avail").arg("-BG").arg("/")
            .output()
            .ok();
        out.and_then(|o| {
            String::from_utf8(o.stdout).ok()
                .and_then(|s| s.lines().nth(1).and_then(|l| l.trim().trim_end_matches('G').parse::<u64>().ok()))
        }).unwrap_or(0)
    };

    let disk_total_gib = {
        let out = std::process::Command::new("df")
            .arg("--output=size").arg("-BG").arg("/")
            .output()
            .ok();
        out.and_then(|o| {
            String::from_utf8(o.stdout).ok()
                .and_then(|s| s.lines().nth(1).and_then(|l| l.trim().trim_end_matches('G').parse::<u64>().ok()))
        }).unwrap_or(1)
    };

    let updates_pending = std::process::Command::new("checkupdates")
        .output()
        .map(|o| o.stdout.iter().filter(|&&b| b == b'\n').count() as u32)
        .unwrap_or(0);

    let secure_boot = {
        let out = std::process::Command::new("mokutil")
            .arg("--sb-state").output().ok();
        out.map(|o| String::from_utf8(o.stdout).ok()
            .map(|s| s.contains("SecureBoot enabled")).unwrap_or(false))
            .unwrap_or(false)
    };

    let battery_pct = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok());

    SystemFacts {
        wayland_session,
        hyprland_running,
        waybar_running,
        theme_set,
        audio_server,
        network_up,
        disk_free_gib,
        disk_total_gib,
        updates_pending,
        secure_boot,
        battery_pct,
    }
}

fn cmd_doctor() {
    println!("\x1b[1;35m╔══════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[1;35m║             SIGMAOS · OMARCHY DOCTOR                     ║\x1b[0m");
    println!("\x1b[1;35m║        Diagnostic Health & Readiness Inspection          ║\x1b[0m");
    println!("\x1b[1;35m╚══════════════════════════════════════════════════════════╝\x1b[0m\n");

    let facts = detect_system_facts();
    let results = doctor(&facts);
    let report = render(&results);
    print!("{}", report);

    let summary = summarize(&results);
    if summary.fail > 0 {
        std::process::exit(1);
    }
}

fn cmd_theme(args: &[String]) {
    println!("\x1b[1;36m── Omarchy Theme Engine ────────────────────────────────────\x1b[0m");
    if args.is_empty() || args[0] == "list" {
        let themes = ["catppuccin-mocha", "nord", "gruvbox", "tokyo-night", "dracula", "rose-pine", "everforest"];
        for t in &themes {
            println!("  {}", t);
        }
    } else if args[0] == "set" && args.len() >= 2 {
        let theme = &args[1];
        println!("Setting theme: {}", theme);
        env::set_var("SIGMAOS_THEME", theme);
        println!("\x1b[32m[✓] Theme '{}' applied.\x1b[0m", theme);
        println!("    Restart Waybar to apply panel colours.");
    } else {
        eprintln!("Usage: omarchy theme [list | set <name>]");
        std::process::exit(2);
    }
}

fn cmd_run() {
    let omarchy_script = "/home/aaryansinghchauhan/SigmaOS/scripts/launch_omarchy_persona.sh";
    let sigmaos_bin = "/home/aaryansinghchauhan/SigmaOS/bin/sigmaos";

    if Path::new(omarchy_script).exists() {
        let status = std::process::Command::new("bash")
            .arg(omarchy_script)
            .status()
            .expect("failed to exec omarchy launcher");
        std::process::exit(status.code().unwrap_or(0));
    } else if Path::new(sigmaos_bin).exists() {
        let status = std::process::Command::new(sigmaos_bin)
            .arg("--omarchy")
            .status()
            .expect("failed to exec sigmaos");
        std::process::exit(status.code().unwrap_or(0));
    } else {
        eprintln!("SigmaOS not found. Clone from: https://github.com/AaryanSinghChauhan09/SigmaOS");
        std::process::exit(1);
    }
}

fn print_help() {
    println!("SigmaOS Omarchy CLI — Aesthetic OS Management\n");
    println!("Usage: omarchy <command> [args]\n");
    println!("Commands:");
    println!("  doctor             System health diagnostics (Wayland, audio, disk, updates)");
    println!("  theme list         List available themes");
    println!("  theme set <name>   Apply a named colour theme");
    println!("  run                Launch Omarchy Linux aesthetic persona");
    println!("  help               Show this help\n");
    println!("Examples:");
    println!("  cargo run -- doctor");
    println!("  cargo run -- theme list");
    println!("  cargo run -- theme set catppuccin-mocha");
    println!("  cargo run -- run");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
        "doctor" => cmd_doctor(),
        "theme" => cmd_theme(&args[1..].to_vec()),
        "run" => cmd_run(),
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("Unknown command: '{}'. Run 'omarchy help' for usage.", other);
            std::process::exit(2);
        }
    }
}
