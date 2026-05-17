use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension};

use crate::error::{Result, RsImsgError};
use crate::time::apple_timestamp_to_utc;
use crate::types::{AttachmentMeta, ChatRecord, MessageRecord};

pub struct MessageStore {
    conn: Connection,
}

impl MessageStore {
    pub fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(RsImsgError::Other(format!(
                "chat.db not found at {}",
                path.display()
            )));
        }
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        conn.execute_batch("PRAGMA query_only = ON;")?;
        Ok(Self { conn })
    }

    pub fn max_message_rowid(&self) -> Result<i64> {
        let rowid: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(ROWID), 0) FROM message", [], |r| r.get(0))?;
        Ok(rowid)
    }

    pub fn list_chats(&self, limit: usize) -> Result<Vec<ChatRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                c.ROWID,
                c.display_name,
                c.chat_identifier,
                c.guid,
                c.service_name,
                c.style,
                (
                    SELECT MAX(m.date)
                    FROM chat_message_join cmj
                    JOIN message m ON m.ROWID = cmj.message_id
                    WHERE cmj.chat_id = c.ROWID
                ) AS last_date
            FROM chat c
            ORDER BY last_date DESC NULLS LAST, c.ROWID DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map([limit as i64], |row| {
            let id: i64 = row.get(0)?;
            let display_name: Option<String> = row.get(1)?;
            let identifier: String = row.get(2)?;
            let guid: String = row.get(3)?;
            let service: Option<String> = row.get(4)?;
            let style: Option<i32> = row.get(5)?;
            let last_date: Option<i64> = row.get(6)?;
            Ok((id, display_name, identifier, guid, service, style, last_date))
        })?;

        let mut out = Vec::new();
        for row in rows {
            let (id, display_name, identifier, guid, service, style, last_date) = row?;
            let participants = self.chat_participants(id)?;
            let is_group = style == Some(43) || participants.len() > 2;
            out.push(ChatRecord {
                id,
                name: display_name,
                identifier,
                guid,
                service,
                is_group,
                participants,
                last_message_at: last_date.and_then(apple_timestamp_to_utc),
            });
        }
        Ok(out)
    }

    pub fn chat_participants(&self, chat_id: i64) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT h.id
            FROM chat_handle_join chj
            JOIN handle h ON h.ROWID = chj.handle_id
            WHERE chj.chat_id = ?1
            ORDER BY h.id
            "#,
        )?;
        let handles = stmt
            .query_map([chat_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(handles)
    }

    pub fn history(&self, chat_id: i64, limit: usize, since_rowid: Option<i64>) -> Result<Vec<MessageRecord>> {
        const SELECT: &str = r#"
            SELECT
                m.ROWID, m.guid, c.ROWID, c.chat_identifier, c.guid, c.display_name, c.style,
                h.id, m.is_from_me, m.text, m.date, m.thread_originator_guid
            FROM message m
            JOIN chat_message_join cmj ON cmj.message_id = m.ROWID
            JOIN chat c ON c.ROWID = cmj.chat_id
            LEFT JOIN handle h ON h.ROWID = m.handle_id
            WHERE cmj.chat_id = ?1
        "#;

        let mut messages = Vec::new();
        if let Some(since) = since_rowid {
            let sql = format!("{SELECT} AND m.ROWID > ?2 ORDER BY m.ROWID ASC LIMIT ?3");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map((chat_id, since, limit as i64), Self::map_message_row)?;
            for mut msg in rows.flatten() {
                self.enrich_message(&mut msg)?;
                messages.push(msg);
            }
        } else {
            let sql = format!("{SELECT} ORDER BY m.ROWID DESC LIMIT ?2");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map((chat_id, limit as i64), Self::map_message_row)?;
            let mut batch: Vec<MessageRecord> = rows.flatten().collect();
            batch.reverse();
            for mut msg in batch {
                self.enrich_message(&mut msg)?;
                messages.push(msg);
            }
        }
        Ok(messages)
    }

    fn enrich_message(&self, msg: &mut MessageRecord) -> Result<()> {
        msg.participants = self.chat_participants(msg.chat_id)?;
        msg.is_group = msg.is_group || msg.participants.len() > 2;
        msg.attachments = self.message_attachments(msg.id)?;
        Ok(())
    }

    fn map_message_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageRecord> {
        let id: i64 = row.get(0)?;
        let guid: String = row.get(1)?;
        let chat_rowid: i64 = row.get(2)?;
        let chat_identifier: String = row.get(3)?;
        let chat_guid: String = row.get(4)?;
        let chat_name: Option<String> = row.get(5)?;
        let style: Option<i32> = row.get(6)?;
        let sender: Option<String> = row.get(7)?;
        let is_from_me: bool = row.get::<_, i32>(8)? != 0;
        let text: Option<String> = row.get(9)?;
        let date_raw: i64 = row.get(10)?;
        let reply_to_guid: Option<String> = row.get(11)?;
        let created_at = apple_timestamp_to_utc(date_raw).unwrap_or_else(chrono::Utc::now);
        Ok(MessageRecord {
            id,
            guid,
            chat_id: chat_rowid,
            chat_identifier,
            chat_guid,
            chat_name,
            participants: Vec::new(),
            is_group: style == Some(43),
            sender,
            is_from_me,
            text,
            created_at,
            reply_to_guid,
            attachments: Vec::new(),
        })
    }

    pub fn messages_after_rowid(&self, since_rowid: i64, chat_id: Option<i64>, limit: usize) -> Result<Vec<MessageRecord>> {
        let sql = if chat_id.is_some() {
            r#"
            SELECT
                m.ROWID, m.guid, c.ROWID, c.chat_identifier, c.guid, c.display_name, c.style,
                h.id, m.is_from_me, m.text, m.date, m.thread_originator_guid
            FROM message m
            JOIN chat_message_join cmj ON cmj.message_id = m.ROWID
            JOIN chat c ON c.ROWID = cmj.chat_id
            LEFT JOIN handle h ON h.ROWID = m.handle_id
            WHERE m.ROWID > ?1 AND cmj.chat_id = ?2
            ORDER BY m.ROWID ASC
            LIMIT ?3
            "#
        } else {
            r#"
            SELECT
                m.ROWID, m.guid, c.ROWID, c.chat_identifier, c.guid, c.display_name, c.style,
                h.id, m.is_from_me, m.text, m.date, m.thread_originator_guid
            FROM message m
            JOIN chat_message_join cmj ON cmj.message_id = m.ROWID
            JOIN chat c ON c.ROWID = cmj.chat_id
            LEFT JOIN handle h ON h.ROWID = m.handle_id
            WHERE m.ROWID > ?1
            ORDER BY m.ROWID ASC
            LIMIT ?2
            "#
        };

        let mut stmt = self.conn.prepare(sql)?;
        let mut out = Vec::new();

        let map = |row: &rusqlite::Row<'_>| -> rusqlite::Result<MessageRecord> {
            let id: i64 = row.get(0)?;
            let guid: String = row.get(1)?;
            let chat_rowid: i64 = row.get(2)?;
            let chat_identifier: String = row.get(3)?;
            let chat_guid: String = row.get(4)?;
            let chat_name: Option<String> = row.get(5)?;
            let style: Option<i32> = row.get(6)?;
            let sender: Option<String> = row.get(7)?;
            let is_from_me: bool = row.get::<_, i32>(8)? != 0;
            let text: Option<String> = row.get(9)?;
            let date_raw: i64 = row.get(10)?;
            let reply_to_guid: Option<String> = row.get(11)?;
            let created_at = apple_timestamp_to_utc(date_raw).unwrap_or_else(chrono::Utc::now);
            Ok(MessageRecord {
                id,
                guid,
                chat_id: chat_rowid,
                chat_identifier,
                chat_guid,
                chat_name,
                participants: Vec::new(),
                is_group: style == Some(43),
                sender,
                is_from_me,
                text,
                created_at,
                reply_to_guid,
                attachments: Vec::new(),
            })
        };

        if let Some(cid) = chat_id {
            let rows = stmt.query_map((since_rowid, cid, limit as i64), map)?;
            for mut msg in rows.flatten() {
                self.enrich_message(&mut msg)?;
                out.push(msg);
            }
        } else {
            let rows = stmt.query_map((since_rowid, limit as i64), map)?;
            for mut msg in rows.flatten() {
                self.enrich_message(&mut msg)?;
                out.push(msg);
            }
        }
        Ok(out)
    }

    fn message_attachments(&self, message_id: i64) -> Result<Vec<AttachmentMeta>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT a.filename, a.uti, a.total_bytes, a.missing
            FROM attachment a
            JOIN message_attachment_join maj ON maj.attachment_id = a.ROWID
            WHERE maj.message_id = ?1
            "#,
        )?;
        let rows = stmt.query_map([message_id], |row| {
            let filename: Option<String> = row.get(0)?;
            let uti: Option<String> = row.get(1)?;
            let byte_count: Option<i64> = row.get(2)?;
            let missing: i32 = row.get(3)?;
            Ok(AttachmentMeta {
                filename,
                mime_type: uti,
                byte_count,
                missing: missing != 0,
            })
        })?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MessageRecord>> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                m.ROWID, m.guid, c.ROWID, c.chat_identifier, c.guid, c.display_name, c.style,
                h.id, m.is_from_me, m.text, m.date, m.thread_originator_guid
            FROM message m
            JOIN chat_message_join cmj ON cmj.message_id = m.ROWID
            JOIN chat c ON c.ROWID = cmj.chat_id
            LEFT JOIN handle h ON h.ROWID = m.handle_id
            WHERE m.text LIKE ?1
            ORDER BY m.ROWID DESC
            LIMIT ?2
            "#,
        )?;
        let rows = stmt.query_map((pattern, limit as i64), |row| {
            let id: i64 = row.get(0)?;
            let guid: String = row.get(1)?;
            let chat_rowid: i64 = row.get(2)?;
            let chat_identifier: String = row.get(3)?;
            let chat_guid: String = row.get(4)?;
            let chat_name: Option<String> = row.get(5)?;
            let style: Option<i32> = row.get(6)?;
            let sender: Option<String> = row.get(7)?;
            let is_from_me: bool = row.get::<_, i32>(8)? != 0;
            let text: Option<String> = row.get(9)?;
            let date_raw: i64 = row.get(10)?;
            let reply_to_guid: Option<String> = row.get(11)?;
            let created_at = apple_timestamp_to_utc(date_raw).unwrap_or_else(chrono::Utc::now);
            Ok(MessageRecord {
                id,
                guid,
                chat_id: chat_rowid,
                chat_identifier,
                chat_guid,
                chat_name,
                participants: Vec::new(),
                is_group: style == Some(43),
                sender,
                is_from_me,
                text,
                created_at,
                reply_to_guid,
                attachments: Vec::new(),
            })
        })?;
        let mut out = Vec::new();
        for mut msg in rows.flatten() {
            self.enrich_message(&mut msg)?;
            out.push(msg);
        }
        Ok(out)
    }

    pub fn chat_by_id(&self, chat_id: i64) -> Result<Option<ChatRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT c.ROWID, c.display_name, c.chat_identifier, c.guid, c.service_name, c.style,
                (SELECT MAX(m.date) FROM chat_message_join cmj JOIN message m ON m.ROWID = cmj.message_id WHERE cmj.chat_id = c.ROWID)
            FROM chat c
            WHERE c.ROWID = ?1
            "#,
        )?;
        let row = stmt
            .query_row([chat_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<i32>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                ))
            })
            .optional()?;
        let Some((id, name, identifier, guid, service, style, last_date)) = row else {
            return Ok(None);
        };
        let participants = self.chat_participants(id)?;
        let is_group = style == Some(43) || participants.len() > 2;
        Ok(Some(ChatRecord {
            id,
            name,
            identifier,
            guid,
            service,
            is_group,
            participants,
            last_message_at: last_date.and_then(apple_timestamp_to_utc),
        }))
    }
}
