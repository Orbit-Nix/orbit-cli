// OrbitOS — Antigravity IDE conversation history and state.vscdb sync

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use rusqlite::Connection;
use uuid::Uuid;

use crate::proto::{
    encode_field_bytes, encode_field_varint, make_timestamp, make_ws_info, parse_proto,
    ProtoFieldsExt,
};
use crate::util::{get_target_user, print_success};

// Synchronize all local conversation history into Antigravity IDE SQLite state DB
pub fn sync_chats(custom_home: Option<&Path>) -> Result<usize> {
    let user_info = get_target_user();
    let home_dir = custom_home
        .map(|p| p.to_path_buf())
        .unwrap_or(user_info.home_dir);

    let db_path = home_dir.join(".config/Antigravity IDE/User/globalStorage/state.vscdb");
    let backup_path = home_dir.join(".config/Antigravity IDE/User/globalStorage/state.vscdb.backup");
    let conv_dir = home_dir.join(".gemini/antigravity-ide/conversations");
    let brain_dir = home_dir.join(".gemini/antigravity-ide/brain");

    if !conv_dir.exists() || !db_path.exists() {
        // Nothing to sync
        return Ok(0);
    }

    // Create database backup
    let _ = fs::copy(&db_path, &backup_path);

    let mut dbs: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&conv_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("db") {
                dbs.push(path);
            }
        }
    }

    // Sort conversations by modified time descending
    dbs.sort_by(|a, b| {
        let ma = a.metadata().and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        let mb = b.metadata().and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        mb.cmp(&ma)
    });

    let mut top_proto = Vec::new();
    let mut count = 0;

    for db in &dbs {
        let cid = match db.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        let mtime_secs = db
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut ws_bytes: Option<Vec<u8>> = None;
        let mut created_ts_bytes: Option<Vec<u8>> = None;
        let mut session_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, cid.as_bytes()).to_string();
        let mut uri = "file:///etc/nixos".to_string();

        if let Ok(conn) = Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) {
            let stmt = conn.prepare("SELECT data FROM trajectory_metadata_blob WHERE id=\"main\"");
            if let Ok(mut stmt) = stmt {
                let rows = stmt.query([]);
                if let Ok(mut rows) = rows {
                    if let Ok(Some(row)) = rows.next() {
                        if let Ok(blob) = row.get::<_, Vec<u8>>(0) {
                            let parsed = parse_proto(&blob);
                            if let Some(b) = parsed.get_first_bytes(1) {
                                ws_bytes = Some(b.to_vec());
                            }
                            if let Some(b) = parsed.get_first_bytes(2) {
                                created_ts_bytes = Some(b.to_vec());
                            }
                            if let Some(s) = parsed.get_first_string(3) {
                                session_id = s;
                            }
                            if let Some(s) = parsed.get_first_string(7) {
                                uri = s;
                            }
                        }
                    }
                }
            }
        }

        let ws_bytes = ws_bytes.unwrap_or_else(|| {
            make_ws_info(&uri, "Orbit-Nix/orbit-config", "git@github.com:Orbit-Nix/orbit-config.git", "main")
        });
        let created_ts_bytes = created_ts_bytes.unwrap_or_else(|| make_timestamp(mtime_secs, 0));

        let mut title: Option<String> = None;
        let t_file = brain_dir
            .join(&cid)
            .join(".system_generated/logs/transcript.jsonl");

        if t_file.is_file() {
            if let Ok(content) = fs::read_to_string(&t_file) {
                for line in content.lines() {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                        if val.get("type").and_then(|v| v.as_str()) == Some("USER_INPUT") {
                            let c = val.get("content").and_then(|v| v.as_str()).unwrap_or("");
                            let text = if c.contains("<USER_REQUEST>") && c.contains("</USER_REQUEST>") {
                                c.split("<USER_REQUEST>")
                                    .nth(1)
                                    .and_then(|part| part.split("</USER_REQUEST>").next())
                                    .unwrap_or(c)
                                    .trim()
                            } else {
                                c.trim()
                            };
                            let first_line = text.lines().next().unwrap_or("").trim();
                            let truncated: String = first_line.chars().take(60).collect();
                            if !truncated.is_empty() {
                                title = Some(truncated);
                            }
                            break;
                        }
                    }
                }
            }
        }

        let title = title.unwrap_or_else(|| {
            let short = if cid.len() >= 8 { &cid[..8] } else { &cid };
            format!("Conversation {}", short)
        });

        let mut sub17 = Vec::new();
        sub17.extend(encode_field_bytes(1, &ws_bytes));
        sub17.extend(encode_field_bytes(2, &created_ts_bytes));
        sub17.extend(encode_field_bytes(3, session_id.as_bytes()));
        sub17.extend(encode_field_bytes(6, cid.as_bytes()));
        sub17.extend(encode_field_bytes(7, uri.as_bytes()));

        let modified_ts_bytes = make_timestamp(mtime_secs, 0);
        let summary_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, cid.as_bytes()).to_string();

        let mut summary = Vec::new();
        summary.extend(encode_field_bytes(1, title.as_bytes()));
        summary.extend(encode_field_varint(2, 207));
        summary.extend(encode_field_bytes(3, &modified_ts_bytes));
        summary.extend(encode_field_bytes(4, summary_id.as_bytes()));
        summary.extend(encode_field_varint(5, 1));
        summary.extend(encode_field_bytes(7, &created_ts_bytes));
        summary.extend(encode_field_bytes(9, &ws_bytes));
        summary.extend(encode_field_bytes(10, &modified_ts_bytes));
        summary.extend(encode_field_bytes(15, &[]));
        summary.extend(encode_field_varint(16, 100));
        summary.extend(encode_field_bytes(17, &sub17));
        summary.extend(encode_field_varint(22, 4));

        let b64_str = BASE64_STANDARD.encode(&summary);
        let submsg = encode_field_bytes(1, b64_str.as_bytes());

        let mut entry = Vec::new();
        entry.extend(encode_field_bytes(1, cid.as_bytes()));
        entry.extend(encode_field_bytes(2, &submsg));

        top_proto.extend(encode_field_bytes(1, &entry));
        count += 1;
    }

    let encoded_b64 = BASE64_STANDARD.encode(&top_proto);

    let conn = Connection::open(&db_path).with_context(|| {
        format!("Failed to open SQLite database: {}", db_path.display())
    })?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT PRIMARY KEY, value TEXT)",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
        rusqlite::params![
            "antigravityUnifiedStateSync.trajectorySummaries",
            encoded_b64
        ],
    )?;
    let _ = conn.execute_batch("PRAGMA wal_checkpoint(FULL);");

    print_success(&format!(
        "Antigravity IDE: Synced {} conversation histories to state.vscdb",
        count
    ));
    Ok(count)
}
