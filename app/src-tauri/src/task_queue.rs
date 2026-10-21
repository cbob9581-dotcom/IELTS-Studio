//! Durable, minimal task index. Papers, recordings and provider keys never enter this database.
use crate::contracts::{now_ms, valid_id, DesktopError, Result};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskRequest {
    pub kind: String,
    pub owner: String,
    pub targets: Vec<String>,
    pub title: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub kind: String,
    pub owner: String,
    pub targets: Vec<String>,
    pub completed: Vec<String>,
    pub state: String,
    pub title: String,
    pub error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(default)]
    pub operation: Option<serde_json::Value>,
    #[serde(default)]
    pub usage: Usage,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub requests: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}
pub struct Queue {
    db: Connection,
}
fn err(s: impl ToString) -> DesktopError {
    DesktopError::new("task_queue", s)
}
fn row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let text: String = row.get(0)?;
    serde_json::from_str(&text).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
fn get(db: &Connection, id: &str) -> Result<Task> {
    db.query_row("SELECT data FROM tasks WHERE id=?1", [id], row)
        .optional()?
        .ok_or_else(|| err("后台任务不存在"))
}
fn save(db: &Connection, t: &mut Task) -> Result<()> {
    t.updated_at = now_ms();
    db.execute(
        "UPDATE tasks SET state=?2,updated_at=?3,data=?4 WHERE id=?1",
        params![t.id, t.state, t.updated_at, serde_json::to_string(t)?],
    )?;
    Ok(())
}
impl Queue {
    pub fn open(root: &Path) -> Result<Self> {
        if !root.is_dir() {
            return Err(err("资产目录不可用，后台任务未创建"));
        }
        let db = Connection::open(root.join("task-runtime.sqlite3"))?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
   CREATE TABLE IF NOT EXISTS tasks(id TEXT PRIMARY KEY,kind TEXT NOT NULL,owner TEXT NOT NULL,state TEXT NOT NULL,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL,data TEXT NOT NULL);
   CREATE INDEX IF NOT EXISTS task_active ON tasks(state,created_at);
   CREATE INDEX IF NOT EXISTS task_owner ON tasks(kind,owner,state);
   CREATE TABLE IF NOT EXISTS task_events(id INTEGER PRIMARY KEY,task_id TEXT NOT NULL,at INTEGER NOT NULL,event TEXT NOT NULL);
   CREATE TABLE IF NOT EXISTS paid_requests(attempt_id TEXT PRIMARY KEY,task_id TEXT NOT NULL,model TEXT NOT NULL,at INTEGER NOT NULL,input_tokens INTEGER,output_tokens INTEGER);
   CREATE INDEX IF NOT EXISTS task_requests ON paid_requests(task_id);")?;
        Ok(Self { db })
    }
    pub fn enqueue(&mut self, r: TaskRequest) -> Result<Task> {
        if !["cloud", "source"].contains(&r.kind.as_str())
            || !valid_id(&r.owner)
            || r.targets.is_empty()
            || r.targets.len() > 1000
            || r.targets.iter().any(|s| !valid_id(s))
            || r.targets
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != r.targets.len()
            || r.title.trim().is_empty()
            || r.title.len() > 1000
        {
            return Err(err("任务类型、标识或处理范围无效"));
        }
        if r.kind == "cloud" && r.targets != vec![r.owner.clone()] {
            return Err(err("云任务处理范围必须与任务编号一致"));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut old=tx.query_row("SELECT data FROM tasks WHERE kind=?1 AND owner=?2 AND state IN ('queued','running','pausing','paused') ORDER BY created_at DESC LIMIT 1",params![r.kind,r.owner],row).optional()?;
        if old.is_none() {
            let mut statement=tx.prepare("SELECT data FROM tasks WHERE kind=?1 AND owner=?2 AND state='failed' ORDER BY created_at DESC")?;
            let mut rows = statement.query(params![r.kind, r.owner])?;
            while let Some(item) = rows.next()? {
                let failed = row(item)?;
                if failed.targets == r.targets {
                    old = Some(failed);
                    break;
                }
            }
        }
        if let Some(mut old) = old {
            if old.targets != r.targets {
                return Err(err(
                    "同一资料项目已有不同范围的任务，请先完成或取消已有任务",
                ));
            }
            if ["paused", "failed"].contains(&old.state.as_str()) {
                old.state = "queued".into();
                old.error = None;
                save(&tx, &mut old)?;
            }
            tx.commit()?;
            return self.read(&old.id);
        }
        let at = now_ms();
        let t = Task {
            id: uuid::Uuid::new_v4().to_string(),
            kind: r.kind,
            owner: r.owner,
            targets: r.targets,
            completed: vec![],
            state: "queued".into(),
            title: r.title,
            error: None,
            created_at: at,
            updated_at: at,
            operation: None,
            usage: Usage::default(),
        };
        tx.execute(
            "INSERT INTO tasks VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                t.id,
                t.kind,
                t.owner,
                t.state,
                t.created_at,
                t.updated_at,
                serde_json::to_string(&t)?
            ],
        )?;
        tx.commit()?;
        Ok(t)
    }
    pub fn read(&self, id: &str) -> Result<Task> {
        if !valid_id(id) {
            return Err(err("任务标识无效"));
        }
        let mut t = get(&self.db, id)?;
        t.usage = self.usage(id)?;
        Ok(t)
    }
    pub fn usage(&self, id: &str) -> Result<Usage> {
        Ok(self.db.query_row("SELECT COUNT(*),SUM(input_tokens),SUM(output_tokens) FROM paid_requests WHERE task_id=?1",[id],|r|Ok(Usage{requests:r.get(0)?,input_tokens:r.get(1)?,output_tokens:r.get(2)?}))?)
    }
    pub fn reserve_request(
        &mut self,
        id: &str,
        attempt: &str,
        model: &str,
        limit: u32,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let task = get(&tx, id)?;
        if !["running", "pausing"].contains(&task.state.as_str()) {
            return Err(err("任务已停止，请求未发送"));
        }
        let count: u32 = tx.query_row(
            "SELECT COUNT(*) FROM paid_requests WHERE task_id=?1",
            [id],
            |r| r.get(0),
        )?;
        if count >= limit {
            return Err(DesktopError::new("task_budget",format!("本任务已达到 {limit} 次文字/评测请求预算，已停止发送。可核对已保存内容后在后台任务设置提高预算，再继续；未调用 TTS")));
        }
        tx.execute(
            "INSERT INTO paid_requests(attempt_id,task_id,model,at) VALUES(?1,?2,?3,?4)",
            params![attempt, id, model, now_ms()],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn record_usage(
        &self,
        attempt: &str,
        input: Option<u64>,
        output: Option<u64>,
    ) -> Result<()> {
        self.db.execute(
            "UPDATE paid_requests SET input_tokens=?2,output_tokens=?3 WHERE attempt_id=?1",
            params![attempt, input, output],
        )?;
        Ok(())
    }
    pub fn progress(&mut self, id: &str, operation: serde_json::Value) -> Result<Task> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut t = get(&tx, id)?;
        t.operation = Some(operation);
        save(&tx, &mut t)?;
        tx.commit()?;
        Ok(t)
    }
    pub fn list(&self, limit: usize) -> Result<Vec<Task>> {
        let mut q = self
            .db
            .prepare("SELECT data FROM tasks ORDER BY created_at DESC,rowid DESC LIMIT ?1")?;
        let rows = q.query_map([limit.clamp(1, 500) as i64], row)?;
        let mut out = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        for t in &mut out {
            t.usage = self.usage(&t.id)?;
        }
        Ok(out)
    }
    pub fn claim(&mut self) -> Result<Option<Task>> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let candidate = tx
            .query_row(
                "SELECT data FROM tasks WHERE state='queued' ORDER BY created_at,rowid LIMIT 1",
                [],
                row,
            )
            .optional()?;
        if let Some(mut t) = candidate {
            t.state = "running".into();
            save(&tx, &mut t)?;
            tx.commit()?;
            Ok(Some(t))
        } else {
            tx.commit()?;
            Ok(None)
        }
    }
    pub fn complete_target(&mut self, id: &str, target: &str) -> Result<Task> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut t = get(&tx, id)?;
        if !t.targets.iter().any(|s| s == target)
            || !["running", "pausing"].contains(&t.state.as_str())
        {
            return Err(err("任务范围或执行状态已变化"));
        }
        if !t.completed.iter().any(|s| s == target) {
            t.completed.push(target.into());
            tx.execute(
                "INSERT INTO task_events(task_id,at,event) VALUES(?1,?2,?3)",
                params![id, now_ms(), format!("target_complete:{target}")],
            )?;
            save(&tx, &mut t)?;
        }
        tx.commit()?;
        Ok(t)
    }
    pub fn pause(&mut self, id: &str) -> Result<Task> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut t = get(&tx, id)?;
        if t.state == "queued" {
            t.state = "paused".into();
        } else if t.state == "running" {
            t.state = "pausing".into();
        }
        save(&tx, &mut t)?;
        tx.commit()?;
        Ok(t)
    }
    pub fn resume(&mut self, id: &str) -> Result<Task> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut t = get(&tx, id)?;
        if !["paused", "failed"].contains(&t.state.as_str()) {
            return Err(err("只有暂停或失败的任务可以继续"));
        }
        t.state = "queued".into();
        t.error = None;
        save(&tx, &mut t)?;
        tx.commit()?;
        Ok(t)
    }
    pub fn cancel(&mut self, id: &str) -> Result<Task> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut t = get(&tx, id)?;
        if !["queued", "paused", "failed"].contains(&t.state.as_str()) {
            return Err(err("请先暂停正在执行的任务，等待当前请求结束后取消"));
        }
        t.state = "cancelled".into();
        save(&tx, &mut t)?;
        tx.commit()?;
        Ok(t)
    }
    pub fn finish(&mut self, id: &str, state: &str, error: Option<String>) -> Result<Task> {
        if !["ready", "failed", "paused"].contains(&state) {
            return Err(err("后台任务结束状态无效"));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut t = get(&tx, id)?;
        if !["running", "pausing"].contains(&t.state.as_str()) {
            return Err(err("此任务已停止，不能覆盖其状态"));
        }
        t.state = if t.state == "pausing" {
            "paused".into()
        } else {
            state.into()
        };
        t.error = error;
        save(&tx, &mut t)?;
        tx.commit()?;
        Ok(t)
    }
    pub fn recover_interrupted(&mut self) -> Result<usize> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let rows = {
            let mut stmt =
                tx.prepare("SELECT data FROM tasks WHERE state IN ('queued','running','pausing')")?;
            let items = stmt
                .query_map([], row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            items
        };
        let count = rows.len();
        for mut t in rows {
            t.state = "paused".into();
            t.error = Some(
                "应用关闭时任务未完成，已保存成功步骤；核对后可继续，未确认请求不会自动重发".into(),
            );
            save(&tx, &mut t)?;
        }
        tx.commit()?;
        Ok(count)
    }
}
