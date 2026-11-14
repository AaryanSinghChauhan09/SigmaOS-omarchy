use sigmaos_omarchy::*;

#[test]
fn component_constructs_and_runs_doctor() {
    let f = SystemFacts {
        wayland_session: true, hyprland_running: true, waybar_running: true,
        theme_set: true, audio_server: true, network_up: true,
        disk_free_gib: 100, disk_total_gib: 500, updates_pending: 0,
        secure_boot: false, battery_pct: None,
    };
    let results = doctor(&f);
    assert!(!results.is_empty());
}
