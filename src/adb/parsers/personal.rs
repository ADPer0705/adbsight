use std::collections::BTreeMap;

use chrono::{DateTime, Local, TimeZone, Utc};
use regex::Regex;

use crate::adb::parsers::parse_content_query_rows;
use crate::models::{AccountRecord, CallLogRecord, ContactRecord, PersonalDataResult, SmsRecord};

pub fn detect_permission_hint(output: &str) -> Option<String> {
    let lower = output.to_ascii_lowercase();
    if lower.contains("securityexception")
        || lower.contains("permission denial")
        || lower.contains("requires android.permission")
        || lower.contains("permission denied")
    {
        Some(
            "Android restricted shell access to this provider (normal on stock Android 10–15+). Enable Root Mode (press 'R') on a rooted device/emulator to unlock."
                .to_string(),
        )
    } else {
        None
    }
}

pub fn parse_contacts(output: &str) -> PersonalDataResult<ContactRecord> {
    if let Some(hint) = detect_permission_hint(output) {
        return PersonalDataResult {
            records: Vec::new(),
            permission_hint: Some(hint),
            raw_error: Some(output.lines().take(5).collect::<Vec<_>>().join(" | ")),
        };
    }

    let rows = parse_content_query_rows(output, &["display_name", "data1", "mimetype"]);
    let mut by_name: BTreeMap<String, ContactRecord> = BTreeMap::new();

    for row in rows {
        let Some(name) = row
            .get("display_name")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let entry = by_name
            .entry(name.to_string())
            .or_insert_with(|| ContactRecord {
                name: name.to_string(),
                phone_numbers: Vec::new(),
                emails: Vec::new(),
            });

        let Some(data1) = row.get("data1").map(|s| s.trim()).filter(|s| !s.is_empty()) else {
            continue;
        };
        let mime = row.get("mimetype").map(String::as_str).unwrap_or("");
        if mime.contains("phone") {
            if !entry.phone_numbers.iter().any(|p| p == data1) {
                entry.phone_numbers.push(data1.to_string());
            }
        } else if mime.contains("email") && !entry.emails.iter().any(|e| e == data1) {
            entry.emails.push(data1.to_string());
        }
    }

    PersonalDataResult {
        records: by_name.into_values().collect(),
        permission_hint: None,
        raw_error: None,
    }
}

pub fn parse_sms(output: &str) -> PersonalDataResult<SmsRecord> {
    if let Some(hint) = detect_permission_hint(output) {
        return PersonalDataResult {
            records: Vec::new(),
            permission_hint: Some(hint),
            raw_error: Some(output.lines().take(5).collect::<Vec<_>>().join(" | ")),
        };
    }

    let rows = parse_content_query_rows(output, &["address", "body", "date", "type"]);
    let mut records = Vec::new();

    for row in rows {
        let address = row
            .get("address")
            .cloned()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "(Unknown)".to_string());
        let body = row
            .get("body")
            .cloned()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "(Empty)".to_string());

        let timestamp_ms = row.get("date").and_then(|d| d.parse::<i64>().ok());
        let date_display = format_millis(timestamp_ms);
        let msg_type = match row.get("type").map(String::as_str) {
            Some("1") => "Inbox",
            Some("2") => "Sent",
            Some("3") => "Draft",
            Some("4") => "Outbox",
            Some("5") => "Failed",
            Some("6") => "Queued",
            _ => "Unknown",
        }
        .to_string();

        records.push(SmsRecord {
            address,
            body,
            timestamp_ms,
            date_display,
            msg_type,
        });
    }

    records.sort_by_key(|a| std::cmp::Reverse(a.timestamp_ms));
    PersonalDataResult {
        records,
        permission_hint: None,
        raw_error: None,
    }
}

pub fn parse_call_logs(output: &str) -> PersonalDataResult<CallLogRecord> {
    if let Some(hint) = detect_permission_hint(output) {
        return PersonalDataResult {
            records: Vec::new(),
            permission_hint: Some(hint),
            raw_error: Some(output.lines().take(5).collect::<Vec<_>>().join(" | ")),
        };
    }

    let rows = parse_content_query_rows(output, &["number", "type", "date", "duration"]);
    let mut records = Vec::new();

    for row in rows {
        let number = row
            .get("number")
            .cloned()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "(Private)".to_string());
        let timestamp_ms = row.get("date").and_then(|d| d.parse::<i64>().ok());
        let date_display = format_millis(timestamp_ms);
        let duration_secs = row
            .get("duration")
            .and_then(|d| d.parse::<u64>().ok())
            .unwrap_or(0);
        let call_type = match row.get("type").map(String::as_str) {
            Some("1") => "Incoming",
            Some("2") => "Outgoing",
            Some("3") => "Missed",
            Some("4") => "Voicemail",
            Some("5") => "Rejected",
            Some("6") => "Blocked",
            _ => "Unknown",
        }
        .to_string();

        records.push(CallLogRecord {
            number,
            call_type,
            timestamp_ms,
            date_display,
            duration_secs,
        });
    }

    records.sort_by_key(|a| std::cmp::Reverse(a.timestamp_ms));
    PersonalDataResult {
        records,
        permission_hint: None,
        raw_error: None,
    }
}

pub fn parse_accounts(output: &str) -> PersonalDataResult<AccountRecord> {
    if let Some(hint) = detect_permission_hint(output) {
        return PersonalDataResult {
            records: Vec::new(),
            permission_hint: Some(hint),
            raw_error: Some(output.lines().take(5).collect::<Vec<_>>().join(" | ")),
        };
    }

    let re = Regex::new(r"Account\s+\{name=([^,]+),\s*type=([^}]+)\}").expect("valid regex");
    let mut records = Vec::new();

    for line in output.lines() {
        if let Some(caps) = re.captures(line) {
            let name = caps[1].trim().to_string();
            let account_type = caps[2].trim().to_string();
            if !records
                .iter()
                .any(|r: &AccountRecord| r.name == name && r.account_type == account_type)
            {
                records.push(AccountRecord { account_type, name });
            }
        }
    }

    PersonalDataResult {
        records,
        permission_hint: None,
        raw_error: None,
    }
}

fn format_millis(ms: Option<i64>) -> String {
    let Some(ms) = ms else {
        return "Unknown".to_string();
    };
    Utc.timestamp_millis_opt(ms)
        .single()
        .map(|dt: DateTime<Utc>| {
            dt.with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| ms.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiline_sms_with_commas() {
        let sample = "\
Row: 0 address=+15551234567, body=Hey, are we still meeting at 5, or later?\nLet me know!, date=1732440000000, type=1
Row: 1 address=+15559876543, body=Sure, sounds good!, date=1732443600000, type=2
";
        let res = parse_sms(sample);
        assert!(res.permission_hint.is_none());
        assert_eq!(res.records.len(), 2);
        // Sorted newest first
        assert_eq!(res.records[0].msg_type, "Sent");
        assert_eq!(res.records[0].body, "Sure, sounds good!");
        assert_eq!(
            res.records[1].body,
            "Hey, are we still meeting at 5, or later?\nLet me know!"
        );
    }

    #[test]
    fn detects_android_permission_denial() {
        let sample = "Error while accessing provider:sms\njava.lang.SecurityException: Permission Denial: reading com.android.providers.telephony.SmsProvider";
        let res = parse_sms(sample);
        assert!(res.records.is_empty());
        assert!(res.permission_hint.is_some());
    }

    #[test]
    fn aggregates_contacts_and_parses_accounts_and_calls() {
        let contacts_raw = "\
Row: 0 display_name=Alice Smith, data1=+15550001111, mimetype=vnd.android.cursor.item/phone_v2
Row: 1 display_name=Alice Smith, data1=+15550002222, mimetype=vnd.android.cursor.item/phone_v2
Row: 2 display_name=Alice Smith, data1=alice@example.com, mimetype=vnd.android.cursor.item/email_v2
";
        let contacts = parse_contacts(contacts_raw);
        assert_eq!(contacts.records.len(), 1);
        assert_eq!(contacts.records[0].phone_numbers.len(), 2);
        assert_eq!(contacts.records[0].emails, vec!["alice@example.com"]);

        let calls_raw = "Row: 0 number=+15550001111, type=3, date=1732440000000, duration=0\n";
        let calls = parse_call_logs(calls_raw);
        assert_eq!(calls.records[0].call_type, "Missed");

        let accounts_raw = "  Account {name=user@gmail.com, type=com.google}\n  Account {name=work@corp.com, type=com.whatsapp}\n";
        let accts = parse_accounts(accounts_raw);
        assert_eq!(accts.records.len(), 2);
    }
}
