use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::models::{
    AccountRecord, CallLogRecord, ContactRecord, FullDeviceSnapshot, SmsRecord, SnapshotMeta,
};

pub struct SnapshotDb {
    conn: Connection,
}

impl SnapshotDb {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("failed to open SQLite database at {}", path.display()))?;
        conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;

            CREATE TABLE IF NOT EXISTS devices (
                serial TEXT PRIMARY KEY,
                model TEXT NOT NULL,
                brand TEXT NOT NULL,
                manufacturer TEXT NOT NULL,
                android_version TEXT NOT NULL,
                sdk_int TEXT NOT NULL,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_serial TEXT NOT NULL,
                model TEXT NOT NULL,
                android_version TEXT NOT NULL,
                label TEXT NOT NULL,
                taken_at TEXT NOT NULL,
                root_used INTEGER NOT NULL,
                payload_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS contacts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_serial TEXT NOT NULL,
                name TEXT NOT NULL,
                phones_json TEXT NOT NULL,
                emails_json TEXT NOT NULL,
                UNIQUE(device_serial, name, phones_json)
            );

            CREATE TABLE IF NOT EXISTS sms_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_serial TEXT NOT NULL,
                address TEXT NOT NULL,
                body TEXT NOT NULL,
                timestamp_ms INTEGER,
                date_display TEXT NOT NULL,
                msg_type TEXT NOT NULL,
                UNIQUE(device_serial, address, body, date_display)
            );

            CREATE TABLE IF NOT EXISTS call_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_serial TEXT NOT NULL,
                number TEXT NOT NULL,
                call_type TEXT NOT NULL,
                timestamp_ms INTEGER,
                date_display TEXT NOT NULL,
                duration_secs INTEGER NOT NULL,
                UNIQUE(device_serial, number, date_display, duration_secs)
            );

            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_serial TEXT NOT NULL,
                account_type TEXT NOT NULL,
                name TEXT NOT NULL,
                UNIQUE(device_serial, account_type, name)
            );
            ",
        )?;

        Ok(Self { conn })
    }

    pub fn save_full_snapshot(
        &mut self,
        snapshot: &FullDeviceSnapshot,
        label: &str,
    ) -> Result<i64> {
        let now_str = snapshot.taken_at.to_rfc3339();
        let serial = &snapshot.identity.serial;
        let payload_json = serde_json::to_string(snapshot)?;

        self.conn.execute(
            "
            INSERT INTO devices (serial, model, brand, manufacturer, android_version, sdk_int, first_seen, last_seen)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
            ON CONFLICT(serial) DO UPDATE SET
                model = excluded.model,
                brand = excluded.brand,
                manufacturer = excluded.manufacturer,
                android_version = excluded.android_version,
                sdk_int = excluded.sdk_int,
                last_seen = excluded.last_seen
            ",
            params![
                serial,
                snapshot.identity.model,
                snapshot.identity.brand,
                snapshot.identity.manufacturer,
                snapshot.identity.android_version,
                snapshot.identity.sdk_int,
                now_str,
            ],
        )?;

        self.conn.execute(
            "
            INSERT INTO snapshots (device_serial, model, android_version, label, taken_at, root_used, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                serial,
                snapshot.identity.model,
                snapshot.identity.android_version,
                label,
                now_str,
                i32::from(snapshot.root_used),
                payload_json,
            ],
        )?;
        let snapshot_id = self.conn.last_insert_rowid();

        self.upsert_personal_data(
            serial,
            &snapshot.contacts.records,
            &snapshot.sms.records,
            &snapshot.call_logs.records,
            &snapshot.accounts.records,
        )?;

        Ok(snapshot_id)
    }

    pub fn upsert_personal_data(
        &mut self,
        serial: &str,
        contacts: &[ContactRecord],
        sms: &[SmsRecord],
        call_logs: &[CallLogRecord],
        accounts: &[AccountRecord],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        for c in contacts {
            let phones = serde_json::to_string(&c.phone_numbers)?;
            let emails = serde_json::to_string(&c.emails)?;
            tx.execute(
                "INSERT OR IGNORE INTO contacts (device_serial, name, phones_json, emails_json) VALUES (?1, ?2, ?3, ?4)",
                params![serial, c.name, phones, emails],
            )?;
        }

        for s in sms {
            tx.execute(
                "INSERT OR IGNORE INTO sms_messages (device_serial, address, body, timestamp_ms, date_display, msg_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![serial, s.address, s.body, s.timestamp_ms, s.date_display, s.msg_type],
            )?;
        }

        for cl in call_logs {
            tx.execute(
                "INSERT OR IGNORE INTO call_logs (device_serial, number, call_type, timestamp_ms, date_display, duration_secs) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![serial, cl.number, cl.call_type, cl.timestamp_ms, cl.date_display, cl.duration_secs as i64],
            )?;
        }

        for a in accounts {
            tx.execute(
                "INSERT OR IGNORE INTO accounts (device_serial, account_type, name) VALUES (?1, ?2, ?3)",
                params![serial, a.account_type, a.name],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn list_snapshots(&self) -> Result<Vec<SnapshotMeta>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, device_serial, model, android_version, label, taken_at, root_used FROM snapshots ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let taken_raw: String = row.get(5)?;
            let taken_at = DateTime::parse_from_rfc3339(&taken_raw)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            Ok(SnapshotMeta {
                id: row.get(0)?,
                device_serial: row.get(1)?,
                model: row.get(2)?,
                android_version: row.get(3)?,
                label: row.get(4)?,
                taken_at,
                root_used: row.get::<_, i32>(6)? != 0,
            })
        })?;

        let mut list = Vec::new();
        for item in rows {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn load_snapshot(&self, id: i64) -> Result<FullDeviceSnapshot> {
        let json_str: String = self.conn.query_row(
            "SELECT payload_json FROM snapshots WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        let snap = serde_json::from_str(&json_str)?;
        Ok(snap)
    }

    pub fn delete_snapshot(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM snapshots WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adb::demo_backend::DemoAdb;
    use crate::adb::AdbClient;

    #[tokio::test]
    async fn saves_lists_loads_and_deduplicates_snapshots() {
        let tmp_dir = std::env::temp_dir().join(format!("adbsight_db_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);
        let db_path = tmp_dir.join("test.db");

        let client = AdbClient::Demo(DemoAdb::new());
        let snap = client
            .collect_full_snapshot("demo-pixel9pro-wifi", true)
            .await
            .expect("demo snapshot");

        let mut db = SnapshotDb::open(&db_path).expect("open db");
        let id = db
            .save_full_snapshot(&snap, "initial-test")
            .expect("save snapshot");
        // Save second time to verify personal-data deduplication doesn't error or duplicate
        let _id2 = db
            .save_full_snapshot(&snap, "second-test")
            .expect("save second snapshot");

        let list = db.list_snapshots().expect("list snapshots");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].label, "second-test");

        let loaded = db.load_snapshot(id).expect("load snapshot");
        assert_eq!(loaded.identity.model, "Pixel 9 Pro");

        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM contacts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count as usize, snap.contacts.records.len());

        db.delete_snapshot(id).expect("delete snapshot");
        assert_eq!(db.list_snapshots().unwrap().len(), 1);

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
