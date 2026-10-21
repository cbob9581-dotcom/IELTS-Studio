//! Native execution survives route changes and webview reloads. No task is silently resubmitted after a process interruption.
use crate::{
    cloud_commands,
    commands::{locked, AppState},
    contracts::{DesktopError, Result},
    source_commands,
    task_queue::{Queue, Task, TaskRequest},
};
use serde_json::json;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
};
use tauri::{Manager, State};
static ROOT_LOCKS: OnceLock<Mutex<HashMap<PathBuf, File>>> = OnceLock::new();
static WORKERS: OnceLock<Mutex<Option<(usize, Arc<tokio::sync::Semaphore>)>>> = OnceLock::new();
static EXAM_ACTIVE: AtomicBool = AtomicBool::new(false);
fn error(s: impl ToString) -> DesktopError {
    DesktopError::new("task_runtime", s)
}
fn asset_root(state: &AppState) -> Result<PathBuf> {
    locked(state, |s| Ok(PathBuf::from(&s.roots.assets)))
}
fn initialize_root(root: &Path) -> Result<()> {
    let root = root.canonicalize()?;
    let mut locks = ROOT_LOCKS
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| error("后台任务状态锁异常"))?;
    if locks.contains_key(&root) {
        return Ok(());
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join(".task-engine.lock"))?;
    file.try_lock()
        .map_err(|_| error("另一个 IELTS Studio 实例正在管理此资产库的任务，请关闭重复实例"))?;
    Queue::open(&root)?.recover_interrupted()?;
    locks.insert(root, file);
    Ok(())
}
#[tauri::command]
pub fn task_enqueue(
    request: TaskRequest,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Task> {
    let lease = cloud_commands::Lease::acquire(&format!("queue-{}", uuid::Uuid::new_v4()))?;
    let root = asset_root(&state)?;
    initialize_root(&root)?;
    if request.kind == "cloud" {
        let mut job = cloud_commands::cloud_read(request.owner.clone(), state.clone())?;
        if job["state"] == "paused" {
            job["state"] = json!("pending");
            let jobroot = if job["kind"] == "assessment" {
                locked(&state, |s| Ok(PathBuf::from(&s.roots.study)))?
            } else {
                root.clone()
            };
            crate::cloud::jobs::save(&jobroot, &job)?;
        }
    } else if request.kind == "source" {
        crate::source_library::load(&root, &request.owner)?;
    }
    let task = Queue::open(&root)?.enqueue(request)?;
    pump(app, root, lease);
    Ok(task)
}
#[tauri::command]
pub fn task_list(state: State<AppState>) -> Result<Vec<Task>> {
    let root = asset_root(&state)?;
    initialize_root(&root)?;
    Queue::open(&root)?.list(100)
}
#[tauri::command]
pub fn task_read(id: String, state: State<AppState>) -> Result<Task> {
    let root = asset_root(&state)?;
    initialize_root(&root)?;
    Queue::open(&root)?.read(&id)
}
#[tauri::command]
pub fn task_pause(id: String, state: State<AppState>) -> Result<Task> {
    let root = asset_root(&state)?;
    initialize_root(&root)?;
    Queue::open(&root)?.pause(&id)
}
#[tauri::command]
pub fn task_cancel(id: String, state: State<AppState>) -> Result<Task> {
    let root = asset_root(&state)?;
    initialize_root(&root)?;
    Queue::open(&root)?.cancel(&id)
}
#[tauri::command]
pub fn task_policy_read(state: State<AppState>) -> Result<crate::task_policy::TaskPolicy> {
    crate::task_policy::TaskPolicy::load(&asset_root(&state)?)
}
#[tauri::command]
pub fn task_policy_save(
    policy: crate::task_policy::TaskPolicy,
    state: State<AppState>,
) -> Result<()> {
    let _idle = cloud_commands::idle_lock()?;
    policy.save(&asset_root(&state)?)
}
#[tauri::command]
pub fn task_exam_active(active: bool) {
    EXAM_ACTIVE.store(active, Ordering::SeqCst);
}
#[tauri::command]
pub fn calibration_read(state: State<AppState>) -> Result<Option<serde_json::Value>> {
    let path = asset_root(&state)?.join("calibration/latest.json");
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_slice(&std::fs::read(path)?)?))
}
#[tauri::command]
pub fn calibration_import(path: String, state: State<AppState>) -> Result<serde_json::Value> {
    let _idle = cloud_commands::idle_lock()?;
    let path = PathBuf::from(path);
    if !path.is_file() || std::fs::metadata(&path)?.len() > 10 * 1024 * 1024 {
        return Err(error("请选择10MB以内的 JSON 或 JSONL 基准文件"));
    }
    let bytes = std::fs::read(&path)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| error("基准文件须使用 UTF-8 编码"))?;
    let data = if text.trim_start().starts_with('[') {
        serde_json::from_str(text)?
    } else {
        let rows = text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(serde_json::from_str::<serde_json::Value>)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        json!(rows)
    };
    let mut summary = crate::calibration::measure(&data)?;
    let root = asset_root(&state)?.join("calibration");
    std::fs::create_dir_all(&root)?;
    let id = crate::papers::hash(&bytes);
    crate::storage::atomic_json(&root.join(format!("{id}.json")), &data)?;
    summary["datasetHash"] = json!(id);
    summary["createdAt"] = json!(crate::contracts::now_ms());
    crate::storage::atomic_json(&root.join("latest.json"), &summary)?;
    Ok(summary)
}
#[tauri::command]
pub fn task_resume(id: String, app: tauri::AppHandle, state: State<AppState>) -> Result<Task> {
    let lease = cloud_commands::Lease::acquire(&format!("queue-{}", uuid::Uuid::new_v4()))?;
    let root = asset_root(&state)?;
    initialize_root(&root)?;
    let task = Queue::open(&root)?.resume(&id)?;
    pump(app, root, lease);
    Ok(task)
}
fn pump(app: tauri::AppHandle, root: PathBuf, lease: cloud_commands::Lease) {
    tauri::async_runtime::spawn(async move {
        let _lease = lease;
        let limit = crate::task_policy::TaskPolicy::load(&root)
            .map(|p| p.background_concurrency)
            .unwrap_or(2);
        let pool = {
            let Ok(mut current) = WORKERS.get_or_init(Default::default).lock() else {
                return;
            };
            if current
                .as_ref()
                .is_none_or(|(n, s)| *n != limit && s.available_permits() == *n)
            {
                *current = Some((limit, Arc::new(tokio::sync::Semaphore::new(limit))));
            }
            current.as_ref().unwrap().1.clone()
        };
        let Ok(_permit) = pool.acquire_owned().await else {
            return;
        };
        loop {
            let claimed = Queue::open(&root).and_then(|mut q| q.claim());
            let task = match claimed {
                Ok(Some(t)) => t,
                _ => break,
            };
            let policy = crate::task_policy::TaskPolicy::load(&root).unwrap_or_default();
            let context = crate::cloud::request::BudgetContext {
                root: root.clone(),
                task_id: task.id.clone(),
                limit: policy.max_text_requests,
            };
            let outcome = crate::cloud::request::BUDGET
                .scope(context, execute(&app, &root, &task))
                .await;
            if let Ok(mut q) = Queue::open(&root) {
                match outcome {
                    Ok(state) => {
                        let _ = q.finish(&task.id, &state, None);
                    }
                    Err(e) => {
                        let _ = q.finish(&task.id, "failed", Some(e.message));
                    }
                }
            }
        }
    });
}
async fn execute(app: &tauri::AppHandle, root: &Path, task: &Task) -> Result<String> {
    let mut failures = vec![];
    for target in &task.targets {
        if task.completed.contains(target) {
            continue;
        }
        loop {
            if Queue::open(root)?.read(&task.id)?.state == "pausing" {
                return Ok("paused".into());
            }
            while EXAM_ACTIVE.load(Ordering::SeqCst) {
                Queue::open(root)?.progress(
                    &task.id,
                    json!({"label":"考试进行中，后台任务等待；实时语音优先"}),
                )?;
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                if Queue::open(root)?.read(&task.id)?.state == "pausing" {
                    return Ok("paused".into());
                }
            }
            let state = app.state::<AppState>();
            if asset_root(&state)? != root {
                return Err(error("资产目录已变化，任务已停止，请从新位置继续"));
            }
            let result = if task.kind == "cloud" {
                cloud_commands::cloud_step(task.owner.clone(), state).await
            } else {
                source_commands::source_step(task.owner.clone(), target.clone(), state).await
            };
            let (value, terminal, failed, delay) = match result {
                Ok(v) if task.kind == "cloud" => {
                    let s = v["state"].as_str().unwrap_or("");
                    let terminal = [
                        "complete",
                        "review_required",
                        "source_required",
                        "failed",
                        "paused",
                    ]
                    .contains(&s);
                    let failed = ["failed", "paused"].contains(&s);
                    let delay = if v["kind"] == "import" && v["stage"] == 1 {
                        3000
                    } else {
                        50
                    };
                    (v, terminal, failed, delay)
                }
                Ok(v) => {
                    let item = v["documents"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .chain(v["units"].as_array().into_iter().flatten())
                        .chain(v["audioDrafts"].as_array().into_iter().flatten())
                        .find(|v| v["id"] == *target)
                        .cloned()
                        .ok_or_else(|| error("资料任务目标已移除"))?;
                    let s = item["state"].as_str().unwrap_or("");
                    let terminal = ["complete", "ready", "media", "failed"].contains(&s);
                    let failed = s == "failed";
                    let delay = if s == "parsing" { 3000 } else { 50 };
                    (item, terminal, failed, delay)
                }
                Err(e) => {
                    if e.code == "task_budget" {
                        return Err(e);
                    }
                    failures.push(format!("{target}：{}", e.message));
                    break;
                }
            };
            if value["operation"].is_object() {
                let mut operation = value["operation"].clone();
                operation["targetId"] = json!(target);
                Queue::open(root)?.progress(&task.id, operation)?;
            }
            if terminal {
                if failed && value["errorCode"] == "task_budget" {
                    return Err(DesktopError::new(
                        "task_budget",
                        value["error"]
                            .as_str()
                            .unwrap_or("请求预算已用完，已停止发送"),
                    ));
                }
                if failed {
                    failures.push(format!(
                        "{target}：{}",
                        value["error"].as_str().unwrap_or("步骤未完成")
                    ));
                } else {
                    Queue::open(root)?.complete_target(&task.id, target)?;
                }
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }
    }
    if failures.is_empty() {
        Ok("ready".into())
    } else {
        Err(error(failures.join("\n")))
    }
}
