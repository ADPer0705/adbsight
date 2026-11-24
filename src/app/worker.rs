use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

use crate::adb::AdbClient;
use crate::adb::LogcatHandle;
use crate::app::events::{AppEvent, FullExportFormat, WorkerCommand};
use crate::models::{MdnsKind, SnapshotMeta};
use crate::storage::db::SnapshotDb;
use crate::storage::export::{export_json, export_markdown_report};

pub struct WorkerPool {
    client: AdbClient,
    db: Arc<Mutex<SnapshotDb>>,
    tx: mpsc::UnboundedSender<AppEvent>,
    qr_watch_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl WorkerPool {
    pub fn spawn(
        client: AdbClient,
        db_path: PathBuf,
        tx: mpsc::UnboundedSender<AppEvent>,
    ) -> anyhow::Result<mpsc::UnboundedSender<WorkerCommand>> {
        let db = SnapshotDb::open(&db_path)?;
        let pool = Arc::new(Self {
            client,
            db: Arc::new(Mutex::new(db)),
            tx,
            qr_watch_task: Arc::new(Mutex::new(None)),
        });

        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<WorkerCommand>();
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                let pool_ref = Arc::clone(&pool);
                tokio::spawn(async move {
                    pool_ref.handle_command(cmd).await;
                });
            }
        });

        Ok(cmd_tx)
    }

    pub async fn start_logcat_stream(
        client: &AdbClient,
        serial: &str,
        tx: &mpsc::UnboundedSender<AppEvent>,
        slot: &Arc<Mutex<Option<LogcatHandle>>>,
    ) {
        if let Some(old) = slot.lock().await.take() {
            old.stop();
        }
        let (raw_tx, mut raw_rx) = mpsc::unbounded_channel();
        match client.spawn_logcat(serial, raw_tx).await {
            Ok(handle) => {
                *slot.lock().await = Some(handle);
                let ui_tx = tx.clone();
                tokio::spawn(async move {
                    let mut batch = Vec::with_capacity(64);
                    while let Some(entry) = raw_rx.recv().await {
                        batch.push(entry);
                        while batch.len() < 128 {
                            match raw_rx.try_recv() {
                                Ok(more) => batch.push(more),
                                Err(_) => break,
                            }
                        }
                        if ui_tx
                            .send(AppEvent::LogcatBatch(std::mem::take(&mut batch)))
                            .is_err()
                        {
                            break;
                        }
                    }
                });
            }
            Err(e) => {
                let _ = tx.send(AppEvent::ActionToast {
                    ok: false,
                    message: format!("Logcat stream failed: {e}"),
                    refresh_after: false,
                });
            }
        }
    }

    async fn handle_command(self: Arc<Self>, cmd: WorkerCommand) {
        let client = &self.client;
        let tx = &self.tx;

        match cmd {
            WorkerCommand::RefreshDevices => {
                let res = client.list_devices().await.map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::DevicesListed(res));
            }
            WorkerCommand::RefreshMdns => {
                let res = client.list_mdns_services().await.map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::MdnsListed(res));
            }
            WorkerCommand::PairWifi { endpoint, code } => {
                match client.pair(&endpoint, &code).await {
                    Ok(msg) => {
                        let _ = tx.send(AppEvent::WirelessActionDone {
                            message: msg,
                            connected_endpoint: None,
                            refresh_devices: true,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::WirelessActionFailed(e.to_string()));
                    }
                }
            }
            WorkerCommand::ConnectWifi { endpoint, alias: _ } => {
                match client.connect(&endpoint).await {
                    Ok(msg) => {
                        let _ = tx.send(AppEvent::WirelessActionDone {
                            message: msg,
                            connected_endpoint: Some(endpoint),
                            refresh_devices: true,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::WirelessActionFailed(e.to_string()));
                    }
                }
            }
            WorkerCommand::DisconnectWifi { endpoint } => {
                match client.disconnect(&endpoint).await {
                    Ok(msg) => {
                        let _ = tx.send(AppEvent::WirelessActionDone {
                            message: msg,
                            connected_endpoint: None,
                            refresh_devices: true,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::WirelessActionFailed(e.to_string()));
                    }
                }
            }
            WorkerCommand::UsbToWifi { usb_serial } => {
                match client.enable_tcpip_and_connect(&usb_serial).await {
                    Ok(msg) => {
                        let ep = msg.split_whitespace().next().map(ToString::to_string);
                        let _ = tx.send(AppEvent::WirelessActionDone {
                            message: format!("Wi-Fi ADB ready: {msg}"),
                            connected_endpoint: ep,
                            refresh_devices: true,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::WirelessActionFailed(e.to_string()));
                    }
                }
            }
            WorkerCommand::WatchQrPairing {
                service_name,
                password,
            } => {
                let mut guard = self.qr_watch_task.lock().await;
                if let Some(old) = guard.take() {
                    old.abort();
                }
                let c = client.clone();
                let t = tx.clone();
                let handle = tokio::spawn(async move {
                    let _ = t.send(AppEvent::QrPairingStatus(format!(
                        "Listening via mDNS for '{service_name}'..."
                    )));
                    for _ in 0..90 {
                        if let Ok(services) = c.list_mdns_services().await
                            && let Some(pair_svc) = services
                                .iter()
                                .find(|s| s.kind == MdnsKind::Pairing && s.name == service_name)
                        {
                            let pair_ep = format!("{}:{}", pair_svc.address, pair_svc.port);
                                let _ = t.send(AppEvent::QrPairingStatus(format!(
                                    "Discovered {pair_ep}, pairing..."
                                )));
                                match c.pair(&pair_ep, &password).await {
                                    Ok(msg) => {
                                        let _ = t.send(AppEvent::QrPairingStatus(format!(
                                            "Paired ({msg}) — searching connect port..."
                                        )));
                                        // Look for _adb-tls-connect on the same IP
                                        for _ in 0..15 {
                                            if let Ok(svcs2) = c.list_mdns_services().await
                                                && let Some(conn) = svcs2.iter().find(|s| {
                                                    s.kind == MdnsKind::Connect
                                                        && s.address == pair_svc.address
                                                })
                                            {
                                                let conn_ep = format!("{}:{}", conn.address, conn.port);
                                                match c.connect(&conn_ep).await {
                                                    Ok(cmsg) => {
                                                        let _ = t.send(AppEvent::WirelessActionDone {
                                                            message: format!("QR paired & connected: {conn_ep} ({cmsg})"),
                                                            connected_endpoint: Some(conn_ep),
                                                            refresh_devices: true,
                                                        });
                                                        return;
                                                    }
                                                    Err(e) => {
                                                        let _ = t.send(AppEvent::WirelessActionFailed(e.to_string()));
                                                        return;
                                                    }
                                                }
                                            }
                                            tokio::time::sleep(Duration::from_secs(1)).await;
                                        }
                                        let _ = t.send(AppEvent::WirelessActionDone {
                                            message: format!("Paired to {pair_ep}; connect manually or pick from mDNS list."),
                                            connected_endpoint: None,
                                            refresh_devices: true,
                                        });
                                        return;
                                    }
                                    Err(e) => {
                                        let _ = t.send(AppEvent::WirelessActionFailed(format!(
                                            "QR pairing failed: {e}"
                                        )));
                                        return;
                                    }
                                }
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    let _ = t.send(AppEvent::QrPairingStatus(
                        "QR pairing timed out (90s). Press [g] to generate a fresh QR.".into(),
                    ));
                });
                *guard = Some(handle);
            }
            WorkerCommand::CheckRoot { serial } => {
                let status = client.check_root(&serial).await.unwrap_or_default();
                let _ = tx.send(AppEvent::RootChecked { serial, status });
            }
            WorkerCommand::FetchIdentity { serial } => {
                let result = client.fetch_identity(&serial).await.map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::IdentityLoaded { serial, result });
            }
            WorkerCommand::FetchTelemetry { serial } => {
                let result = client.fetch_telemetry(&serial).await.map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::TelemetryLoaded { serial, result });
            }
            WorkerCommand::FetchSystemAll { serial } => {
                let (props, settings, sensors, dumpsys_list) = tokio::join!(
                    client.fetch_props(&serial),
                    client.fetch_settings(&serial),
                    client.fetch_sensors(&serial),
                    client.fetch_dumpsys_list(&serial),
                );
                let _ = tx.send(AppEvent::SystemAllLoaded {
                    serial,
                    props: props.map_err(|e| e.to_string()),
                    settings: settings.map_err(|e| e.to_string()),
                    sensors: sensors.map_err(|e| e.to_string()),
                    dumpsys_list: dumpsys_list.map_err(|e| e.to_string()),
                });
            }
            WorkerCommand::FetchDumpsysService {
                serial,
                service,
                use_root,
            } => {
                let result = client
                    .fetch_dumpsys_service(&serial, &service, use_root)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::DumpsysServiceLoaded { service, result });
            }
            WorkerCommand::FetchProcesses { serial, use_root } => {
                let (procs, services) = tokio::join!(
                    client.fetch_processes(&serial, use_root),
                    client.fetch_services(&serial, use_root),
                );
                let _ = tx.send(AppEvent::ProcessesLoaded {
                    serial,
                    procs: procs.map_err(|e| e.to_string()),
                    services: services.map_err(|e| e.to_string()),
                });
            }
            WorkerCommand::FetchPackages { serial } => {
                let result = client.fetch_packages(&serial).await.map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::PackagesLoaded { serial, result });
            }
            WorkerCommand::FetchPackageDetail { serial, package } => {
                let result = client
                    .fetch_package_detail(&serial, &package)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::PackageDetailLoaded { package, result });
            }
            WorkerCommand::ListDir {
                serial,
                path,
                use_root,
            } => {
                let result = client
                    .list_dir(&serial, &path, use_root)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::DirLoaded { path, result });
            }
            WorkerCommand::PreviewFile {
                serial,
                path,
                use_root,
            } => {
                let result = client
                    .preview_file(&serial, &path, use_root)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::FilePreviewLoaded { path, result });
            }
            WorkerCommand::PullPath {
                serial,
                remote,
                local,
            } => match client.pull_path(&serial, &remote, &local).await {
                Ok(msg) => {
                    let _ = tx.send(AppEvent::ActionToast {
                        ok: true,
                        message: format!("Pulled -> {} ({msg})", local.display()),
                        refresh_after: false,
                    });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ActionToast {
                        ok: false,
                        message: format!("Pull failed: {e}"),
                        refresh_after: false,
                    });
                }
            },
            WorkerCommand::PushPath {
                serial,
                local,
                remote,
            } => match client.push_path(&serial, &local, &remote).await {
                Ok(msg) => {
                    let _ = tx.send(AppEvent::ActionToast {
                        ok: true,
                        message: format!("Pushed -> {remote} ({msg})"),
                        refresh_after: true,
                    });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ActionToast {
                        ok: false,
                        message: format!("Push failed: {e}"),
                        refresh_after: false,
                    });
                }
            },
            WorkerCommand::FetchPersonalAll { serial, use_root } => {
                let (contacts, sms, calls, accounts) = tokio::join!(
                    client.fetch_contacts(&serial, use_root),
                    client.fetch_sms(&serial, use_root),
                    client.fetch_call_logs(&serial, use_root),
                    client.fetch_accounts(&serial, use_root),
                );
                let _ = tx.send(AppEvent::PersonalAllLoaded {
                    serial,
                    contacts: contacts.map_err(|e| e.to_string()),
                    sms: sms.map_err(|e| e.to_string()),
                    calls: calls.map_err(|e| e.to_string()),
                    accounts: accounts.map_err(|e| e.to_string()),
                });
            }
            WorkerCommand::RunShell {
                serial,
                command,
                use_root,
            } => {
                let result = client
                    .shell(&serial, &command, use_root)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::ShellOutput { command, result });
            }
            WorkerCommand::KillPid {
                serial,
                pid,
                use_root,
            } => match client.kill_pid(&serial, pid, use_root).await {
                Ok(_) => {
                    let _ = tx.send(AppEvent::ActionToast {
                        ok: true,
                        message: format!("Sent SIGKILL to PID {pid}"),
                        refresh_after: true,
                    });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ActionToast {
                        ok: false,
                        message: format!("Kill PID {pid} failed: {e}"),
                        refresh_after: false,
                    });
                }
            },
            WorkerCommand::LaunchApp { serial, package } => {
                let res = client.launch_app(&serial, &package).await;
                send_action_toast(tx, res, &format!("Launched {package}"), false);
            }
            WorkerCommand::ForceStopApp { serial, package } => {
                let res = client.force_stop(&serial, &package).await;
                send_action_toast(tx, res, &format!("Force-stopped {package}"), false);
            }
            WorkerCommand::ClearAppData { serial, package } => {
                let res = client.clear_app_data(&serial, &package).await;
                send_action_toast(tx, res, &format!("Cleared data for {package}"), false);
            }
            WorkerCommand::PullApk {
                serial,
                apk_remote,
                local,
            } => {
                let res = client.pull_apk(&serial, &apk_remote, &local).await;
                send_action_toast(
                    tx,
                    res,
                    &format!("APK pulled -> {}", local.display()),
                    false,
                );
            }
            WorkerCommand::Screencap { serial, local } => {
                let res = client.screencap(&serial, &local).await.map(|_| String::new());
                send_action_toast(
                    tx,
                    res,
                    &format!("Screenshot saved -> {}", local.display()),
                    false,
                );
            }
            WorkerCommand::Reboot { serial, mode } => {
                let res = client.reboot(&serial, mode).await;
                send_action_toast(tx, res, &format!("Triggered {}", mode.label()), true);
            }
            WorkerCommand::SendInputText { serial, text } => {
                let res = client.send_input_text(&serial, &text).await;
                send_action_toast(tx, res, "Sent input text to device", false);
            }
            WorkerCommand::SaveFullSnapshot {
                serial,
                use_root,
                label,
            } => {
                let res = async {
                    let snap = client.collect_full_snapshot(&serial, use_root).await?;
                    let mut db = self.db.lock().await;
                    let id = db.save_full_snapshot(&snap, &label)?;
                    anyhow::Ok((id, label))
                }
                .await
                .map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::SnapshotSaved(res));
            }
            WorkerCommand::ListSnapshots => {
                let db = self.db.lock().await;
                let res = db.list_snapshots().map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::SnapshotsListed(res));
            }
            WorkerCommand::LoadSnapshot { id } => {
                let db = self.db.lock().await;
                let res = (|| -> anyhow::Result<(SnapshotMeta, crate::models::FullDeviceSnapshot)> {
                    let list = db.list_snapshots()?;
                    let meta = list
                        .into_iter()
                        .find(|m| m.id == id)
                        .ok_or_else(|| anyhow::anyhow!("Snapshot #{id} not found"))?;
                    let snap = db.load_snapshot(id)?;
                    Ok((meta, snap))
                })()
                .map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::SnapshotLoaded(Box::new(res)));
            }
            WorkerCommand::DeleteSnapshot { id } => {
                let mut err = None;
                {
                    let db = self.db.lock().await;
                    if let Err(e) = db.delete_snapshot(id) {
                        err = Some(e.to_string());
                    }
                }
                if let Some(e) = err {
                    let _ = tx.send(AppEvent::ActionToast {
                        ok: false,
                        message: format!("Delete failed: {e}"),
                        refresh_after: false,
                    });
                } else {
                    let db = self.db.lock().await;
                    let res = db.list_snapshots().map_err(|e| e.to_string());
                    let _ = tx.send(AppEvent::SnapshotsListed(res));
                }
            }
            WorkerCommand::ExportFull {
                serial,
                use_root,
                snapshot_override,
                format,
                out_path,
            } => {
                let res = async {
                    let snap = match *snapshot_override {
                        Some(s) => s,
                        None => client.collect_full_snapshot(&serial, use_root).await?,
                    };
                    match format {
                        FullExportFormat::Json => export_json(&snap, &out_path)?,
                        FullExportFormat::Markdown => export_markdown_report(&snap, &out_path)?,
                    }
                    anyhow::Ok(format!("Exported -> {}", out_path.display()))
                }
                .await;
                match res {
                    Ok(msg) => {
                        let _ = tx.send(AppEvent::ActionToast {
                            ok: true,
                            message: msg,
                            refresh_after: false,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::ActionToast {
                            ok: false,
                            message: format!("Export failed: {e}"),
                            refresh_after: false,
                        });
                    }
                }
            }
        }
    }

    pub async fn ensure_logcat(
        client: &AdbClient,
        serial: &str,
        tx: &mpsc::UnboundedSender<AppEvent>,
        handle_slot: &Arc<Mutex<Option<LogcatHandle>>>,
    ) {
        Self::start_logcat_stream(client, serial, tx, handle_slot).await;
    }
}

fn send_action_toast(
    tx: &mpsc::UnboundedSender<AppEvent>,
    res: anyhow::Result<String>,
    ok_msg: &str,
    refresh_after: bool,
) {
    match res {
        Ok(_) => {
            let _ = tx.send(AppEvent::ActionToast {
                ok: true,
                message: ok_msg.to_string(),
                refresh_after,
            });
        }
        Err(e) => {
            let _ = tx.send(AppEvent::ActionToast {
                ok: false,
                message: format!("{ok_msg} failed: {e}"),
                refresh_after: false,
            });
        }
    }
}
