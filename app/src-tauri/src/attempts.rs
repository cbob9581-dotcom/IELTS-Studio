use crate::{
    contracts::{now_ms, valid_id, AttemptContract, DesktopError, Result},
    papers,
    storage::Store,
};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
pub fn load(s: &Store, id: &str) -> Result<Value> {
    let raw: String =
        s.db.query_row("SELECT snapshot FROM attempts WHERE id=?", [id], |r| {
            r.get(0)
        })?;
    Ok(serde_json::from_str(&raw)?)
}
pub fn list(s: &Store) -> Result<Vec<Value>> {
    let mut stmt =
        s.db.prepare("SELECT snapshot FROM attempts ORDER BY updated_at DESC")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    rows.map(|r| Ok(serde_json::from_str(&r?)?)).collect()
}
fn conflict() -> DesktopError {
    DesktopError::new("revision_conflict", "记录已更新，请重新加载后继续")
}
pub fn save(s: &Store, mut snapshot: Value, expected: u64) -> Result<Value> {
    let c: AttemptContract = serde_json::from_value(snapshot.clone())?;
    if !valid_id(&c.id)
        || !valid_id(&c.paper_id)
        || c.revision != expected
        || !["training", "exam"].contains(&c.mode.as_str())
        || !["listening", "reading", "writing", "speaking"].contains(&c.section.as_str())
        || !["room", "answering", "interrupted"].contains(&c.status.as_str())
    {
        return Err(DesktopError::new("invalid_attempt", "作答状态或标识不正确"));
    }
    let p = papers::load(s, &c.paper_id, c.paper_version)?;
    if c.mode == "exam" && p["provenance"]["kind"] == "developer_fixture" {
        return Err(DesktopError::new("not_ready", "开发卷不可进入正式模拟"));
    }
    let section = p["sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["section"] == c.section)
        .ok_or_else(|| DesktopError::new("invalid_attempt", "卷中没有此科目"))?;
    for r in &c.responses {
        if !section["questions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|q| q["id"] == r["questionId"])
        {
            return Err(DesktopError::new("invalid_attempt", "响应属于卷外题目"));
        }
    }
    if c.clock["remainingMs"].as_u64().is_none() || c.clock["durationMs"].as_u64().is_none() {
        return Err(DesktopError::new("invalid_attempt", "计时状态无效"));
    }
    let tx = s.db.unchecked_transaction()?;
    let old: Option<(u64, String, String, u64, String, String)> = tx
        .query_row(
            "SELECT revision,status,paper_id,paper_version,section,mode FROM attempts WHERE id=?",
            [&c.id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        )
        .optional()?;
    if let Some((revision, status, pid, pv, sec, mode)) = old {
        if revision != expected {
            return Err(conflict());
        }
        if status == "submitted"
            || pid != c.paper_id
            || pv != c.paper_version
            || sec != c.section
            || mode != c.mode
        {
            return Err(DesktopError::new(
                "immutable_attempt",
                "已提交记录和绑定的卷版本不能修改",
            ));
        }
    } else if expected != 0 {
        return Err(conflict());
    }
    snapshot["revision"] = json!(expected + 1);
    snapshot["updatedAt"] = json!(now_ms());
    tx.execute("INSERT INTO attempts(id,paper_id,paper_version,section,mode,status,revision,snapshot,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET status=excluded.status,revision=excluded.revision,snapshot=excluded.snapshot,updated_at=excluded.updated_at",params![c.id,c.paper_id,c.paper_version,c.section,c.mode,c.status,expected+1,serde_json::to_string(&snapshot)?,now_ms()])?;
    for table in ["responses", "annotations", "interruption_events"] {
        tx.execute(&format!("DELETE FROM {table} WHERE attempt_id=?"), [&c.id])?;
    }
    for r in &c.responses {
        tx.execute(
            "INSERT INTO responses VALUES(?,?,?)",
            params![
                c.id,
                r["questionId"].as_str().unwrap_or(""),
                serde_json::to_string(&r["value"])?
            ],
        )?;
    }
    for (table, values) in [
        ("annotations", &c.annotations),
        ("interruption_events", &c.interruption_events),
    ] {
        for (i, v) in values.iter().enumerate() {
            tx.execute(
                &format!("INSERT INTO {table} VALUES(?,?,?)"),
                params![c.id, i as u64, serde_json::to_string(v)?],
            )?;
        }
    }
    if let Some(run) = snapshot["examRunId"].as_str() {
        if !valid_id(run) {
            return Err(DesktopError::new("invalid_attempt", "examRunId 无效"));
        }
        tx.execute(
            "INSERT OR IGNORE INTO exam_runs VALUES(?,?,?,'in_progress')",
            params![run, c.paper_id, c.paper_version],
        )?;
    }
    tx.commit()?;
    Ok(snapshot)
}
pub fn submit(s: &Store, id: &str, expected: u64) -> Result<Value> {
    let tx = s.db.unchecked_transaction()?;
    let raw: String = tx.query_row("SELECT snapshot FROM attempts WHERE id=?", [id], |r| {
        r.get(0)
    })?;
    let mut a: Value = serde_json::from_str(&raw)?;
    if a["status"] == "submitted" {
        return Ok(a);
    }
    if a["revision"].as_u64() != Some(expected) {
        return Err(conflict());
    }
    a["status"] = json!("submitted");
    a["revision"] = json!(expected + 1);
    a["submittedAt"] = json!(now_ms());
    tx.execute(
        "UPDATE attempts SET status='submitted',revision=?,snapshot=?,updated_at=? WHERE id=?",
        params![expected + 1, serde_json::to_string(&a)?, now_ms(), id],
    )?;
    tx.commit()?;
    Ok(a)
}
