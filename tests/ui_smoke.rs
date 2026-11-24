use std::fs;

use adbsight::adb::AdbClient;
use adbsight::adb::demo_backend::DemoAdb;
use adbsight::adb::wireless::QrPairingSession;
use adbsight::app::App;
use adbsight::app::state::{
    Modal, PendingAction, PersonalSubTab, SystemSubTab, Tab, WirelessModalState, WirelessSubTab,
};
use adbsight::models::RebootMode;
use adbsight::storage::config::AppConfig;
use adbsight::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[tokio::test]
async fn renders_all_tabs_and_modals_against_demo_device() {
    let tmp_dir = std::env::temp_dir().join(format!("adbsight_ui_smoke_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();
    let db_path = tmp_dir.join("ui_smoke.db");

    let client = AdbClient::Demo(DemoAdb::new());
    let (mut app, _rx) =
        App::new(client.clone(), AppConfig::default(), db_path, "demo".into()).unwrap();

    // Populate state directly from DemoAdb so every tab renders rich real-shaped data
    let serial = "demo-pixel9pro-wifi".to_string();
    let snap = client.collect_full_snapshot(&serial, true).await.unwrap();
    app.state.devices = client.list_devices().await.unwrap();
    app.state.mdns_services = client.list_mdns_services().await.unwrap();
    app.state.active_serial = Some(serial.clone());
    app.state.root_status = client.check_root(&serial).await.unwrap();
    app.state.use_root = true;
    app.state.dashboard.identity = Some(snap.identity.clone());
    app.state.dashboard.telemetry = Some(snap.telemetry.clone());
    app.state.dashboard.cpu_history.extend([18, 25, 33, 41]);
    app.state.dashboard.rx_history.extend([42, 68, 55, 79]);
    app.state.system.props = snap.props.clone();
    app.state.system.settings = client.fetch_settings(&serial).await.unwrap();
    app.state.system.sensors = client.fetch_sensors(&serial).await.unwrap();
    app.state.system.dumpsys_list = client.fetch_dumpsys_list(&serial).await.unwrap();
    app.state.system.dumpsys_output = client
        .fetch_dumpsys_service(&serial, "battery", true)
        .await
        .unwrap();
    app.state.processes.processes = snap.processes.clone();
    app.state.processes.services = client.fetch_services(&serial, true).await.unwrap();
    app.state.packages.packages = snap.packages.clone();
    app.state.packages.detail = Some(
        client
            .fetch_package_detail(&serial, "com.termux")
            .await
            .unwrap(),
    );
    app.state.files.entries = client
        .list_dir(&serial, "/sdcard", true)
        .await
        .unwrap()
        .entries;
    app.state.personal.contacts = snap.contacts.clone();
    app.state.personal.sms = snap.sms.clone();
    app.state.personal.call_logs = snap.call_logs.clone();
    app.state.personal.accounts = snap.accounts.clone();

    let backend = TestBackend::new(140, 42);
    let mut terminal = Terminal::new(backend).unwrap();

    // 1. All 8 tabs
    for tab in Tab::ALL {
        app.state.active_tab = tab;
        terminal
            .draw(|frame| ui::render(frame, &mut app.state))
            .unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("ADBSight"), "header missing on tab {tab:?}");
    }

    // Exercise sub-tabs on System & Personal
    app.state.active_tab = Tab::System;
    for sub in SystemSubTab::ALL {
        app.state.system.sub_tab = sub;
        terminal
            .draw(|frame| ui::render(frame, &mut app.state))
            .unwrap();
    }
    app.state.active_tab = Tab::Personal;
    for sub in PersonalSubTab::ALL {
        app.state.personal.sub_tab = sub;
        terminal
            .draw(|frame| ui::render(frame, &mut app.state))
            .unwrap();
    }

    // 2. Wireless Modal (Devices/mDNS + QR Pair + Manual)
    app.state.active_tab = Tab::Dashboard;
    let qr = QrPairingSession::generate().unwrap();
    app.state.modal = Some(Modal::Wireless(WirelessModalState {
        sub_tab: WirelessSubTab::QrPair,
        qr_session: Some(qr),
        ..Default::default()
    }));
    terminal
        .draw(|frame| ui::render(frame, &mut app.state))
        .unwrap();
    let qr_screen = buffer_text(&terminal);
    assert!(qr_screen.contains("Android 11+ Wireless QR Pairing"));

    app.state.modal = Some(Modal::Wireless(WirelessModalState {
        sub_tab: WirelessSubTab::DevicesAndMdns,
        ..Default::default()
    }));
    terminal
        .draw(|frame| ui::render(frame, &mut app.state))
        .unwrap();
    assert!(buffer_text(&terminal).contains("Attached Devices"));

    // 3. Export, SnapshotHistory, ConfirmAction, Help Modals
    app.state.modal = Some(Modal::Export { selected: 0 });
    terminal
        .draw(|frame| ui::render(frame, &mut app.state))
        .unwrap();
    assert!(buffer_text(&terminal).contains("Export Data"));

    app.state.modal = Some(Modal::SnapshotHistory {
        list: Vec::new(),
        selected: 0,
    });
    terminal
        .draw(|frame| ui::render(frame, &mut app.state))
        .unwrap();
    assert!(buffer_text(&terminal).contains("SQLite Snapshot History"));

    app.state.modal = Some(Modal::ConfirmAction {
        title: "Reboot".into(),
        message: "Confirm reboot?".into(),
        action: PendingAction::Reboot(RebootMode::System),
    });
    terminal
        .draw(|frame| ui::render(frame, &mut app.state))
        .unwrap();
    assert!(buffer_text(&terminal).contains("CONFIRM: Reboot"));

    app.state.modal = Some(Modal::Help);
    terminal
        .draw(|frame| ui::render(frame, &mut app.state))
        .unwrap();
    assert!(buffer_text(&terminal).contains("Keybindings"));

    let _ = fs::remove_dir_all(&tmp_dir);
}
