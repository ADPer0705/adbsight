pub mod events;
pub mod state;
pub mod worker;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::Result;
use chrono::Local;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
    MouseEventKind,
};
use crossterm::execute;
use ratatui::DefaultTerminal;
use tokio::sync::{Mutex, mpsc};

use crate::adb::wireless::QrPairingSession;
use crate::adb::{AdbClient, LogcatHandle};
use crate::app::events::{AppEvent, FullExportFormat, WorkerCommand};
use crate::app::state::{
    AppState, ExportChoice, Modal, PackageFilter, PendingAction, PersonalSubTab, ProcessSubTab,
    PromptKind, SystemSubTab, Tab, TextInput, WirelessField, WirelessModalState, WirelessSubTab,
};
use crate::app::worker::WorkerPool;
use crate::models::{LogLevel, RebootMode};
use crate::storage::config::AppConfig;
use crate::storage::export::{
    export_accounts_csv, export_call_logs_csv, export_contacts_csv, export_json, export_logcat_csv,
    export_packages_csv, export_processes_csv, export_props_csv, export_sms_csv,
};
use crate::ui;

pub struct App {
    pub state: AppState,
    pub client: AdbClient,
    pub cmd_tx: mpsc::UnboundedSender<WorkerCommand>,
    pub event_tx: mpsc::UnboundedSender<AppEvent>,
    pub logcat_slot: Arc<Mutex<Option<LogcatHandle>>>,
}

impl App {
    pub fn new(
        client: AdbClient,
        config: AppConfig,
        db_path: PathBuf,
        adb_info_label: String,
    ) -> Result<(Self, mpsc::UnboundedReceiver<AppEvent>)> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let is_demo = client.is_demo();
        let cmd_tx = WorkerPool::spawn(client.clone(), db_path, event_tx.clone())?;
        let state = AppState::new(config, is_demo, adb_info_label);

        let app = Self {
            state,
            client,
            cmd_tx,
            event_tx,
            logcat_slot: Arc::new(Mutex::new(None)),
        };
        Ok((app, event_rx))
    }

    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        mut event_rx: mpsc::UnboundedReceiver<AppEvent>,
        initial_connect: Option<String>,
        initial_serial: Option<String>,
    ) -> Result<()> {
        let _ = execute!(std::io::stdout(), EnableMouseCapture);

        if let Some(ep) = initial_connect {
            let _ = self.cmd_tx.send(WorkerCommand::ConnectWifi {
                endpoint: ep,
                alias: None,
            });
        }
        if let Some(s) = initial_serial {
            self.state.active_serial = Some(s);
        }

        let _ = self.cmd_tx.send(WorkerCommand::RefreshDevices);
        let _ = self.cmd_tx.send(WorkerCommand::RefreshMdns);

        let stop_flag = Arc::new(AtomicBool::new(false));
        let input_stop = Arc::clone(&stop_flag);
        let input_tx = self.event_tx.clone();
        let input_thread = std::thread::spawn(move || {
            while !input_stop.load(Ordering::Relaxed) {
                if crossterm::event::poll(Duration::from_millis(50)).unwrap_or(false)
                    && let Ok(ev) = crossterm::event::read()
                    && input_tx.send(AppEvent::Input(ev)).is_err()
                {
                    break;
                }
            }
        });

        let refresh_ms = self.state.config.refresh_interval_ms.max(750);
        let tick_tx = self.event_tx.clone();
        let tick_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(refresh_ms));
            loop {
                interval.tick().await;
                if tick_tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
        });

        while !self.state.should_quit {
            self.state.prune_toasts();
            terminal.draw(|frame| ui::render(frame, &mut self.state))?;

            if let Some(ev) = event_rx.recv().await {
                self.handle_event(ev).await;
                // Drain queued events before redrawing for snappy rendering
                while let Ok(more) = event_rx.try_recv() {
                    self.handle_event(more).await;
                }
            } else {
                break;
            }
        }

        stop_flag.store(true, Ordering::Relaxed);
        tick_task.abort();
        if let Some(h) = self.logcat_slot.lock().await.take() {
            h.stop();
        }
        let _ = input_thread.join();
        let _ = execute!(std::io::stdout(), DisableMouseCapture);
        Ok(())
    }

    pub async fn handle_event(&mut self, ev: AppEvent) {
        match ev {
            AppEvent::Input(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                self.handle_key(key).await;
            }
            AppEvent::Input(Event::Mouse(mouse)) => match mouse.kind {
                MouseEventKind::ScrollDown => self.move_selection(1),
                MouseEventKind::ScrollUp => self.move_selection(-1),
                _ => {}
            },
            AppEvent::Input(_) => {}
            AppEvent::Tick => {
                if self.state.viewing_snapshot.is_none()
                    && let Some(serial) = self.state.active_serial.clone()
                    && self.state.active_tab == Tab::Dashboard
                {
                    let _ = self.cmd_tx.send(WorkerCommand::FetchTelemetry { serial });
                }
            }
            AppEvent::LogcatBatch(batch) => {
                if !self.state.logcat.paused {
                    for item in batch {
                        if self.state.logcat.entries.len() >= self.state.logcat.max_lines {
                            self.state.logcat.entries.pop_front();
                        }
                        self.state.logcat.entries.push_back(item);
                    }
                }
            }
            AppEvent::DevicesListed(Ok(devs)) => {
                let prev = self.state.active_serial.clone();
                self.state.devices = devs;
                let still_present = prev.as_ref().is_some_and(|s| {
                    self.state
                        .devices
                        .iter()
                        .any(|d| &d.serial == s && d.state.is_ready())
                });
                if !still_present {
                    if let Some(ready) = self.state.devices.iter().find(|d| d.state.is_ready()) {
                        let serial = ready.serial.clone();
                        self.select_device(serial).await;
                    } else if let Some(first) = self.state.devices.first() {
                        self.state.active_serial = Some(first.serial.clone());
                    }
                } else if let Some(s) = prev
                    && !self.state.is_tab_loaded(&s, Tab::Dashboard)
                {
                    self.select_device(s).await;
                }
            }
            AppEvent::DevicesListed(Err(e)) => {
                self.state
                    .push_toast(format!("Device scan failed: {e}"), true);
            }
            AppEvent::MdnsListed(Ok(svcs)) => {
                self.state.mdns_services = svcs;
            }
            AppEvent::MdnsListed(Err(_)) => {}
            AppEvent::WirelessActionDone {
                message,
                connected_endpoint,
                refresh_devices,
            } => {
                self.state.push_toast(message.clone(), false);
                if let Some(ep) = connected_endpoint {
                    self.state
                        .config
                        .bookmark_wifi_device(ep.clone(), ep.clone());
                    let _ = self.state.config.save();
                    self.state.active_serial = Some(ep);
                }
                if let Some(Modal::Wireless(ref mut w)) = self.state.modal {
                    w.qr_status = message;
                }
                if refresh_devices {
                    let _ = self.cmd_tx.send(WorkerCommand::RefreshDevices);
                    let _ = self.cmd_tx.send(WorkerCommand::RefreshMdns);
                }
            }
            AppEvent::WirelessActionFailed(err) => {
                self.state.push_toast(err.clone(), true);
                if let Some(Modal::Wireless(ref mut w)) = self.state.modal {
                    w.qr_status = err;
                }
            }
            AppEvent::QrPairingStatus(status) => {
                if let Some(Modal::Wireless(ref mut w)) = self.state.modal {
                    w.qr_status = status;
                }
            }
            AppEvent::RootChecked { serial, status } => {
                if self.state.active_serial.as_deref() == Some(serial.as_str()) {
                    self.state.root_status = status;
                }
            }
            AppEvent::IdentityLoaded { serial, result } => {
                if self.state.active_serial.as_deref() == Some(serial.as_str()) {
                    match result {
                        Ok(id) => {
                            self.state.dashboard.identity = Some(id);
                            self.state.mark_tab_loaded(&serial, Tab::Dashboard);
                        }
                        Err(e) => self.state.push_toast(format!("Identity error: {e}"), true),
                    }
                }
            }
            AppEvent::TelemetryLoaded { serial, result } => {
                if self.state.active_serial.as_deref() == Some(serial.as_str())
                    && let Ok(tel) = result
                {
                    let cpu_val = tel.cpu.total_pct.round() as u64;
                    let rx_kb = (tel.network.rx_bytes_per_sec / 1024.0).round() as u64;
                    let tx_kb = (tel.network.tx_bytes_per_sec / 1024.0).round() as u64;
                    push_history(&mut self.state.dashboard.cpu_history, cpu_val, 60);
                    push_history(&mut self.state.dashboard.rx_history, rx_kb, 60);
                    push_history(&mut self.state.dashboard.tx_history, tx_kb, 60);
                    self.state.dashboard.telemetry = Some(tel);
                }
            }
            AppEvent::SystemAllLoaded {
                serial,
                props,
                settings,
                sensors,
                dumpsys_list,
            } => {
                if self.state.active_serial.as_deref() == Some(serial.as_str()) {
                    self.state.system.loading = false;
                    if let Ok(p) = props {
                        self.state.system.props = p;
                    }
                    if let Ok(s) = settings {
                        self.state.system.settings = s;
                    }
                    if let Ok(sn) = sensors {
                        self.state.system.sensors = sn;
                    }
                    if let Ok(dl) = dumpsys_list {
                        self.state.system.dumpsys_list = dl;
                    }
                    self.state.mark_tab_loaded(&serial, Tab::System);
                }
            }
            AppEvent::DumpsysServiceLoaded { service, result } => {
                self.state.system.dumpsys_active_service = Some(service);
                self.state.system.dumpsys_scroll = 0;
                self.state.system.dumpsys_output = match result {
                    Ok(out) => out,
                    Err(e) => format!("Error dumping service: {e}"),
                };
            }
            AppEvent::ProcessesLoaded {
                serial,
                procs,
                services,
            } => {
                if self.state.active_serial.as_deref() == Some(serial.as_str()) {
                    self.state.processes.loading = false;
                    if let Ok(p) = procs {
                        self.state.processes.processes = p;
                    }
                    if let Ok(s) = services {
                        self.state.processes.services = s;
                    }
                    self.state.mark_tab_loaded(&serial, Tab::Processes);
                }
            }
            AppEvent::PackagesLoaded { serial, result } => {
                if self.state.active_serial.as_deref() == Some(serial.as_str()) {
                    self.state.packages.loading = false;
                    match result {
                        Ok(pkgs) => {
                            self.state.packages.packages = pkgs;
                            self.state.mark_tab_loaded(&serial, Tab::Packages);
                        }
                        Err(e) => self.state.push_toast(format!("Packages error: {e}"), true),
                    }
                }
            }
            AppEvent::PackageDetailLoaded { package, result } => {
                self.state.packages.detail_loading = false;
                if let Ok(detail) = result {
                    self.state.packages.detail = Some(detail);
                    self.state.packages.detail_scroll = 0;
                } else if let Err(e) = result {
                    self.state
                        .push_toast(format!("Failed inspecting {package}: {e}"), true);
                }
            }
            AppEvent::DirLoaded { path, result } => {
                self.state.files.loading = false;
                match result {
                    Ok(res) => {
                        self.state.files.current_path = path;
                        self.state.files.entries = res.entries;
                        self.state.files.permission_denied = res.permission_denied;
                        self.state.files.selected = 0;
                        if let Some(serial) = self.state.active_serial.clone() {
                            self.state.mark_tab_loaded(&serial, Tab::Files);
                        }
                    }
                    Err(e) => self.state.push_toast(format!("List dir failed: {e}"), true),
                }
            }
            AppEvent::FilePreviewLoaded { path, result } => {
                self.state.files.preview_title = path;
                self.state.files.preview_scroll = 0;
                self.state.files.preview_content = match result {
                    Ok(txt) => txt,
                    Err(e) => format!("Preview unavailable: {e}"),
                };
            }
            AppEvent::PersonalAllLoaded {
                serial,
                contacts,
                sms,
                calls,
                accounts,
            } => {
                if self.state.active_serial.as_deref() == Some(serial.as_str()) {
                    self.state.personal.loading = false;
                    if let Ok(c) = contacts {
                        self.state.personal.contacts = c;
                    }
                    if let Ok(s) = sms {
                        self.state.personal.sms = s;
                    }
                    if let Ok(cl) = calls {
                        self.state.personal.call_logs = cl;
                    }
                    if let Ok(a) = accounts {
                        self.state.personal.accounts = a;
                    }
                    self.state.mark_tab_loaded(&serial, Tab::Personal);
                }
            }
            AppEvent::ShellOutput { command, result } => {
                self.state.shell.running = false;
                self.state.shell.output_lines.push(format!("$ {command}"));
                match result {
                    Ok(out) => {
                        if out.trim().is_empty() {
                            self.state.shell.output_lines.push("(no output)".into());
                        } else {
                            for line in out.lines() {
                                self.state.shell.output_lines.push(line.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.state.shell.output_lines.push(format!("[error] {e}"));
                    }
                }
                if self.state.shell.output_lines.len() > 2000 {
                    let excess = self.state.shell.output_lines.len() - 2000;
                    self.state.shell.output_lines.drain(0..excess);
                }
                self.state.shell.scroll =
                    self.state.shell.output_lines.len().saturating_sub(1) as u16;
            }
            AppEvent::ActionToast {
                ok,
                message,
                refresh_after,
            } => {
                self.state.push_toast(message, !ok);
                if refresh_after {
                    self.refresh_active_tab().await;
                }
            }
            AppEvent::SnapshotSaved(res) => match res {
                Ok((id, label)) => {
                    self.state
                        .push_toast(format!("Saved SQLite snapshot #{id} ({label})"), false);
                }
                Err(e) => {
                    self.state
                        .push_toast(format!("Snapshot save failed: {e}"), true);
                }
            },
            AppEvent::SnapshotsListed(res) => match res {
                Ok(list) => {
                    self.state.modal = Some(Modal::SnapshotHistory { list, selected: 0 });
                }
                Err(e) => self.state.push_toast(format!("DB read failed: {e}"), true),
            },
            AppEvent::SnapshotLoaded(res) => match *res {
                Ok((meta, snap)) => {
                    self.state.modal = None;
                    let label = meta.label.clone();
                    self.state.apply_loaded_snapshot(meta, snap);
                    self.state
                        .push_toast(format!("Viewing offline snapshot: {label}"), false);
                }
                Err(e) => self
                    .state
                    .push_toast(format!("Load snapshot failed: {e}"), true),
            },
        }
    }

    pub async fn select_device(&mut self, serial: String) {
        self.state.active_serial = Some(serial.clone());
        self.state.viewing_snapshot = None;
        self.state.reset_device_caches();

        let _ = self.cmd_tx.send(WorkerCommand::CheckRoot {
            serial: serial.clone(),
        });
        let _ = self.cmd_tx.send(WorkerCommand::FetchIdentity {
            serial: serial.clone(),
        });
        let _ = self.cmd_tx.send(WorkerCommand::FetchTelemetry {
            serial: serial.clone(),
        });

        self.state.logcat.entries.clear();
        self.state.logcat.streaming = true;
        WorkerPool::ensure_logcat(&self.client, &serial, &self.event_tx, &self.logcat_slot).await;

        if self.state.active_tab != Tab::Dashboard {
            self.load_tab_if_needed(self.state.active_tab).await;
        }
    }

    pub async fn switch_tab(&mut self, tab: Tab) {
        self.state.active_tab = tab;
        self.load_tab_if_needed(tab).await;
    }

    async fn load_tab_if_needed(&mut self, tab: Tab) {
        if self.state.viewing_snapshot.is_some() {
            return;
        }
        let Some(serial) = self.state.active_serial.clone() else {
            return;
        };
        if self.state.is_tab_loaded(&serial, tab) {
            return;
        }
        self.refresh_tab(tab, &serial).await;
    }

    pub async fn refresh_active_tab(&mut self) {
        if self.state.viewing_snapshot.is_some() {
            self.state.viewing_snapshot = None;
            self.state
                .push_toast("Returned to live device inspection", false);
        }
        let Some(serial) = self.state.active_serial.clone() else {
            let _ = self.cmd_tx.send(WorkerCommand::RefreshDevices);
            return;
        };
        self.refresh_tab(self.state.active_tab, &serial).await;
    }

    async fn refresh_tab(&mut self, tab: Tab, serial: &str) {
        let use_root = self.state.use_root;
        match tab {
            Tab::Dashboard => {
                let _ = self.cmd_tx.send(WorkerCommand::FetchIdentity {
                    serial: serial.to_string(),
                });
                let _ = self.cmd_tx.send(WorkerCommand::FetchTelemetry {
                    serial: serial.to_string(),
                });
            }
            Tab::System => {
                self.state.system.loading = true;
                let _ = self.cmd_tx.send(WorkerCommand::FetchSystemAll {
                    serial: serial.to_string(),
                });
            }
            Tab::Processes => {
                self.state.processes.loading = true;
                let _ = self.cmd_tx.send(WorkerCommand::FetchProcesses {
                    serial: serial.to_string(),
                    use_root,
                });
            }
            Tab::Packages => {
                self.state.packages.loading = true;
                let _ = self.cmd_tx.send(WorkerCommand::FetchPackages {
                    serial: serial.to_string(),
                });
            }
            Tab::Logcat => {
                self.state.logcat.streaming = true;
                WorkerPool::ensure_logcat(&self.client, serial, &self.event_tx, &self.logcat_slot)
                    .await;
            }
            Tab::Files => {
                self.state.files.loading = true;
                let _ = self.cmd_tx.send(WorkerCommand::ListDir {
                    serial: serial.to_string(),
                    path: self.state.files.current_path.clone(),
                    use_root,
                });
            }
            Tab::Personal => {
                self.state.personal.loading = true;
                let _ = self.cmd_tx.send(WorkerCommand::FetchPersonalAll {
                    serial: serial.to_string(),
                    use_root,
                });
            }
            Tab::Shell => {}
        }
    }

    async fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.state.should_quit = true;
            return;
        }

        if self.state.modal.is_some() {
            self.handle_modal_key(key).await;
            return;
        }

        // Active text filter typing
        if self.active_filter_input().is_some_and(|f| f.active) {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    if let Some(f) = self.active_filter_input_mut() {
                        f.active = false;
                    }
                }
                KeyCode::Backspace => {
                    if let Some(f) = self.active_filter_input_mut() {
                        f.pop();
                    }
                    self.reset_tab_selection();
                }
                KeyCode::Char(c) => {
                    if let Some(f) = self.active_filter_input_mut() {
                        f.push(c);
                    }
                    self.reset_tab_selection();
                }
                _ => {}
            }
            return;
        }

        // Shell tab direct command input when active
        if self.state.active_tab == Tab::Shell && self.state.shell.input.active {
            match key.code {
                KeyCode::Esc => {
                    self.state.shell.input.active = false;
                    return;
                }
                KeyCode::Enter => {
                    let cmd = self.state.shell.input.value.trim().to_string();
                    self.state.shell.input.clear();
                    if !cmd.is_empty() {
                        self.state.shell.history.push(cmd.clone());
                        self.state.shell.history_idx = None;
                        if let Some(serial) = self.state.active_serial.clone() {
                            self.state.shell.running = true;
                            let _ = self.cmd_tx.send(WorkerCommand::RunShell {
                                serial,
                                command: cmd,
                                use_root: self.state.use_root,
                            });
                        } else {
                            self.state.push_toast("No active device selected", true);
                        }
                    }
                    return;
                }
                KeyCode::Up => {
                    let len = self.state.shell.history.len();
                    if len > 0 {
                        let next = match self.state.shell.history_idx {
                            None => len - 1,
                            Some(i) => i.saturating_sub(1),
                        };
                        self.state.shell.history_idx = Some(next);
                        self.state.shell.input.value = self.state.shell.history[next].clone();
                    }
                    return;
                }
                KeyCode::Down => {
                    if let Some(i) = self.state.shell.history_idx {
                        if i + 1 < self.state.shell.history.len() {
                            self.state.shell.history_idx = Some(i + 1);
                            self.state.shell.input.value = self.state.shell.history[i + 1].clone();
                        } else {
                            self.state.shell.history_idx = None;
                            self.state.shell.input.clear();
                        }
                    }
                    return;
                }
                KeyCode::Backspace => {
                    self.state.shell.input.pop();
                    return;
                }
                KeyCode::F(2) => {
                    self.trigger_screencap();
                    return;
                }
                KeyCode::F(3) => {
                    self.open_reboot_confirm(RebootMode::System);
                    return;
                }
                KeyCode::F(4) => {
                    self.open_snapshot_save_modal();
                    return;
                }
                KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.open_input_text_prompt();
                    return;
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.state.shell.input.push(c);
                    return;
                }
                _ => {}
            }
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => {
                self.state.should_quit = true;
                return;
            }
            KeyCode::Char('?') => {
                self.state.modal = Some(Modal::Help);
                return;
            }
            KeyCode::Char('w') => {
                self.open_wireless_modal();
                return;
            }
            KeyCode::Char('e') if self.state.active_tab != Tab::Logcat => {
                self.state.modal = Some(Modal::Export { selected: 0 });
                return;
            }
            KeyCode::Char('S') => {
                self.open_snapshot_save_modal();
                return;
            }
            KeyCode::Char('H') => {
                let _ = self.cmd_tx.send(WorkerCommand::ListSnapshots);
                return;
            }
            KeyCode::Char('R') => {
                self.toggle_root().await;
                return;
            }
            KeyCode::Char('r') => {
                self.refresh_active_tab().await;
                return;
            }
            KeyCode::F(2) => {
                self.trigger_screencap();
                return;
            }
            KeyCode::F(3) => {
                self.open_reboot_confirm(RebootMode::System);
                return;
            }
            KeyCode::F(4) => {
                self.open_snapshot_save_modal();
                return;
            }
            KeyCode::Char(d @ '1'..='8') => {
                if let Some(tab) = Tab::from_digit(d) {
                    // In Files tab, 1/2/3 jump if Ctrl is not needed? No, digits 1-8 always switch top tabs cleanly.
                    self.switch_tab(tab).await;
                    return;
                }
            }
            KeyCode::Char('/') => {
                if let Some(f) = self.active_filter_input_mut() {
                    f.active = true;
                }
                return;
            }
            KeyCode::Esc => {
                if let Some(f) = self.active_filter_input_mut()
                    && !f.value.is_empty()
                {
                    f.clear();
                    self.reset_tab_selection();
                } else if self.state.viewing_snapshot.is_some() {
                    self.state.viewing_snapshot = None;
                    self.state.push_toast("Exited offline snapshot view", false);
                }
                return;
            }
            _ => {}
        }

        // Tab-specific navigation and actions
        match self.state.active_tab {
            Tab::Dashboard => {}
            Tab::System => self.handle_system_key(key),
            Tab::Processes => self.handle_processes_key(key),
            Tab::Packages => self.handle_packages_key(key),
            Tab::Logcat => self.handle_logcat_key(key),
            Tab::Files => self.handle_files_key(key),
            Tab::Personal => self.handle_personal_key(key),
            Tab::Shell => match key.code {
                KeyCode::Enter | KeyCode::Char('i') => {
                    self.state.shell.input.active = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.state.shell.scroll = self.state.shell.scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.state.shell.scroll = self.state.shell.scroll.saturating_add(1);
                }
                _ => {}
            },
        }
    }

    async fn toggle_root(&mut self) {
        if !self.state.use_root {
            if self.state.root_status.is_available() {
                self.state.use_root = true;
                self.state
                    .push_toast("Root Mode ENABLED — refreshing protected views", false);
                self.refresh_active_tab().await;
            } else {
                self.state.push_toast(
                    "Root not detected on device (neither adbd root nor su binary found)",
                    true,
                );
            }
        } else {
            self.state.use_root = false;
            self.state.push_toast("Root Mode disabled", false);
            self.refresh_active_tab().await;
        }
    }

    fn open_wireless_modal(&mut self) {
        let mut w = WirelessModalState::default();
        if let Ok(session) = QrPairingSession::generate() {
            w.qr_session = Some(session);
        }
        self.state.modal = Some(Modal::Wireless(w));
        let _ = self.cmd_tx.send(WorkerCommand::RefreshDevices);
        let _ = self.cmd_tx.send(WorkerCommand::RefreshMdns);
    }

    fn open_snapshot_save_modal(&mut self) {
        let default_label = format!("snap-{}", Local::now().format("%Y%m%d-%H%M%S"));
        let mut inp = TextInput::with_value(default_label);
        inp.active = true;
        self.state.modal = Some(Modal::SnapshotSave(inp));
    }

    fn open_reboot_confirm(&mut self, mode: RebootMode) {
        self.state.modal = Some(Modal::ConfirmAction {
            title: mode.label().to_string(),
            message: format!("Execute {} on active device?", mode.label()),
            action: PendingAction::Reboot(mode),
        });
    }

    fn open_input_text_prompt(&mut self) {
        let inp = TextInput {
            active: true,
            ..Default::default()
        };
        self.state.modal = Some(Modal::InputPrompt {
            title: "Send Text Input to Device".into(),
            prompt: "Text to type via `adb shell input text`:".into(),
            input: inp,
            kind: PromptKind::SendInputText,
        });
    }

    fn trigger_screencap(&mut self) {
        let Some(serial) = self.state.active_serial.clone() else {
            self.state.push_toast("No active device selected", true);
            return;
        };
        let dir = self.state.config.resolved_export_dir();
        let file = dir.join(format!(
            "screenshot_{}.png",
            Local::now().format("%Y%m%d_%H%M%S")
        ));
        let _ = self.cmd_tx.send(WorkerCommand::Screencap {
            serial,
            local: file,
        });
    }

    fn handle_system_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.state.system.sub_tab = self.state.system.sub_tab.next();
                self.state.system.selected = 0;
            }
            KeyCode::BackTab => {
                self.state.system.sub_tab = self.state.system.sub_tab.prev();
                self.state.system.selected = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::PageDown => {
                if self.state.system.sub_tab == SystemSubTab::Dumpsys {
                    self.state.system.dumpsys_scroll =
                        self.state.system.dumpsys_scroll.saturating_add(12);
                } else {
                    self.move_selection(15);
                }
            }
            KeyCode::PageUp => {
                if self.state.system.sub_tab == SystemSubTab::Dumpsys {
                    self.state.system.dumpsys_scroll =
                        self.state.system.dumpsys_scroll.saturating_sub(12);
                } else {
                    self.move_selection(-15);
                }
            }
            KeyCode::Char('g') => self.state.system.selected = 0,
            KeyCode::Enter if self.state.system.sub_tab == SystemSubTab::Dumpsys => {
                let filtered = self.filtered_dumpsys_services();
                if let Some(svc) = filtered.get(self.state.system.selected).cloned()
                    && let Some(serial) = self.state.active_serial.clone()
                {
                    let _ = self.cmd_tx.send(WorkerCommand::FetchDumpsysService {
                        serial,
                        service: svc,
                        use_root: self.state.use_root,
                    });
                }
            }
            _ => {}
        }
    }

    fn handle_processes_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab | KeyCode::BackTab => {
                self.state.processes.sub_tab = match self.state.processes.sub_tab {
                    ProcessSubTab::Processes => ProcessSubTab::Services,
                    ProcessSubTab::Services => ProcessSubTab::Processes,
                };
                self.state.processes.selected = 0;
            }
            KeyCode::Char('s') => {
                self.state.processes.sort = self.state.processes.sort.next();
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::PageDown => self.move_selection(15),
            KeyCode::PageUp => self.move_selection(-15),
            KeyCode::Char('g') => self.state.processes.selected = 0,
            KeyCode::Char('K') if self.state.processes.sub_tab == ProcessSubTab::Processes => {
                let list = self.filtered_sorted_processes();
                if let Some(p) = list.get(self.state.processes.selected) {
                    self.state.modal = Some(Modal::ConfirmAction {
                        title: "Kill Process".into(),
                        message: format!("Send SIGKILL to PID {} ({})?", p.pid, p.name),
                        action: PendingAction::KillPid {
                            pid: p.pid,
                            name: p.name.clone(),
                        },
                    });
                }
            }
            _ => {}
        }
    }

    fn handle_packages_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('f') => {
                self.state.packages.filter_mode = self.state.packages.filter_mode.next();
                self.state.packages.selected = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::PageDown => {
                self.state.packages.detail_scroll =
                    self.state.packages.detail_scroll.saturating_add(10);
            }
            KeyCode::PageUp => {
                self.state.packages.detail_scroll =
                    self.state.packages.detail_scroll.saturating_sub(10);
            }
            KeyCode::Char('g') => self.state.packages.selected = 0,
            KeyCode::Enter => {
                if let Some(pkg) = self.selected_package().cloned()
                    && let Some(serial) = self.state.active_serial.clone()
                {
                    self.state.packages.detail_loading = true;
                    let _ = self.cmd_tx.send(WorkerCommand::FetchPackageDetail {
                        serial,
                        package: pkg.name,
                    });
                }
            }
            KeyCode::Char('l') => {
                if let Some(pkg) = self.selected_package().cloned()
                    && let Some(serial) = self.state.active_serial.clone()
                {
                    let _ = self.cmd_tx.send(WorkerCommand::LaunchApp {
                        serial,
                        package: pkg.name,
                    });
                }
            }
            KeyCode::Char('x') => {
                if let Some(pkg) = self.selected_package().cloned() {
                    self.state.modal = Some(Modal::ConfirmAction {
                        title: "Force Stop App".into(),
                        message: format!("Force-stop `{}`?", pkg.name),
                        action: PendingAction::ForceStopApp(pkg.name),
                    });
                }
            }
            KeyCode::Char('c') => {
                if let Some(pkg) = self.selected_package().cloned() {
                    self.state.modal = Some(Modal::ConfirmAction {
                        title: "Clear App Data".into(),
                        message: format!(
                            "Clear ALL app data & cache for `{}`? This cannot be undone.",
                            pkg.name
                        ),
                        action: PendingAction::ClearAppData(pkg.name),
                    });
                }
            }
            KeyCode::Char('p') => {
                if let Some(pkg) = self.selected_package().cloned()
                    && let Some(serial) = self.state.active_serial.clone()
                {
                    if pkg.apk_path.is_empty() {
                        self.state
                            .push_toast("APK path unknown for this package", true);
                    } else {
                        let out = self
                            .state
                            .config
                            .resolved_export_dir()
                            .join("apks")
                            .join(format!("{}.apk", pkg.name));
                        let _ = self.cmd_tx.send(WorkerCommand::PullApk {
                            serial,
                            apk_remote: pkg.apk_path,
                            local: out,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_logcat_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(' ') => {
                self.state.logcat.paused = !self.state.logcat.paused;
            }
            KeyCode::Char('C') => {
                self.state.logcat.entries.clear();
                self.state.logcat.scroll = 0;
            }
            KeyCode::Char('v') => self.state.logcat.min_level = LogLevel::Verbose,
            KeyCode::Char('d') => self.state.logcat.min_level = LogLevel::Debug,
            KeyCode::Char('i') => self.state.logcat.min_level = LogLevel::Info,
            KeyCode::Char('w') => self.state.logcat.min_level = LogLevel::Warn,
            KeyCode::Char('e') => self.state.logcat.min_level = LogLevel::Error,
            KeyCode::Char('f') => self.state.logcat.min_level = LogLevel::Fatal,
            KeyCode::Char('E') => {
                self.state.modal = Some(Modal::Export { selected: 0 });
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.logcat.auto_scroll = false;
                self.state.logcat.scroll = self.state.logcat.scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.logcat.auto_scroll = false;
                self.state.logcat.scroll = self.state.logcat.scroll.saturating_sub(1);
            }
            KeyCode::Char('G') => {
                self.state.logcat.auto_scroll = true;
            }
            _ => {}
        }
    }

    fn handle_files_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::PageDown => {
                self.state.files.preview_scroll =
                    self.state.files.preview_scroll.saturating_add(10);
            }
            KeyCode::PageUp => {
                self.state.files.preview_scroll =
                    self.state.files.preview_scroll.saturating_sub(10);
            }
            KeyCode::Backspace | KeyCode::Char('h') => {
                let parent = parent_android_path(&self.state.files.current_path);
                if let Some(serial) = self.state.active_serial.clone() {
                    self.state.files.loading = true;
                    let _ = self.cmd_tx.send(WorkerCommand::ListDir {
                        serial,
                        path: parent,
                        use_root: self.state.use_root,
                    });
                }
            }
            KeyCode::Char('d') => self.jump_dir("/sdcard"),
            KeyCode::Char('t') => self.jump_dir("/data/local/tmp"),
            KeyCode::Char('/') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.jump_dir("/");
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(entry) = self.selected_file_entry().cloned()
                    && let Some(serial) = self.state.active_serial.clone()
                {
                    match entry.kind {
                        crate::models::FsKind::Directory | crate::models::FsKind::Symlink => {
                            self.state.files.loading = true;
                            let _ = self.cmd_tx.send(WorkerCommand::ListDir {
                                serial,
                                path: entry.path,
                                use_root: self.state.use_root,
                            });
                        }
                        crate::models::FsKind::File => {
                            let _ = self.cmd_tx.send(WorkerCommand::PreviewFile {
                                serial,
                                path: entry.path,
                                use_root: self.state.use_root,
                            });
                        }
                        crate::models::FsKind::Other => {}
                    }
                }
            }
            KeyCode::Char('p') => {
                if let Some(entry) = self.selected_file_entry().cloned() {
                    let default_local = self
                        .state
                        .config
                        .resolved_export_dir()
                        .join("pulled")
                        .join(&entry.name);
                    let mut inp =
                        TextInput::with_value(default_local.to_string_lossy().to_string());
                    inp.active = true;
                    self.state.modal = Some(Modal::InputPrompt {
                        title: "Pull Remote Path".into(),
                        prompt: format!("Pull `{}` to local path:", entry.path),
                        input: inp,
                        kind: PromptKind::PullFile { remote: entry.path },
                    });
                }
            }
            KeyCode::Char('u') => {
                let inp = TextInput {
                    active: true,
                    ..Default::default()
                };
                self.state.modal = Some(Modal::InputPrompt {
                    title: "Push Local File".into(),
                    prompt: format!(
                        "Local file path to push into `{}`:",
                        self.state.files.current_path
                    ),
                    input: inp,
                    kind: PromptKind::PushFile {
                        remote_dir: self.state.files.current_path.clone(),
                    },
                });
            }
            _ => {}
        }
    }

    fn jump_dir(&mut self, path: &str) {
        if let Some(serial) = self.state.active_serial.clone() {
            self.state.files.loading = true;
            let _ = self.cmd_tx.send(WorkerCommand::ListDir {
                serial,
                path: path.to_string(),
                use_root: self.state.use_root,
            });
        }
    }

    fn handle_personal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.state.personal.sub_tab = self.state.personal.sub_tab.next();
                self.state.personal.selected = 0;
            }
            KeyCode::BackTab => {
                self.state.personal.sub_tab = self.state.personal.sub_tab.prev();
                self.state.personal.selected = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('g') => self.state.personal.selected = 0,
            _ => {}
        }
    }

    async fn handle_modal_key(&mut self, key: KeyEvent) {
        let Some(modal) = self.state.modal.take() else {
            return;
        };

        match modal {
            Modal::Help => match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter => {}
                _ => self.state.modal = Some(Modal::Help),
            },
            Modal::Export { mut selected } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {}
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.saturating_sub(1);
                    self.state.modal = Some(Modal::Export { selected });
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    selected = (selected + 1).min(ExportChoice::ALL.len() - 1);
                    self.state.modal = Some(Modal::Export { selected });
                }
                KeyCode::Enter => {
                    self.execute_export(ExportChoice::ALL[selected]);
                }
                _ => self.state.modal = Some(Modal::Export { selected }),
            },
            Modal::SnapshotSave(mut inp) => match key.code {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    let label = inp.value.trim().to_string();
                    if let Some(serial) = self.state.active_serial.clone() {
                        let _ = self.cmd_tx.send(WorkerCommand::SaveFullSnapshot {
                            serial,
                            use_root: self.state.use_root,
                            label: if label.is_empty() {
                                "manual-snapshot".into()
                            } else {
                                label
                            },
                        });
                    } else {
                        self.state.push_toast("No active device to snapshot", true);
                    }
                }
                KeyCode::Backspace => {
                    inp.pop();
                    self.state.modal = Some(Modal::SnapshotSave(inp));
                }
                KeyCode::Char(c) => {
                    inp.push(c);
                    self.state.modal = Some(Modal::SnapshotSave(inp));
                }
                _ => self.state.modal = Some(Modal::SnapshotSave(inp)),
            },
            Modal::SnapshotHistory { list, mut selected } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {}
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.saturating_sub(1);
                    self.state.modal = Some(Modal::SnapshotHistory { list, selected });
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !list.is_empty() {
                        selected = (selected + 1).min(list.len() - 1);
                    }
                    self.state.modal = Some(Modal::SnapshotHistory { list, selected });
                }
                KeyCode::Enter => {
                    if let Some(meta) = list.get(selected) {
                        let _ = self
                            .cmd_tx
                            .send(WorkerCommand::LoadSnapshot { id: meta.id });
                    }
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    if let Some(meta) = list.get(selected) {
                        let _ = self
                            .cmd_tx
                            .send(WorkerCommand::DeleteSnapshot { id: meta.id });
                    }
                }
                _ => self.state.modal = Some(Modal::SnapshotHistory { list, selected }),
            },
            Modal::ConfirmAction {
                title,
                message,
                action,
            } => match key.code {
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {}
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.execute_confirmed_action(action);
                }
                _ => {
                    self.state.modal = Some(Modal::ConfirmAction {
                        title,
                        message,
                        action,
                    });
                }
            },
            Modal::InputPrompt {
                title,
                prompt,
                mut input,
                kind,
            } => match key.code {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    let val = input.value.trim().to_string();
                    if !val.is_empty()
                        && let Some(serial) = self.state.active_serial.clone()
                    {
                        match kind {
                            PromptKind::PullFile { remote } => {
                                let _ = self.cmd_tx.send(WorkerCommand::PullPath {
                                    serial,
                                    remote,
                                    local: PathBuf::from(val),
                                });
                            }
                            PromptKind::PushFile { remote_dir } => {
                                let local = PathBuf::from(&val);
                                let fname = local
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "upload.bin".into());
                                let remote =
                                    format!("{}/{}", remote_dir.trim_end_matches('/'), fname);
                                self.state.modal = Some(Modal::ConfirmAction {
                                    title: "Confirm File Push".into(),
                                    message: format!("Push `{val}` -> `{remote}`?"),
                                    action: PendingAction::PushFile { local, remote },
                                });
                            }
                            PromptKind::SendInputText => {
                                let _ = self
                                    .cmd_tx
                                    .send(WorkerCommand::SendInputText { serial, text: val });
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                    self.state.modal = Some(Modal::InputPrompt {
                        title,
                        prompt,
                        input,
                        kind,
                    });
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    self.state.modal = Some(Modal::InputPrompt {
                        title,
                        prompt,
                        input,
                        kind,
                    });
                }
                _ => {
                    self.state.modal = Some(Modal::InputPrompt {
                        title,
                        prompt,
                        input,
                        kind,
                    });
                }
            },
            Modal::Wireless(mut w) => {
                if key.code == KeyCode::Esc {
                    return;
                }
                if key.code == KeyCode::Tab {
                    w.sub_tab = match w.sub_tab {
                        WirelessSubTab::DevicesAndMdns => WirelessSubTab::QrPair,
                        WirelessSubTab::QrPair => WirelessSubTab::ManualPairConnect,
                        WirelessSubTab::ManualPairConnect => WirelessSubTab::DevicesAndMdns,
                    };
                    if w.sub_tab == WirelessSubTab::QrPair
                        && let Some(ref qr) = w.qr_session
                    {
                        let _ = self.cmd_tx.send(WorkerCommand::WatchQrPairing {
                            service_name: qr.service_name.clone(),
                            password: qr.password.clone(),
                        });
                    }
                    self.state.modal = Some(Modal::Wireless(w));
                    return;
                }

                match w.sub_tab {
                    WirelessSubTab::DevicesAndMdns => {
                        match key.code {
                            KeyCode::Char('1') => w.focus_section = 0,
                            KeyCode::Char('2') => w.focus_section = 1,
                            KeyCode::Char('3') => w.focus_section = 2,
                            KeyCode::Char('j') | KeyCode::Down => match w.focus_section {
                                0 if !self.state.devices.is_empty() => {
                                    w.selected_device =
                                        (w.selected_device + 1).min(self.state.devices.len() - 1);
                                }
                                1 if !self.state.mdns_services.is_empty() => {
                                    w.selected_mdns = (w.selected_mdns + 1)
                                        .min(self.state.mdns_services.len() - 1);
                                }
                                2 if !self.state.config.saved_devices.is_empty() => {
                                    w.selected_bookmark = (w.selected_bookmark + 1)
                                        .min(self.state.config.saved_devices.len() - 1);
                                }
                                _ => {}
                            },
                            KeyCode::Char('k') | KeyCode::Up => match w.focus_section {
                                0 => w.selected_device = w.selected_device.saturating_sub(1),
                                1 => w.selected_mdns = w.selected_mdns.saturating_sub(1),
                                2 => w.selected_bookmark = w.selected_bookmark.saturating_sub(1),
                                _ => {}
                            },
                            KeyCode::Char('r') => {
                                let _ = self.cmd_tx.send(WorkerCommand::RefreshDevices);
                                let _ = self.cmd_tx.send(WorkerCommand::RefreshMdns);
                            }
                            KeyCode::Char('t') => {
                                if let Some(dev) = self.state.devices.get(w.selected_device) {
                                    let _ = self.cmd_tx.send(WorkerCommand::UsbToWifi {
                                        usb_serial: dev.serial.clone(),
                                    });
                                }
                            }
                            KeyCode::Char('x') => {
                                if let Some(dev) = self.state.devices.get(w.selected_device) {
                                    let _ = self.cmd_tx.send(WorkerCommand::DisconnectWifi {
                                        endpoint: dev.serial.clone(),
                                    });
                                }
                            }
                            KeyCode::Enter => match w.focus_section {
                                0 => {
                                    if let Some(dev) =
                                        self.state.devices.get(w.selected_device).cloned()
                                    {
                                        self.select_device(dev.serial).await;
                                        return;
                                    }
                                }
                                1 => {
                                    if let Some(svc) = self.state.mdns_services.get(w.selected_mdns)
                                    {
                                        let ep = format!("{}:{}", svc.address, svc.port);
                                        let _ = self.cmd_tx.send(WorkerCommand::ConnectWifi {
                                            endpoint: ep,
                                            alias: Some(svc.name.clone()),
                                        });
                                    }
                                }
                                2 => {
                                    if let Some(bm) =
                                        self.state.config.saved_devices.get(w.selected_bookmark)
                                    {
                                        let _ = self.cmd_tx.send(WorkerCommand::ConnectWifi {
                                            endpoint: bm.address.clone(),
                                            alias: Some(bm.name.clone()),
                                        });
                                    }
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                        self.state.modal = Some(Modal::Wireless(w));
                    }
                    WirelessSubTab::QrPair => {
                        match key.code {
                            KeyCode::Char('g') | KeyCode::Char('r') => {
                                if let Ok(session) = QrPairingSession::generate() {
                                    let _ = self.cmd_tx.send(WorkerCommand::WatchQrPairing {
                                        service_name: session.service_name.clone(),
                                        password: session.password.clone(),
                                    });
                                    w.qr_session = Some(session);
                                }
                            }
                            _ => {}
                        }
                        self.state.modal = Some(Modal::Wireless(w));
                    }
                    WirelessSubTab::ManualPairConnect => {
                        match key.code {
                            KeyCode::Up | KeyCode::Down => {
                                w.active_field = match w.active_field {
                                    WirelessField::Endpoint => WirelessField::PairingCode,
                                    WirelessField::PairingCode => WirelessField::Endpoint,
                                };
                            }
                            KeyCode::Backspace => match w.active_field {
                                WirelessField::Endpoint => w.endpoint_input.pop(),
                                WirelessField::PairingCode => w.code_input.pop(),
                            },
                            KeyCode::Enter => {
                                let ep = w.endpoint_input.value.trim().to_string();
                                let code = w.code_input.value.trim().to_string();
                                if !ep.is_empty() {
                                    if code.is_empty() {
                                        let _ = self.cmd_tx.send(WorkerCommand::ConnectWifi {
                                            endpoint: ep,
                                            alias: None,
                                        });
                                    } else {
                                        let _ = self
                                            .cmd_tx
                                            .send(WorkerCommand::PairWifi { endpoint: ep, code });
                                    }
                                }
                            }
                            KeyCode::Char(c) => match w.active_field {
                                WirelessField::Endpoint => w.endpoint_input.push(c),
                                WirelessField::PairingCode => w.code_input.push(c),
                            },
                            _ => {}
                        }
                        self.state.modal = Some(Modal::Wireless(w));
                    }
                }
            }
        }
    }

    fn execute_confirmed_action(&mut self, action: PendingAction) {
        let Some(serial) = self.state.active_serial.clone() else {
            self.state.push_toast("No active device selected", true);
            return;
        };
        match action {
            PendingAction::KillPid { pid, .. } => {
                let _ = self.cmd_tx.send(WorkerCommand::KillPid {
                    serial,
                    pid,
                    use_root: self.state.use_root,
                });
            }
            PendingAction::ForceStopApp(package) => {
                let _ = self
                    .cmd_tx
                    .send(WorkerCommand::ForceStopApp { serial, package });
            }
            PendingAction::ClearAppData(package) => {
                let _ = self
                    .cmd_tx
                    .send(WorkerCommand::ClearAppData { serial, package });
            }
            PendingAction::PushFile { local, remote } => {
                let _ = self.cmd_tx.send(WorkerCommand::PushPath {
                    serial,
                    local,
                    remote,
                });
            }
            PendingAction::Reboot(mode) => {
                let _ = self.cmd_tx.send(WorkerCommand::Reboot { serial, mode });
            }
        }
    }

    fn execute_export(&mut self, choice: ExportChoice) {
        let dir = self.state.config.resolved_export_dir();
        let stamp = Local::now().format("%Y%m%d_%H%M%S");
        let serial_safe = self
            .state
            .active_serial
            .as_deref()
            .unwrap_or("device")
            .replace([':', '/'], "_");

        match choice {
            ExportChoice::FullSnapshotJson => {
                let out = dir.join(format!("adbsight_{serial_safe}_{stamp}.json"));
                if let Some(serial) = self.state.active_serial.clone() {
                    let _ = self.cmd_tx.send(WorkerCommand::ExportFull {
                        serial,
                        use_root: self.state.use_root,
                        snapshot_override: Box::new(self.state.live_snapshot_cache.clone()),
                        format: FullExportFormat::Json,
                        out_path: out,
                    });
                }
            }
            ExportChoice::FullMarkdownReport => {
                let out = dir.join(format!("adbsight_{serial_safe}_{stamp}.md"));
                if let Some(serial) = self.state.active_serial.clone() {
                    let _ = self.cmd_tx.send(WorkerCommand::ExportFull {
                        serial,
                        use_root: self.state.use_root,
                        snapshot_override: Box::new(self.state.live_snapshot_cache.clone()),
                        format: FullExportFormat::Markdown,
                        out_path: out,
                    });
                }
            }
            ExportChoice::ActiveTabJson => {
                let out = dir.join(format!(
                    "{serial_safe}_{}_{stamp}.json",
                    format!("{:?}", self.state.active_tab).to_lowercase()
                ));
                let res = match self.state.active_tab {
                    Tab::Dashboard => export_json(&self.state.dashboard.telemetry, &out),
                    Tab::System => export_json(&self.state.system.props, &out),
                    Tab::Processes => export_json(&self.state.processes.processes, &out),
                    Tab::Packages => export_json(&self.state.packages.packages, &out),
                    Tab::Logcat => export_json(&self.state.logcat.entries, &out),
                    Tab::Files => export_json(&self.state.files.entries, &out),
                    Tab::Personal => match self.state.personal.sub_tab {
                        PersonalSubTab::Contacts => {
                            export_json(&self.state.personal.contacts.records, &out)
                        }
                        PersonalSubTab::Sms => export_json(&self.state.personal.sms.records, &out),
                        PersonalSubTab::CallLogs => {
                            export_json(&self.state.personal.call_logs.records, &out)
                        }
                        PersonalSubTab::Accounts => {
                            export_json(&self.state.personal.accounts.records, &out)
                        }
                    },
                    Tab::Shell => export_json(&self.state.shell.output_lines, &out),
                };
                match res {
                    Ok(()) => self
                        .state
                        .push_toast(format!("Exported -> {}", out.display()), false),
                    Err(e) => self.state.push_toast(format!("Export failed: {e}"), true),
                }
            }
            ExportChoice::ActiveTabCsv => {
                let out = dir.join(format!(
                    "{serial_safe}_{}_{stamp}.csv",
                    format!("{:?}", self.state.active_tab).to_lowercase()
                ));
                let res = match self.state.active_tab {
                    Tab::System => export_props_csv(&self.state.system.props, &out),
                    Tab::Processes => export_processes_csv(&self.state.processes.processes, &out),
                    Tab::Packages => export_packages_csv(&self.state.packages.packages, &out),
                    Tab::Logcat => {
                        let logs: Vec<_> = self.state.logcat.entries.iter().cloned().collect();
                        export_logcat_csv(&logs, &out)
                    }
                    Tab::Personal => match self.state.personal.sub_tab {
                        PersonalSubTab::Contacts => {
                            export_contacts_csv(&self.state.personal.contacts.records, &out)
                        }
                        PersonalSubTab::Sms => {
                            export_sms_csv(&self.state.personal.sms.records, &out)
                        }
                        PersonalSubTab::CallLogs => {
                            export_call_logs_csv(&self.state.personal.call_logs.records, &out)
                        }
                        PersonalSubTab::Accounts => {
                            export_accounts_csv(&self.state.personal.accounts.records, &out)
                        }
                    },
                    _ => export_props_csv(&self.state.system.props, &out),
                };
                match res {
                    Ok(()) => self
                        .state
                        .push_toast(format!("Exported -> {}", out.display()), false),
                    Err(e) => self
                        .state
                        .push_toast(format!("CSV export failed: {e}"), true),
                }
            }
        }
    }

    fn active_filter_input(&self) -> Option<&crate::app::state::TextInput> {
        match self.state.active_tab {
            Tab::System => Some(&self.state.system.filter),
            Tab::Processes => Some(&self.state.processes.filter),
            Tab::Packages => Some(&self.state.packages.filter),
            Tab::Logcat => Some(&self.state.logcat.filter),
            Tab::Files => Some(&self.state.files.filter),
            Tab::Personal => Some(&self.state.personal.filter),
            Tab::Dashboard | Tab::Shell => None,
        }
    }

    fn active_filter_input_mut(&mut self) -> Option<&mut crate::app::state::TextInput> {
        match self.state.active_tab {
            Tab::System => Some(&mut self.state.system.filter),
            Tab::Processes => Some(&mut self.state.processes.filter),
            Tab::Packages => Some(&mut self.state.packages.filter),
            Tab::Logcat => Some(&mut self.state.logcat.filter),
            Tab::Files => Some(&mut self.state.files.filter),
            Tab::Personal => Some(&mut self.state.personal.filter),
            Tab::Dashboard | Tab::Shell => None,
        }
    }

    fn reset_tab_selection(&mut self) {
        match self.state.active_tab {
            Tab::System => self.state.system.selected = 0,
            Tab::Processes => self.state.processes.selected = 0,
            Tab::Packages => self.state.packages.selected = 0,
            Tab::Files => self.state.files.selected = 0,
            Tab::Personal => self.state.personal.selected = 0,
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let len = match self.state.active_tab {
            Tab::System => match self.state.system.sub_tab {
                SystemSubTab::Properties => self.filtered_props_len(),
                SystemSubTab::Settings => self.filtered_settings_len(),
                SystemSubTab::Sensors => self.filtered_sensors_len(),
                SystemSubTab::Dumpsys => self.filtered_dumpsys_services().len(),
            },
            Tab::Processes => match self.state.processes.sub_tab {
                ProcessSubTab::Processes => self.filtered_sorted_processes().len(),
                ProcessSubTab::Services => self.state.processes.services.len(),
            },
            Tab::Packages => self.filtered_packages().len(),
            Tab::Files => self.filtered_files().len(),
            Tab::Personal => match self.state.personal.sub_tab {
                PersonalSubTab::Contacts => self.state.personal.contacts.records.len(),
                PersonalSubTab::Sms => self.state.personal.sms.records.len(),
                PersonalSubTab::CallLogs => self.state.personal.call_logs.records.len(),
                PersonalSubTab::Accounts => self.state.personal.accounts.records.len(),
            },
            _ => 0,
        };

        if len == 0 {
            return;
        }
        let max_idx = len - 1;
        let sel = match self.state.active_tab {
            Tab::System => &mut self.state.system.selected,
            Tab::Processes => &mut self.state.processes.selected,
            Tab::Packages => &mut self.state.packages.selected,
            Tab::Files => &mut self.state.files.selected,
            Tab::Personal => &mut self.state.personal.selected,
            _ => return,
        };
        let next = (*sel as isize + delta).clamp(0, max_idx as isize) as usize;
        *sel = next;
    }

    fn filtered_props_len(&self) -> usize {
        let q = self.state.system.filter.value.to_ascii_lowercase();
        self.state
            .system
            .props
            .iter()
            .filter(|p| {
                q.is_empty()
                    || p.key.to_ascii_lowercase().contains(&q)
                    || p.value.to_ascii_lowercase().contains(&q)
            })
            .count()
    }

    fn filtered_settings_len(&self) -> usize {
        let q = self.state.system.filter.value.to_ascii_lowercase();
        self.state
            .system
            .settings
            .iter()
            .filter(|s| {
                q.is_empty()
                    || s.key.to_ascii_lowercase().contains(&q)
                    || s.value.to_ascii_lowercase().contains(&q)
            })
            .count()
    }

    fn filtered_sensors_len(&self) -> usize {
        let q = self.state.system.filter.value.to_ascii_lowercase();
        self.state
            .system
            .sensors
            .iter()
            .filter(|s| q.is_empty() || s.name.to_ascii_lowercase().contains(&q))
            .count()
    }

    fn filtered_dumpsys_services(&self) -> Vec<String> {
        let q = self.state.system.filter.value.to_ascii_lowercase();
        self.state
            .system
            .dumpsys_list
            .iter()
            .filter(|s| q.is_empty() || s.to_ascii_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    pub fn filtered_sorted_processes(&self) -> Vec<&crate::models::ProcessEntry> {
        let q = self.state.processes.filter.value.to_ascii_lowercase();
        let mut list: Vec<_> = self
            .state
            .processes
            .processes
            .iter()
            .filter(|p| {
                q.is_empty()
                    || p.name.to_ascii_lowercase().contains(&q)
                    || p.user.to_ascii_lowercase().contains(&q)
                    || p.pid.to_string().contains(&q)
            })
            .collect();
        match self.state.processes.sort {
            crate::app::state::ProcessSort::CpuDesc => list.sort_by(|a, b| {
                b.cpu_pct
                    .partial_cmp(&a.cpu_pct)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            crate::app::state::ProcessSort::RamDesc => {
                list.sort_by_key(|a| std::cmp::Reverse(a.rss_kb))
            }
            crate::app::state::ProcessSort::PidAsc => list.sort_by_key(|a| a.pid),
            crate::app::state::ProcessSort::NameAsc => list.sort_by(|a, b| a.name.cmp(&b.name)),
        }
        list
    }

    pub fn filtered_packages(&self) -> Vec<&crate::models::PackageSummary> {
        let q = self.state.packages.filter.value.to_ascii_lowercase();
        self.state
            .packages
            .packages
            .iter()
            .filter(|p| match self.state.packages.filter_mode {
                PackageFilter::All => true,
                PackageFilter::UserOnly => !p.is_system,
                PackageFilter::SystemOnly => p.is_system,
            })
            .filter(|p| q.is_empty() || p.name.to_ascii_lowercase().contains(&q))
            .collect()
    }

    fn selected_package(&self) -> Option<&crate::models::PackageSummary> {
        let list = self.filtered_packages();
        list.get(self.state.packages.selected).copied()
    }

    pub fn filtered_files(&self) -> Vec<&crate::models::FsEntry> {
        let q = self.state.files.filter.value.to_ascii_lowercase();
        self.state
            .files
            .entries
            .iter()
            .filter(|e| q.is_empty() || e.name.to_ascii_lowercase().contains(&q))
            .collect()
    }

    fn selected_file_entry(&self) -> Option<&crate::models::FsEntry> {
        let list = self.filtered_files();
        list.get(self.state.files.selected).copied()
    }
}

fn push_history(buf: &mut std::collections::VecDeque<u64>, val: u64, cap: usize) {
    if buf.len() >= cap {
        buf.pop_front();
    }
    buf.push_back(val);
}

fn parent_android_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".to_string();
    }
    match trimmed.rsplit_once('/') {
        Some(("", _)) | None => "/".to_string(),
        Some((parent, _)) => parent.to_string(),
    }
}
