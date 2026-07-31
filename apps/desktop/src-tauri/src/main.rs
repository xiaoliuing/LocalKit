#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, State, Window,
    WindowEvent,
};

const DESKTOP_SEED_VERSION_KEY: &str = "desktop_seed_version";
const CURRENT_DESKTOP_SEED_VERSION: &str = "3";
const BUNDLED_DEFAULT_DOCS_DIR_NAME: &str = "docs";
const DEFAULT_DOCS_SENTINELS: [&str; 2] = ["what-is-docs-atlas.md", "getting-started/README.md"];
const WORKSPACE_SOURCES_CHANGED_EVENT: &str = "workspace-sources-changed";
const WORKSPACE_SOURCE_WATCH_INTERVAL_MS: u64 = 1_500;
const APP_LOGS_DIR_NAME: &str = "logs";
const APP_LOG_FILE_NAME: &str = "docs-atlas.log";
const AGENT_SESSION_BACKUPS_DIR_NAME: &str = "agent-session-backups";
const APP_WINDOW_STATE_KEY: &str = "desktop_main_window_state";
const DESKTOP_MENU_ACTION_EVENT: &str = "desktop-menu-action";
const MENU_ID_IMPORT_WORKSPACE: &str = "workspace.import";
const MENU_ID_EXPORT_WORKSPACE: &str = "workspace.export";
const MENU_ID_OPEN_SEARCH: &str = "view.open-search";
const MENU_ID_OPEN_SETTINGS: &str = "view.open-settings";
const MENU_ID_OPEN_APP_DATA_DIRECTORY: &str = "system.open-app-data-directory";
const MENU_ID_OPEN_LOGS_DIRECTORY: &str = "system.open-logs-directory";
const MENU_ACTION_IMPORT_WORKSPACE: &str = "import-workspace";
const MENU_ACTION_EXPORT_WORKSPACE: &str = "export-workspace";
const MENU_ACTION_OPEN_SEARCH: &str = "open-search";
const MENU_ACTION_OPEN_SETTINGS: &str = "open-settings";
static IMPORT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

const WORKSPACE_DB_SCHEMA: &str = r#"
create table if not exists workspaces (
  id text primary key,
  name text not null,
  description text not null default '',
  icon text not null default '',
  color text not null default '',
  default_search_scope text not null default 'global',
  sort_order integer not null default 0,
  created_at text not null,
  updated_at text not null,
  last_opened_at text
);

create table if not exists workspace_source_nodes (
  id text primary key,
  workspace_id text not null,
  parent_id text,
  kind text not null,
  name text not null,
  path text not null default '',
  enabled integer not null default 1,
  position integer not null default 0,
  created_at text not null,
  updated_at text not null,
  foreign key (workspace_id) references workspaces(id) on delete cascade
);

create table if not exists app_settings (
  key text primary key,
  value_json text not null,
  updated_at text not null
);

create table if not exists recent_workspace_entries (
  workspace_id text primary key,
  opened_at text not null,
  foreign key (workspace_id) references workspaces(id) on delete cascade
);

create table if not exists workspace_source_scan_cache (
  source_root text primary key,
  fingerprint text not null,
  payload_json text not null,
  updated_at text not null
);
"#;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceDetailPayload {
    id: String,
    name: String,
    description: String,
    icon: String,
    color: String,
    default_search_scope: String,
    sort_order: i64,
    created_at: String,
    updated_at: String,
    last_opened_at: Option<String>,
    sources: Vec<WorkspaceSourceNodePayload>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSourceNodePayload {
    id: String,
    workspace_id: String,
    parent_id: Option<String>,
    kind: String,
    name: String,
    path: String,
    enabled: bool,
    position: i64,
    children: Vec<WorkspaceSourceNodePayload>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSaveInput {
    id: String,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    default_search_scope: Option<String>,
    sort_order: Option<i64>,
    last_opened_at: Option<String>,
    sources: Option<Vec<WorkspaceSourceNodeInput>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSourceNodeInput {
    id: String,
    parent_id: Option<String>,
    kind: String,
    name: String,
    path: Option<String>,
    enabled: Option<bool>,
    position: Option<i64>,
    children: Option<Vec<WorkspaceSourceNodeInput>>,
}

#[derive(Debug, Clone)]
struct WorkspaceSourceNodeRow {
    id: String,
    workspace_id: String,
    parent_id: Option<String>,
    kind: String,
    name: String,
    path: String,
    enabled: bool,
    position: i64,
}

#[derive(Debug, Clone)]
struct WorkspaceSummaryRow {
    id: String,
    name: String,
    description: String,
    icon: String,
    color: String,
    default_search_scope: String,
    sort_order: i64,
    created_at: String,
    updated_at: String,
    last_opened_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourcePathValidationPayload {
    exists: bool,
    is_directory: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoDirectoryScanPayload {
    exists: bool,
    is_directory: bool,
    message: String,
    video_count: usize,
    tree: Vec<VideoTreeNodePayload>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct VideoTreeNodePayload {
    id: String,
    name: String,
    path: String,
    relative_path: String,
    kind: String,
    size: u64,
    modified_at: u64,
    children: Vec<VideoTreeNodePayload>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSourceDocumentPayload {
    source_node_id: String,
    source_root: String,
    absolute_path: String,
    relative_path: String,
    #[serde(default)]
    modified_at: u64,
    markdown: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSourceStatusPayload {
    source_node_id: String,
    source_root: String,
    state: String,
    message: String,
    document_count: usize,
    used_cache: bool,
    checked_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSourceScanPayload {
    documents: Vec<WorkspaceSourceDocumentPayload>,
    source_statuses: Vec<WorkspaceSourceStatusPayload>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSourceWatchEventPayload {
    workspace_id: String,
    detected_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceTransferPayload {
    schema_version: u8,
    exported_at: String,
    workspace: WorkspaceTransferWorkspacePayload,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceTransferWorkspacePayload {
    name: String,
    description: String,
    icon: String,
    color: String,
    default_search_scope: String,
    sources: Vec<WorkspaceSourceNodeInput>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedWorkspaceSourcePayload {
    documents: Vec<WorkspaceSourceDocumentPayload>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DesktopMenuActionPayload {
    action: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentSessionProviderPayload {
    id: String,
    name: String,
    status: String,
    data_dir: String,
    cli_path: Option<String>,
    session_count: usize,
    deletion_support: String,
    message: String,
    risk_note: String,
    scanned_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentSessionEntryPayload {
    id: String,
    provider_id: String,
    title: String,
    project_path: String,
    updated_at: Option<String>,
    size_bytes: u64,
    status: String,
    deletion_support: String,
    risk_level: String,
    metadata: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentSessionScanPayload {
    providers: Vec<AgentSessionProviderPayload>,
    sessions: Vec<AgentSessionEntryPayload>,
    scanned_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentSessionDeleteSelection {
    provider_id: String,
    session_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentSessionDeletePlanRequest {
    selections: Vec<AgentSessionDeleteSelection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentSessionDeletePlanItemPayload {
    id: String,
    provider_id: String,
    session_id: String,
    label: String,
    action: String,
    target: String,
    size_bytes: u64,
    risk_level: String,
    protected: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentSessionDeletePlanPayload {
    id: String,
    created_at: String,
    items: Vec<AgentSessionDeletePlanItemPayload>,
    total_size_bytes: u64,
    backup_required: bool,
    high_risk: bool,
    summary: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentSessionDeleteResultItemPayload {
    plan_item_id: String,
    provider_id: String,
    session_id: String,
    status: String,
    target: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentSessionDeleteResultPayload {
    plan_id: String,
    backup_path: Option<String>,
    deleted_count: usize,
    skipped_count: usize,
    failed_count: usize,
    released_bytes: u64,
    items: Vec<AgentSessionDeleteResultItemPayload>,
    completed_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct PersistedWindowState {
    x: Option<i32>,
    y: Option<i32>,
    width: Option<u32>,
    height: Option<u32>,
    maximized: bool,
}

#[derive(Debug, Clone)]
struct MarkdownFileSnapshot {
    absolute_path: PathBuf,
    relative_path: String,
    modified_at: u64,
    size: u64,
}

#[derive(Default)]
struct WorkspaceSourceWatchState {
    active_stop_signal: Mutex<Option<Arc<AtomicBool>>>,
}

#[tauri::command]
fn list_workspace_details(app: AppHandle) -> Result<Vec<WorkspaceDetailPayload>, String> {
    let connection = open_workspace_database(&app)?;
    let mut statement = connection
        .prepare(
            r#"
      select
        id,
        name,
        description,
        icon,
        color,
        default_search_scope,
        sort_order,
        created_at,
        updated_at,
        last_opened_at
      from workspaces
      order by sort_order asc, name asc
      "#,
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            Ok(WorkspaceSummaryRow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                default_search_scope: row.get(5)?,
                sort_order: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_opened_at: row.get(9)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|workspace| load_workspace_detail(&connection, workspace))
        .collect()
}

#[tauri::command]
fn upsert_workspace(
    app: AppHandle,
    input: WorkspaceSaveInput,
) -> Result<WorkspaceDetailPayload, String> {
    let mut connection = open_workspace_database(&app)?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let now = current_timestamp();

    let created_at = transaction
        .query_row(
            "select created_at from workspaces where id = ?1",
            params![&input.id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .unwrap_or_else(|| now.clone());

    transaction
        .execute(
            r#"
      insert into workspaces (
        id,
        name,
        description,
        icon,
        color,
        default_search_scope,
        sort_order,
        created_at,
        updated_at,
        last_opened_at
      )
      values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
      on conflict(id) do update set
        name = excluded.name,
        description = excluded.description,
        icon = excluded.icon,
        color = excluded.color,
        default_search_scope = excluded.default_search_scope,
        sort_order = excluded.sort_order,
        updated_at = excluded.updated_at,
        last_opened_at = excluded.last_opened_at
      "#,
            params![
                &input.id,
                &input.name,
                input.description.clone().unwrap_or_default(),
                input.icon.clone().unwrap_or_default(),
                input.color.clone().unwrap_or_else(|| "#1f54d9".to_string()),
                input
                    .default_search_scope
                    .clone()
                    .unwrap_or_else(|| "global".to_string()),
                input.sort_order.unwrap_or(0),
                created_at,
                now,
                input.last_opened_at.clone()
            ],
        )
        .map_err(|error| error.to_string())?;

    replace_workspace_source_nodes(&transaction, &input.id, input.sources.unwrap_or_default())?;
    transaction.commit().map_err(|error| error.to_string())?;

    let connection = open_workspace_database(&app)?;
    let summary = load_workspace_summary(&connection, &input.id)?;
    load_workspace_detail(&connection, summary)
}

#[tauri::command]
fn mark_workspace_opened(
    app: AppHandle,
    workspace_id: String,
) -> Result<Option<WorkspaceDetailPayload>, String> {
    let connection = open_workspace_database(&app)?;
    let now = current_timestamp();

    let updated = connection
        .execute(
            r#"
      update workspaces
      set
        updated_at = ?2,
        last_opened_at = ?2
      where id = ?1
      "#,
            params![&workspace_id, now],
        )
        .map_err(|error| error.to_string())?;

    if updated == 0 {
        return Ok(None);
    }

    let summary = load_workspace_summary(&connection, &workspace_id)?;
    load_workspace_detail(&connection, summary).map(Some)
}

#[tauri::command]
fn delete_workspace(app: AppHandle, workspace_id: String) -> Result<bool, String> {
    let mut connection = open_workspace_database(&app)?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let workspace_count = transaction
        .query_row("select count(*) from workspaces", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|error| error.to_string())?;

    if workspace_count <= 1 {
        return Ok(false);
    }

    let deleted = transaction
        .execute(
            "delete from workspaces where id = ?1",
            params![workspace_id],
        )
        .map_err(|error| error.to_string())?;

    if deleted == 0 {
        return Ok(false);
    }

    transaction.commit().map_err(|error| error.to_string())?;
    Ok(true)
}

#[tauri::command]
fn pick_folder_path() -> Option<String> {
    rfd::FileDialog::new()
        .pick_folder()
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
fn pick_folder_paths() -> Vec<String> {
    rfd::FileDialog::new()
        .pick_folders()
        .unwrap_or_default()
        .into_iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect()
}

#[tauri::command]
fn validate_source_path(path: String) -> SourcePathValidationPayload {
    let metadata = std::fs::metadata(Path::new(&path));

    match metadata {
        Ok(metadata) => SourcePathValidationPayload {
            exists: true,
            is_directory: metadata.is_dir(),
        },
        Err(_) => SourcePathValidationPayload {
            exists: false,
            is_directory: false,
        },
    }
}

#[tauri::command]
fn scan_video_directory(path: String) -> Result<VideoDirectoryScanPayload, String> {
    let trimmed_path = path.trim();
    if trimmed_path.is_empty() {
        return Ok(VideoDirectoryScanPayload {
            exists: false,
            is_directory: false,
            message: "视频目录不能为空".to_string(),
            video_count: 0,
            tree: Vec::new(),
        });
    }

    let root = PathBuf::from(trimmed_path);
    let metadata = match std::fs::metadata(&root) {
        Ok(metadata) => metadata,
        Err(_) => {
            return Ok(VideoDirectoryScanPayload {
                exists: false,
                is_directory: false,
                message: "目录不存在".to_string(),
                video_count: 0,
                tree: Vec::new(),
            });
        }
    };

    if !metadata.is_dir() {
        return Ok(VideoDirectoryScanPayload {
            exists: true,
            is_directory: false,
            message: "路径不是目录".to_string(),
            video_count: 0,
            tree: Vec::new(),
        });
    }

    let tree = collect_video_tree_nodes(&root, &root)?;
    let video_count = count_video_tree_files(&tree);
    let message = if video_count == 0 {
        "目录下没有可识别的视频文件".to_string()
    } else {
        format!("已扫描 {} 个视频", video_count)
    };

    Ok(VideoDirectoryScanPayload {
        exists: true,
        is_directory: true,
        message,
        video_count,
        tree,
    })
}

#[tauri::command]
fn scan_workspace_sources(
    app: AppHandle,
    sources: Vec<WorkspaceSourceNodeInput>,
) -> Result<WorkspaceSourceScanPayload, String> {
    let connection = open_workspace_database(&app)?;
    let mut documents = Vec::<WorkspaceSourceDocumentPayload>::new();
    let mut source_statuses = Vec::<WorkspaceSourceStatusPayload>::new();
    let folder_sources = collect_enabled_folder_sources(None, sources, true);
    let folder_source_count = folder_sources.len();

    record_app_info(
        &app,
        "workspace.scan",
        &format!("start source_count={folder_source_count}"),
    );

    for source in &folder_sources {
        let checked_at = current_timestamp();
        match scan_single_source(&connection, source, &checked_at) {
            Ok((source_documents, status)) => {
                documents.extend(source_documents);
                source_statuses.push(status);
            }
            Err(message) => {
                record_app_error(
                    &app,
                    "workspace.scan",
                    &format!(
                        "source_id={} path={} error={message}",
                        source.id, source.path
                    ),
                );
                source_statuses.push(WorkspaceSourceStatusPayload {
                    source_node_id: source.id.clone(),
                    source_root: source.path.clone(),
                    state: "error".to_string(),
                    message,
                    document_count: 0,
                    used_cache: false,
                    checked_at,
                });
            }
        }
    }

    record_app_info(
        &app,
        "workspace.scan",
        &format!(
            "completed source_count={} document_count={} status_count={}",
            folder_source_count,
            documents.len(),
            source_statuses.len()
        ),
    );

    Ok(WorkspaceSourceScanPayload {
        documents,
        source_statuses,
    })
}

#[tauri::command]
fn save_markdown_document(app: AppHandle, path: String, markdown: String) -> Result<bool, String> {
    let trimmed_path = path.trim();
    if trimmed_path.is_empty() {
        return Err("文档路径不能为空".to_string());
    }

    let markdown_path = PathBuf::from(trimmed_path);
    if !markdown_path.exists() {
        return Err("目标文档不存在".to_string());
    }

    let metadata = std::fs::metadata(&markdown_path).map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err("目标路径不是文件".to_string());
    }

    let is_markdown = markdown_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("md"))
        .unwrap_or(false);
    if !is_markdown {
        return Err("仅支持保存 Markdown 文档".to_string());
    }

    std::fs::write(&markdown_path, markdown).map_err(|error| error.to_string())?;
    record_app_info(
        &app,
        "workspace.document.save",
        &format!("path={}", markdown_path.to_string_lossy()),
    );

    Ok(true)
}

#[tauri::command]
fn get_default_docs_path(app: AppHandle) -> Result<String, String> {
    Ok(resolve_default_docs_path(&app))
}

#[tauri::command]
fn export_workspace_config(app: AppHandle, workspace_id: String) -> Result<bool, String> {
    let connection = open_workspace_database(&app)?;
    let summary = load_workspace_summary(&connection, &workspace_id)?;
    let workspace = load_workspace_detail(&connection, summary)?;
    let workspace_name = workspace.name.clone();

    let file_path = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name(&format!(
            "{}.docs-atlas-workspace.json",
            sanitize_workspace_filename(&workspace_name)
        ))
        .save_file();

    let Some(file_path) = file_path else {
        return Ok(false);
    };

    let payload = WorkspaceTransferPayload {
        schema_version: 1,
        exported_at: current_timestamp(),
        workspace: WorkspaceTransferWorkspacePayload {
            name: workspace_name.clone(),
            description: workspace.description,
            icon: workspace.icon,
            color: workspace.color,
            default_search_scope: workspace.default_search_scope,
            sources: export_workspace_sources(workspace.sources),
        },
    };

    let payload_json = serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?;
    std::fs::write(&file_path, payload_json).map_err(|error| error.to_string())?;
    record_app_info(
        &app,
        "workspace.export",
        &format!(
            "workspace_id={} workspace_name={} file={}",
            workspace_id,
            workspace_name,
            file_path.to_string_lossy()
        ),
    );
    Ok(true)
}

#[tauri::command]
fn import_workspace_config(app: AppHandle) -> Result<Option<WorkspaceDetailPayload>, String> {
    let file_path = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .pick_file();

    let Some(file_path) = file_path else {
        return Ok(None);
    };

    let raw_value = std::fs::read_to_string(&file_path).map_err(|error| error.to_string())?;
    let payload = serde_json::from_str::<WorkspaceTransferPayload>(&raw_value)
        .map_err(|error| error.to_string())?;

    if payload.schema_version != 1 {
        return Err("暂不支持该文档仓库导入版本".to_string());
    }

    let imported_workspace = insert_imported_workspace(&app, payload)?;
    record_app_info(
        &app,
        "workspace.import",
        &format!(
            "workspace_id={} workspace_name={} file={}",
            imported_workspace.id,
            imported_workspace.name,
            file_path.to_string_lossy()
        ),
    );
    Ok(Some(imported_workspace))
}

#[tauri::command]
fn watch_workspace_sources(
    app: AppHandle,
    state: State<'_, WorkspaceSourceWatchState>,
    workspace_id: String,
    sources: Vec<WorkspaceSourceNodeInput>,
) -> Result<(), String> {
    stop_workspace_sources_watch(&state);

    let folder_sources = collect_enabled_folder_sources(None, sources, true);
    if folder_sources.is_empty() {
        record_app_info(
            &app,
            "workspace.watch",
            &format!(
                "skip workspace_id={} reason=no_enabled_sources",
                workspace_id
            ),
        );
        return Ok(());
    }

    let stop_signal = Arc::new(AtomicBool::new(false));
    {
        let mut active_stop_signal = state
            .active_stop_signal
            .lock()
            .map_err(|_| "failed to lock workspace source watch state".to_string())?;
        *active_stop_signal = Some(stop_signal.clone());
    }

    record_app_info(
        &app,
        "workspace.watch",
        &format!(
            "started workspace_id={} source_count={}",
            workspace_id,
            folder_sources.len()
        ),
    );
    spawn_workspace_sources_watch(app, workspace_id, folder_sources, stop_signal);
    Ok(())
}

#[tauri::command]
fn unwatch_workspace_sources(
    app: AppHandle,
    state: State<'_, WorkspaceSourceWatchState>,
) -> Result<(), String> {
    stop_workspace_sources_watch(&state);
    record_app_info(&app, "workspace.watch", "stopped");
    Ok(())
}

#[tauri::command]
fn open_app_data_directory(app: AppHandle) -> Result<bool, String> {
    let app_data_directory = resolve_app_data_directory(&app)?;
    std::fs::create_dir_all(&app_data_directory).map_err(|error| error.to_string())?;
    open_path_in_file_manager(&app_data_directory)?;
    record_app_info(
        &app,
        "system.open_path",
        &format!(
            "kind=app_data path={}",
            app_data_directory.to_string_lossy()
        ),
    );
    Ok(true)
}

#[tauri::command]
fn open_logs_directory(app: AppHandle) -> Result<bool, String> {
    let logs_directory = ensure_logs_directory(&app)?;
    open_path_in_file_manager(&logs_directory)?;
    record_app_info(
        &app,
        "system.open_path",
        &format!("kind=logs path={}", logs_directory.to_string_lossy()),
    );
    Ok(true)
}

#[tauri::command]
fn export_logs_file(app: AppHandle) -> Result<bool, String> {
    let log_file_path = ensure_log_file_path(&app)?;
    let file_path = rfd::FileDialog::new()
        .add_filter("Log", &["log", "txt"])
        .set_file_name(&format!("docs-atlas-logs-{}.log", current_timestamp()))
        .save_file();

    let Some(file_path) = file_path else {
        return Ok(false);
    };

    std::fs::copy(&log_file_path, &file_path).map_err(|error| error.to_string())?;
    record_app_info(
        &app,
        "system.export_logs",
        &format!(
            "from={} to={}",
            log_file_path.to_string_lossy(),
            file_path.to_string_lossy()
        ),
    );
    Ok(true)
}

#[tauri::command]
fn open_external_url(app: AppHandle, url: String) -> Result<bool, String> {
    let normalized_url = url.trim();

    if !(normalized_url.starts_with("https://") || normalized_url.starts_with("http://")) {
        return Err("only http and https URLs are supported".into());
    }

    open_url_in_browser(normalized_url)?;
    record_app_info(&app, "system.open_url", normalized_url);
    Ok(true)
}

#[tauri::command]
fn set_window_background_color(window: Window, color: String) -> Result<bool, String> {
    let parsed = parse_hex_color(&color)?;
    window
        .set_background_color(Some(parsed))
        .map_err(|error| error.to_string())?;
    Ok(true)
}

#[tauri::command]
fn scan_agent_sessions(app: AppHandle) -> Result<AgentSessionScanPayload, String> {
    let scanned_at = current_timestamp();
    let mut providers = Vec::new();
    let mut sessions = Vec::new();

    let claude = scan_claude_code_sessions(&scanned_at);
    sessions.extend(claude.sessions);
    providers.push(claude.provider);

    let codex = scan_codex_sessions(&scanned_at);
    sessions.extend(codex.sessions);
    providers.push(codex.provider);

    let opencode = scan_opencode_sessions(&scanned_at);
    sessions.extend(opencode.sessions);
    providers.push(opencode.provider);

    record_app_info(
        &app,
        "agent_sessions.scan",
        &format!(
            "provider_count={} session_count={}",
            providers.len(),
            sessions.len()
        ),
    );

    Ok(AgentSessionScanPayload {
        providers,
        sessions,
        scanned_at,
    })
}

#[tauri::command]
fn create_agent_session_delete_plan(
    request: AgentSessionDeletePlanRequest,
) -> Result<AgentSessionDeletePlanPayload, String> {
    let scan = build_agent_session_scan_snapshot();
    let created_at = current_timestamp();
    let mut items = Vec::new();

    for selection in request.selections {
        let Some(session) = scan.sessions.iter().find(|item| {
            item.provider_id == selection.provider_id && item.id == selection.session_id
        }) else {
            items.push(AgentSessionDeletePlanItemPayload {
                id: format!(
                    "plan:item:missing:{}",
                    create_stable_id(&format!(
                        "{}:{}",
                        selection.provider_id, selection.session_id
                    ))
                ),
                provider_id: selection.provider_id,
                session_id: selection.session_id,
                label: "缺失会话".to_string(),
                action: "skip".to_string(),
                target: String::new(),
                size_bytes: 0,
                risk_level: "medium".to_string(),
                protected: true,
                message: "会话在当前扫描结果中不存在，已跳过".to_string(),
            });
            continue;
        };

        items.extend(create_agent_session_plan_items(session));
    }

    let total_size_bytes = items
        .iter()
        .filter(|item| !item.protected && item.action != "skip" && item.action != "manual")
        .map(|item| item.size_bytes)
        .sum::<u64>();
    let high_risk = items.iter().any(|item| item.risk_level == "high");
    let actionable_count = items
        .iter()
        .filter(|item| !item.protected && item.action != "skip" && item.action != "manual")
        .count();

    Ok(AgentSessionDeletePlanPayload {
        id: format!("agent-session-plan:{}", current_timestamp()),
        created_at,
        items,
        total_size_bytes,
        backup_required: actionable_count > 0,
        high_risk,
        summary: format!(
            "将处理 {actionable_count} 个可执行目标，预计释放 {}",
            format_size_bytes(total_size_bytes)
        ),
    })
}

#[tauri::command]
fn execute_agent_session_delete_plan(
    app: AppHandle,
    plan: AgentSessionDeletePlanPayload,
    confirm_high_risk: bool,
    skip_backup: Option<bool>,
) -> Result<AgentSessionDeleteResultPayload, String> {
    if plan.high_risk && !confirm_high_risk {
        return Err("删除计划包含高风险目标，请确认后再执行".to_string());
    }
    let should_skip_backup = skip_backup.unwrap_or(false);

    let actionable_items = plan
        .items
        .iter()
        .filter(|item| !item.protected && item.action != "skip" && item.action != "manual")
        .cloned()
        .collect::<Vec<_>>();
    let backup_path = if actionable_items.is_empty() {
        None
    } else if should_skip_backup {
        None
    } else {
        Some(create_agent_session_backup(&app, &plan, &actionable_items)?)
    };
    let mut result_items = Vec::new();
    let mut deleted_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;
    let mut released_bytes = 0;

    for item in plan.items {
        if item.protected || item.action == "skip" || item.action == "manual" {
            skipped_count += 1;
            result_items.push(AgentSessionDeleteResultItemPayload {
                plan_item_id: item.id,
                provider_id: item.provider_id,
                session_id: item.session_id,
                status: "skipped".to_string(),
                target: item.target,
                message: item.message,
            });
            continue;
        }

        let delete_result = execute_agent_session_plan_item(&item);
        match delete_result {
            Ok(message) => {
                deleted_count += 1;
                released_bytes += item.size_bytes;
                result_items.push(AgentSessionDeleteResultItemPayload {
                    plan_item_id: item.id,
                    provider_id: item.provider_id,
                    session_id: item.session_id,
                    status: "deleted".to_string(),
                    target: item.target,
                    message,
                });
            }
            Err(error) => {
                failed_count += 1;
                result_items.push(AgentSessionDeleteResultItemPayload {
                    plan_item_id: item.id,
                    provider_id: item.provider_id,
                    session_id: item.session_id,
                    status: "failed".to_string(),
                    target: item.target,
                    message: error,
                });
            }
        }
    }

    record_app_info(
        &app,
        "agent_sessions.delete",
        &format!(
            "plan_id={} deleted={} skipped={} failed={} mode={} backup={}",
            plan.id,
            deleted_count,
            skipped_count,
            failed_count,
            if should_skip_backup {
                "permanent"
            } else {
                "backup_then_delete"
            },
            backup_path
                .as_ref()
                .map(|path| path.to_string_lossy())
                .unwrap_or_default()
        ),
    );

    Ok(AgentSessionDeleteResultPayload {
        plan_id: plan.id,
        backup_path: backup_path.map(|path| path.to_string_lossy().to_string()),
        deleted_count,
        skipped_count,
        failed_count,
        released_bytes,
        items: result_items,
        completed_at: current_timestamp(),
    })
}

#[tauri::command]
fn open_agent_session_backups_directory(app: AppHandle) -> Result<bool, String> {
    let backups_directory = ensure_agent_session_backups_directory(&app)?;
    open_path_in_file_manager(&backups_directory)?;
    record_app_info(
        &app,
        "agent_sessions.open_backups",
        &format!("path={}", backups_directory.to_string_lossy()),
    );
    Ok(true)
}

#[derive(Debug, Clone)]
struct AgentSessionProviderScan {
    provider: AgentSessionProviderPayload,
    sessions: Vec<AgentSessionEntryPayload>,
}

fn build_agent_session_scan_snapshot() -> AgentSessionScanPayload {
    let scanned_at = current_timestamp();
    let mut providers = Vec::new();
    let mut sessions = Vec::new();

    let claude = scan_claude_code_sessions(&scanned_at);
    sessions.extend(claude.sessions);
    providers.push(claude.provider);

    let codex = scan_codex_sessions(&scanned_at);
    sessions.extend(codex.sessions);
    providers.push(codex.provider);

    let opencode = scan_opencode_sessions(&scanned_at);
    sessions.extend(opencode.sessions);
    providers.push(opencode.provider);

    AgentSessionScanPayload {
        providers,
        sessions,
        scanned_at,
    }
}

fn scan_claude_code_sessions(scanned_at: &str) -> AgentSessionProviderScan {
    let data_dir = home_directory()
        .map(|home| home.join(".claude").join("projects"))
        .unwrap_or_else(|| PathBuf::from(".claude/projects"));
    let cli_path = find_command_path("claude");
    let mut sessions = Vec::new();
    let status;
    let message;

    if data_dir.is_dir() {
        let files = collect_files_with_extension(&data_dir, "jsonl");
        sessions = files
            .into_iter()
            .map(|file_path| {
                build_file_agent_session("claude-code", &data_dir, &file_path, "file", "medium")
            })
            .collect();
        status = "ready";
        message = if cli_path.is_some() {
            "已检测到 Claude Code 项目会话和 CLI"
        } else {
            "已检测到 Claude Code 项目会话，未检测到 CLI"
        };
    } else {
        status = "unavailable";
        message = "未找到 Claude Code 项目会话目录";
    }

    AgentSessionProviderScan {
        provider: AgentSessionProviderPayload {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            status: status.to_string(),
            data_dir: data_dir.to_string_lossy().to_string(),
            cli_path,
            session_count: sessions.len(),
            deletion_support: if sessions.is_empty() {
                "unsupported"
            } else {
                "file"
            }
            .to_string(),
            message: message.to_string(),
            risk_note:
                "优先按已识别 transcript 文件清理；项目级清理会影响该项目全部 Claude Code 状态"
                    .to_string(),
            scanned_at: scanned_at.to_string(),
        },
        sessions,
    }
}

fn scan_codex_sessions(scanned_at: &str) -> AgentSessionProviderScan {
    let data_dir = env::var("CODEX_HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| home_directory().map(|home| home.join(".codex")))
        .unwrap_or_else(|| PathBuf::from(".codex"));
    let sessions_dir = data_dir.join("sessions");
    let archived_sessions_dir = data_dir.join("archived_sessions");
    let cli_path = find_command_path("codex");
    let mut session_files = Vec::new();
    session_files.extend(collect_files_with_extension(&sessions_dir, "jsonl"));
    session_files.extend(collect_files_with_extension(
        &archived_sessions_dir,
        "jsonl",
    ));
    let codex_title_index = build_codex_session_title_index(&data_dir);
    let sessions = session_files
        .into_iter()
        .map(|file_path| {
            let mut session =
                build_file_agent_session("codex", &data_dir, &file_path, "high-risk", "high");
            if let Some(title) = codex_title_index.get(&session.id) {
                session.title = title.clone();
            }
            if file_path.starts_with(&archived_sessions_dir) {
                session.status = "归档会话，高风险".to_string();
                session.metadata.push("archived_sessions".to_string());
            } else {
                session.status = "活动会话，高风险".to_string();
                session.metadata.push("sessions".to_string());
            }
            session
                .metadata
                .push("需要关注 state_5.sqlite / session_index.jsonl 一致性".to_string());
            session
        })
        .collect::<Vec<_>>();
    let status = if data_dir.is_dir() {
        if sessions.is_empty() {
            "limited"
        } else {
            "ready"
        }
    } else {
        "unavailable"
    };

    AgentSessionProviderScan {
        provider: AgentSessionProviderPayload {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            status: status.to_string(),
            data_dir: data_dir.to_string_lossy().to_string(),
            cli_path,
            session_count: sessions.len(),
            deletion_support: if sessions.is_empty() {
                "unsupported"
            } else {
                "high-risk"
            }
            .to_string(),
            message: if data_dir.is_dir() {
                "已检测 Codex 数据目录；删除会话前必须备份并确认索引风险".to_string()
            } else {
                "未找到 Codex 数据目录".to_string()
            },
            risk_note: "Codex 会话可能关联 state_5.sqlite、session_index.jsonl 和归档目录，禁止删除整个根目录".to_string(),
            scanned_at: scanned_at.to_string(),
        },
        sessions,
    }
}

fn scan_opencode_sessions(scanned_at: &str) -> AgentSessionProviderScan {
    let cli_path = find_command_path("opencode");
    let data_dir = resolve_opencode_data_dir().unwrap_or_else(|| default_opencode_data_dir());
    let mut sessions = if cli_path.is_some() {
        scan_opencode_sessions_from_cli()
    } else {
        Vec::new()
    };

    if sessions.is_empty() && data_dir.is_dir() {
        sessions = collect_opencode_file_sessions(&data_dir);
    }

    let status = if cli_path.is_some() {
        "ready"
    } else if data_dir.is_dir() {
        "limited"
    } else {
        "unavailable"
    };
    let deletion_support = if cli_path.is_some() {
        "official"
    } else if sessions.is_empty() {
        "unsupported"
    } else {
        "read-only"
    };

    AgentSessionProviderScan {
        provider: AgentSessionProviderPayload {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            status: status.to_string(),
            data_dir: data_dir.to_string_lossy().to_string(),
            cli_path,
            session_count: sessions.len(),
            deletion_support: deletion_support.to_string(),
            message: if status == "ready" {
                "已检测到 OpenCode CLI，删除时将优先调用官方命令".to_string()
            } else if status == "limited" {
                "未检测到 OpenCode CLI，仅提供只读扫描".to_string()
            } else {
                "未找到 OpenCode CLI 或数据目录".to_string()
            },
            risk_note: "OpenCode 删除优先使用 opencode session delete；无 CLI 时不直接改数据库"
                .to_string(),
            scanned_at: scanned_at.to_string(),
        },
        sessions,
    }
}

fn build_file_agent_session(
    provider_id: &str,
    data_root: &Path,
    file_path: &Path,
    deletion_support: &str,
    risk_level: &str,
) -> AgentSessionEntryPayload {
    let metadata = fs::metadata(file_path).ok();
    let size_bytes = metadata.as_ref().map(|item| item.len()).unwrap_or(0);
    let updated_at = metadata
        .and_then(|item| item.modified().ok())
        .map(system_time_to_string);
    let relative_path = file_path.strip_prefix(data_root).unwrap_or(file_path);
    let first_line = read_first_line(file_path);
    let title = if provider_id == "claude-code" {
        extract_claude_ai_title_from_file(file_path)
    } else {
        None
    }
    .or_else(|| first_line.as_deref().and_then(extract_title_from_json_line))
    .unwrap_or_else(|| {
        file_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("未命名会话")
            .to_string()
    });
    let project_path = first_line
        .as_deref()
        .and_then(|line| extract_project_path_from_json_line(provider_id, line))
        .unwrap_or_else(|| infer_project_path_from_session_path(provider_id, data_root, file_path));
    let session_id = first_line
        .as_deref()
        .and_then(|line| extract_session_id_from_json_line(provider_id, line))
        .unwrap_or_else(|| {
            create_stable_id(&format!("{}:{}", provider_id, file_path.to_string_lossy()))
        });

    AgentSessionEntryPayload {
        id: session_id,
        provider_id: provider_id.to_string(),
        title,
        project_path,
        updated_at,
        size_bytes,
        status: if risk_level == "high" {
            "高风险".to_string()
        } else {
            "可删除文件".to_string()
        },
        deletion_support: deletion_support.to_string(),
        risk_level: risk_level.to_string(),
        metadata: vec![
            relative_path.to_string_lossy().to_string(),
            file_path.to_string_lossy().to_string(),
        ],
    }
}

fn read_first_line(path: &Path) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    if reader.read_line(&mut line).ok()? == 0 {
        return None;
    }
    Some(line.trim_end_matches(['\r', '\n']).to_string())
}

fn read_jsonl_values(path: &Path, max_lines: usize) -> Vec<Value> {
    let Ok(file) = fs::File::open(path) else {
        return Vec::new();
    };
    BufReader::new(file)
        .lines()
        .take(max_lines)
        .filter_map(Result::ok)
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .collect()
}

fn extract_claude_ai_title_from_file(path: &Path) -> Option<String> {
    read_jsonl_values(path, 16).into_iter().find_map(|value| {
        if value.get("type").and_then(|item| item.as_str()) != Some("ai-title") {
            return None;
        }
        value
            .get("aiTitle")
            .and_then(|item| item.as_str())
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(compact_session_title)
    })
}

fn build_codex_session_title_index(data_dir: &Path) -> HashMap<String, String> {
    let mut titles = read_codex_session_index_titles(&data_dir.join("session_index.jsonl"));
    for (session_id, title) in read_codex_state_sqlite_titles(&data_dir.join("state_5.sqlite")) {
        titles.entry(session_id).or_insert(title);
    }
    titles
}

fn read_codex_session_index_titles(path: &Path) -> HashMap<String, String> {
    let mut titles = HashMap::new();
    for value in read_jsonl_values(path, usize::MAX) {
        let Some(session_id) = extract_string_by_keys(&value, &["id", "sessionId", "session_id"])
        else {
            continue;
        };
        let Some(title) = extract_string_by_keys(&value, &["title", "aiTitle", "summary", "name"])
        else {
            continue;
        };
        titles.insert(session_id, compact_session_title(&title));
    }
    titles
}

fn read_codex_state_sqlite_titles(path: &Path) -> HashMap<String, String> {
    let mut titles = HashMap::new();
    if !path.is_file() {
        return titles;
    }
    let Ok(connection) = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
        return titles;
    };
    let table_names = sqlite_table_names(&connection);
    for table_name in table_names {
        let columns = sqlite_table_columns(&connection, &table_name);
        let id_columns = columns
            .iter()
            .filter(|column| {
                let normalized = column.to_lowercase();
                normalized == "id" || (normalized.contains("session") && normalized.contains("id"))
            })
            .collect::<Vec<_>>();
        let title_columns = columns
            .iter()
            .filter(|column| {
                let normalized = column.to_lowercase();
                normalized.contains("title")
                    || normalized.contains("summary")
                    || normalized == "name"
            })
            .collect::<Vec<_>>();

        for id_column in &id_columns {
            for title_column in &title_columns {
                let query = format!(
                    "select {}, {} from {} where {} is not null and trim(cast({} as text)) != ''",
                    quote_sqlite_identifier(id_column),
                    quote_sqlite_identifier(title_column),
                    quote_sqlite_identifier(&table_name),
                    quote_sqlite_identifier(title_column),
                    quote_sqlite_identifier(title_column),
                );
                let Ok(mut statement) = connection.prepare(&query) else {
                    continue;
                };
                let Ok(rows) = statement.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                }) else {
                    continue;
                };
                for row in rows.flatten() {
                    let session_id = row.0.trim().to_string();
                    let title = row.1.trim().to_string();
                    if !session_id.is_empty() && !title.is_empty() {
                        titles
                            .entry(session_id)
                            .or_insert_with(|| compact_session_title(&title));
                    }
                }
            }
        }
    }
    titles
}

fn sqlite_table_names(connection: &Connection) -> Vec<String> {
    let Ok(mut statement) = connection.prepare(
        "select name from sqlite_master where type = 'table' and name not like 'sqlite_%'",
    ) else {
        return Vec::new();
    };
    statement
        .query_map([], |row| row.get::<_, String>(0))
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
}

fn sqlite_table_columns(connection: &Connection, table_name: &str) -> Vec<String> {
    let query = format!("pragma table_info({})", quote_sqlite_identifier(table_name));
    let Ok(mut statement) = connection.prepare(&query) else {
        return Vec::new();
    };
    statement
        .query_map([], |row| row.get::<_, String>(1))
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
}

fn quote_sqlite_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn extract_string_by_keys(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(value) = map.get(*key).and_then(|value| value.as_str()) {
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
            map.values()
                .find_map(|value| extract_string_by_keys(value, keys))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|value| extract_string_by_keys(value, keys)),
        _ => None,
    }
}

fn extract_title_from_json_line(line: &str) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(line).ok()?;
    for key in ["title", "summary", "cwd", "projectPath", "project_path"] {
        if let Some(value) = parsed.get(key).and_then(|item| item.as_str()) {
            if !value.trim().is_empty() {
                return Some(compact_session_title(value));
            }
        }
    }
    if let Some(value) = parsed
        .get("payload")
        .and_then(|payload| payload.get("cwd"))
        .and_then(|value| value.as_str())
    {
        if !value.trim().is_empty() {
            return Some(compact_session_title(value));
        }
    }
    None
}

fn compact_session_title(value: &str) -> String {
    let trimmed = value.trim();
    Path::new(trimmed)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(trimmed)
        .chars()
        .take(120)
        .collect()
}

fn extract_project_path_from_json_line(provider_id: &str, line: &str) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(line).ok()?;
    let value = match provider_id {
        "codex" => parsed
            .get("payload")
            .and_then(|payload| payload.get("cwd"))
            .or_else(|| parsed.get("cwd")),
        _ => parsed
            .get("cwd")
            .or_else(|| parsed.get("projectPath"))
            .or_else(|| parsed.get("project_path")),
    }?;
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn extract_session_id_from_json_line(provider_id: &str, line: &str) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(line).ok()?;
    let value = match provider_id {
        "codex" => parsed
            .get("payload")
            .and_then(|payload| payload.get("id"))
            .or_else(|| parsed.get("id")),
        _ => parsed.get("id"),
    }?;
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn infer_project_path_from_session_path(
    provider_id: &str,
    data_root: &Path,
    file_path: &Path,
) -> String {
    match provider_id {
        "claude-code" => file_path
            .parent()
            .and_then(|parent| parent.strip_prefix(data_root).ok())
            .map(|path| path.to_string_lossy().replace('-', "/"))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "未知项目".to_string()),
        _ => "未知项目".to_string(),
    }
}

fn scan_opencode_sessions_from_cli() -> Vec<AgentSessionEntryPayload> {
    let output = Command::new("opencode")
        .args(["session", "list", "--json"])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let raw_value = String::from_utf8_lossy(&output.stdout);
    let parsed = serde_json::from_str::<Value>(&raw_value).ok();
    let Some(parsed) = parsed else {
        return Vec::new();
    };
    let items = if let Some(array) = parsed.as_array() {
        array.clone()
    } else if let Some(array) = parsed.get("sessions").and_then(|value| value.as_array()) {
        array.clone()
    } else {
        Vec::new()
    };

    items
        .into_iter()
        .filter_map(|item| {
            let id = item
                .get("id")
                .or_else(|| item.get("sessionID"))
                .or_else(|| item.get("sessionId"))
                .and_then(|value| value.as_str())?
                .to_string();
            let title = item
                .get("title")
                .or_else(|| item.get("name"))
                .and_then(|value| value.as_str())
                .unwrap_or(&id)
                .to_string();
            let project_path = item
                .get("cwd")
                .or_else(|| item.get("projectPath"))
                .or_else(|| item.get("directory"))
                .and_then(|value| value.as_str())
                .unwrap_or("未知项目")
                .to_string();
            let updated_at = item
                .get("updated")
                .or_else(|| item.get("updatedAt"))
                .or_else(|| item.get("time"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());

            Some(AgentSessionEntryPayload {
                id,
                provider_id: "opencode".to_string(),
                title,
                project_path,
                updated_at,
                size_bytes: item
                    .get("size")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0),
                status: "官方 CLI".to_string(),
                deletion_support: "official".to_string(),
                risk_level: "low".to_string(),
                metadata: vec!["opencode session list --json".to_string()],
            })
        })
        .collect()
}

fn collect_opencode_file_sessions(data_dir: &Path) -> Vec<AgentSessionEntryPayload> {
    collect_files_with_extension(data_dir, "json")
        .into_iter()
        .filter(|path| path.to_string_lossy().to_lowercase().contains("session"))
        .map(|path| build_file_agent_session("opencode", data_dir, &path, "read-only", "medium"))
        .collect()
}

fn create_agent_session_plan_items(
    session: &AgentSessionEntryPayload,
) -> Vec<AgentSessionDeletePlanItemPayload> {
    let mut items = Vec::new();
    match session.provider_id.as_str() {
        "opencode" if session.deletion_support == "official" => {
            items.push(AgentSessionDeletePlanItemPayload {
                id: create_plan_item_id(session, "delete-opencode-session", &session.id),
                provider_id: session.provider_id.clone(),
                session_id: session.id.clone(),
                label: session.title.clone(),
                action: "delete-opencode-session".to_string(),
                target: session.id.clone(),
                size_bytes: session.size_bytes,
                risk_level: "low".to_string(),
                protected: false,
                message: "将调用 opencode session delete 删除该会话".to_string(),
            });
        }
        "opencode" => {
            items.push(create_skip_plan_item(
                session,
                "OpenCode CLI 不可用，避免直接修改数据库",
            ));
        }
        "claude-code" | "codex" => {
            let target = session.metadata.get(1).cloned().unwrap_or_default();
            let protected =
                is_protected_agent_session_path(&session.provider_id, Path::new(&target));
            items.push(AgentSessionDeletePlanItemPayload {
                id: create_plan_item_id(session, "delete-file", &target),
                provider_id: session.provider_id.clone(),
                session_id: session.id.clone(),
                label: session.title.clone(),
                action: if protected { "skip" } else { "delete-file" }.to_string(),
                target,
                size_bytes: session.size_bytes,
                risk_level: session.risk_level.clone(),
                protected,
                message: if protected {
                    "目标路径属于受保护数据，已跳过".to_string()
                } else if session.provider_id == "codex" {
                    "将删除已识别 Codex 会话文件；索引/数据库需要保守处理".to_string()
                } else {
                    "将删除已识别 Claude Code transcript 文件".to_string()
                },
            });

            if session.provider_id == "codex" {
                items.extend(create_codex_consistency_plan_items(session));
            }
        }
        _ => items.push(create_skip_plan_item(
            session,
            "该 Provider 当前不支持自动删除",
        )),
    }

    items
}

fn create_codex_consistency_plan_items(
    session: &AgentSessionEntryPayload,
) -> Vec<AgentSessionDeletePlanItemPayload> {
    let data_dir = env::var("CODEX_HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| home_directory().map(|home| home.join(".codex")))
        .unwrap_or_else(|| PathBuf::from(".codex"));
    ["state_5.sqlite", "session_index.jsonl"]
        .into_iter()
        .filter_map(|filename| {
            let path = data_dir.join(filename);
            if !path.exists() {
                return None;
            }
            Some(AgentSessionDeletePlanItemPayload {
                id: create_plan_item_id(session, "manual", &path.to_string_lossy()),
                provider_id: session.provider_id.clone(),
                session_id: session.id.clone(),
                label: format!("Codex 一致性检查：{filename}"),
                action: "manual".to_string(),
                target: path.to_string_lossy().to_string(),
                size_bytes: file_size(&path),
                risk_level: "high".to_string(),
                protected: true,
                message: "该索引或数据库不会被自动删除；需要保留备份并通过后续专门逻辑处理一致性"
                    .to_string(),
            })
        })
        .collect()
}

fn create_skip_plan_item(
    session: &AgentSessionEntryPayload,
    message: &str,
) -> AgentSessionDeletePlanItemPayload {
    AgentSessionDeletePlanItemPayload {
        id: create_plan_item_id(session, "skip", message),
        provider_id: session.provider_id.clone(),
        session_id: session.id.clone(),
        label: session.title.clone(),
        action: "skip".to_string(),
        target: String::new(),
        size_bytes: 0,
        risk_level: session.risk_level.clone(),
        protected: true,
        message: message.to_string(),
    }
}

fn create_plan_item_id(session: &AgentSessionEntryPayload, action: &str, target: &str) -> String {
    format!(
        "plan:item:{}",
        create_stable_id(&format!(
            "{}:{}:{}:{}",
            session.provider_id, session.id, action, target
        ))
    )
}

fn execute_agent_session_plan_item(
    item: &AgentSessionDeletePlanItemPayload,
) -> Result<String, String> {
    match item.action.as_str() {
        "delete-file" => {
            let path = PathBuf::from(&item.target);
            if !path.is_file() {
                return Err("目标文件不存在或不是文件".to_string());
            }
            fs::remove_file(&path).map_err(|error| error.to_string())?;
            Ok("文件已删除".to_string())
        }
        "delete-directory" => {
            let path = PathBuf::from(&item.target);
            if !path.is_dir() {
                return Err("目标目录不存在或不是目录".to_string());
            }
            fs::remove_dir_all(&path).map_err(|error| error.to_string())?;
            Ok("目录已删除".to_string())
        }
        "delete-opencode-session" => {
            let output = Command::new("opencode")
                .args(["session", "delete", &item.session_id])
                .output()
                .map_err(|error| error.to_string())?;
            if output.status.success() {
                Ok("OpenCode 官方 CLI 删除完成".to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
            }
        }
        _ => Err("不支持的删除动作".to_string()),
    }
}

fn create_agent_session_backup(
    app: &AppHandle,
    plan: &AgentSessionDeletePlanPayload,
    items: &[AgentSessionDeletePlanItemPayload],
) -> Result<PathBuf, String> {
    let backup_root = ensure_agent_session_backups_directory(app)?;
    let backup_dir = backup_root.join(sanitize_filename(&format!(
        "{}-{}",
        current_timestamp(),
        plan.id
    )));
    fs::create_dir_all(&backup_dir).map_err(|error| error.to_string())?;
    let mut manifest = Vec::new();

    for item in items {
        match item.action.as_str() {
            "delete-file" | "delete-directory" => {
                let source = PathBuf::from(&item.target);
                if !source.exists() {
                    return Err(format!(
                        "备份失败，目标不存在：{}",
                        source.to_string_lossy()
                    ));
                }
                let relative_target =
                    sanitize_filename(&format!("{}-{}", item.provider_id, item.session_id));
                let destination = backup_dir.join(relative_target);
                if source.is_dir() {
                    copy_directory_recursive(&source, &destination)?;
                } else {
                    fs::copy(&source, &destination).map_err(|error| error.to_string())?;
                }
                manifest.push(format!(
                    "{}\t{}\t{}",
                    item.provider_id,
                    source.to_string_lossy(),
                    destination.to_string_lossy()
                ));
            }
            "delete-opencode-session" => {
                let Some(db_path) = resolve_opencode_backup_source() else {
                    return Err("备份失败，未找到 OpenCode 数据库或数据目录".to_string());
                };
                let destination = backup_dir.join("opencode-data-backup");
                if db_path.is_dir() {
                    copy_directory_recursive(&db_path, &destination)?;
                } else {
                    fs::copy(&db_path, &destination).map_err(|error| error.to_string())?;
                }
                manifest.push(format!(
                    "opencode\t{}\t{}",
                    db_path.to_string_lossy(),
                    destination.to_string_lossy()
                ));
            }
            _ => {}
        }
    }

    fs::write(backup_dir.join("manifest.tsv"), manifest.join("\n"))
        .map_err(|error| error.to_string())?;
    Ok(backup_dir)
}

fn ensure_agent_session_backups_directory(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = resolve_app_data_directory(app)?.join(AGENT_SESSION_BACKUPS_DIR_NAME);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory)
}

fn is_protected_agent_session_path(provider_id: &str, path: &Path) -> bool {
    if path.as_os_str().is_empty() {
        return true;
    }
    let normalized = path.to_string_lossy().to_lowercase();
    let protected_names = [
        "auth.json",
        "config.json",
        "config.toml",
        "settings.json",
        "credentials",
        ".env",
        "mcp.json",
        "mcp_servers.json",
    ];
    if protected_names
        .iter()
        .any(|name| normalized.ends_with(name) || normalized.contains(&format!("/{name}")))
    {
        return true;
    }
    match provider_id {
        "claude-code" => !normalized.ends_with(".jsonl"),
        "codex" => !normalized.ends_with(".jsonl"),
        _ => true,
    }
}

fn collect_files_with_extension(root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !root.is_dir() {
        return files;
    }
    collect_files_with_extension_inner(root, extension, &mut files);
    files.sort();
    files
}

fn collect_files_with_extension_inner(root: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root);
    let Ok(entries) = entries else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension_inner(&path, extension, files);
            continue;
        }
        if path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case(extension))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory_recursive(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn resolve_opencode_data_dir() -> Option<PathBuf> {
    let output = Command::new("opencode")
        .args(["db", "path"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(raw_path))
    }
}

fn resolve_opencode_backup_source() -> Option<PathBuf> {
    resolve_existing_opencode_backup_source(
        resolve_opencode_data_dir(),
        default_opencode_data_dir(),
    )
}

fn resolve_existing_opencode_backup_source(
    cli_path: Option<PathBuf>,
    default_dir: PathBuf,
) -> Option<PathBuf> {
    cli_path.filter(|path| path.exists()).or_else(|| {
        if default_dir.exists() {
            Some(default_dir)
        } else {
            None
        }
    })
}

fn default_opencode_data_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        home_directory()
            .map(|home| {
                home.join("Library")
                    .join("Application Support")
                    .join("opencode")
            })
            .unwrap_or_else(|| PathBuf::from("opencode"))
    } else if cfg!(target_os = "windows") {
        env::var("APPDATA")
            .map(|value| PathBuf::from(value).join("opencode"))
            .unwrap_or_else(|_| PathBuf::from("opencode"))
    } else {
        env::var("XDG_DATA_HOME")
            .map(|value| PathBuf::from(value).join("opencode"))
            .or_else(|_| {
                home_directory()
                    .map(|home| home.join(".local").join("share").join("opencode"))
                    .ok_or(())
            })
            .unwrap_or_else(|_| PathBuf::from("opencode"))
    }
}

fn find_command_path(command_name: &str) -> Option<String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("where").arg(command_name).output().ok()?
    } else {
        Command::new("sh")
            .args(["-lc", &format!("command -v {}", command_name)])
            .output()
            .ok()?
    };
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
}

fn home_directory() -> Option<PathBuf> {
    env::var("HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var("USERPROFILE").ok().map(PathBuf::from))
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}

fn format_size_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }

    let units = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} {}", units[unit_index])
    } else {
        format!("{size:.1} {}", units[unit_index])
    }
}

fn create_stable_id(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn sanitize_filename(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "agent-session".to_string()
    } else {
        sanitized
    }
}

fn build_desktop_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let import_workspace = MenuItem::with_id(
        app,
        MENU_ID_IMPORT_WORKSPACE,
        "导入文档仓库…",
        true,
        Some("CmdOrCtrl+O"),
    )?;
    let export_workspace = MenuItem::with_id(
        app,
        MENU_ID_EXPORT_WORKSPACE,
        "导出当前文档仓库…",
        true,
        Some("CmdOrCtrl+Shift+E"),
    )?;
    let open_search = MenuItem::with_id(
        app,
        MENU_ID_OPEN_SEARCH,
        "搜索文档",
        true,
        Some("CmdOrCtrl+K"),
    )?;
    let open_settings = MenuItem::with_id(
        app,
        MENU_ID_OPEN_SETTINGS,
        "系统设置",
        true,
        Some("CmdOrCtrl+,"),
    )?;
    let open_app_data_directory = MenuItem::with_id(
        app,
        MENU_ID_OPEN_APP_DATA_DIRECTORY,
        "打开数据目录",
        true,
        None::<&str>,
    )?;
    let open_logs_directory = MenuItem::with_id(
        app,
        MENU_ID_OPEN_LOGS_DIRECTORY,
        "打开日志目录",
        true,
        None::<&str>,
    )?;
    Menu::with_items(
        app,
        &[
            #[cfg(target_os = "macos")]
            &Submenu::with_items(
                app,
                "Docs Atlas",
                true,
                &[
                    &PredefinedMenuItem::about(app, Some("关于 Docs Atlas"), None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, None)?,
                    &PredefinedMenuItem::hide_others(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?,
            &Submenu::with_items(
                app,
                "File",
                true,
                &[
                    &import_workspace,
                    &export_workspace,
                    &PredefinedMenuItem::separator(app)?,
                    &open_app_data_directory,
                    &open_logs_directory,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::close_window(app, None)?,
                    #[cfg(not(target_os = "macos"))]
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?,
            &Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?,
            &Submenu::with_items(app, "View", true, &[&open_search, &open_settings])?,
            &Submenu::with_items(
                app,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(app, None)?,
                    &PredefinedMenuItem::maximize(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::close_window(app, None)?,
                ],
            )?,
            #[cfg(not(target_os = "macos"))]
            &Submenu::with_items(
                app,
                "Help",
                true,
                &[&PredefinedMenuItem::about(
                    app,
                    Some("关于 Docs Atlas"),
                    None,
                )?],
            )?,
        ],
    )
}

fn emit_desktop_menu_action(app: &AppHandle, action: &str) {
    let _ = app.emit(
        DESKTOP_MENU_ACTION_EVENT,
        DesktopMenuActionPayload {
            action: action.to_string(),
        },
    );
    record_app_info(app, "menu.action", &format!("action={action}"));
}

fn handle_desktop_menu_event(app: &AppHandle, menu_id: &str) {
    match menu_id {
        MENU_ID_IMPORT_WORKSPACE => emit_desktop_menu_action(app, MENU_ACTION_IMPORT_WORKSPACE),
        MENU_ID_EXPORT_WORKSPACE => emit_desktop_menu_action(app, MENU_ACTION_EXPORT_WORKSPACE),
        MENU_ID_OPEN_SEARCH => emit_desktop_menu_action(app, MENU_ACTION_OPEN_SEARCH),
        MENU_ID_OPEN_SETTINGS => emit_desktop_menu_action(app, MENU_ACTION_OPEN_SETTINGS),
        MENU_ID_OPEN_APP_DATA_DIRECTORY => {
            if let Err(error) = open_app_data_directory(app.clone()) {
                record_app_error(
                    app,
                    "menu.action",
                    &format!("action=open-app-data error={error}"),
                );
            }
        }
        MENU_ID_OPEN_LOGS_DIRECTORY => {
            if let Err(error) = open_logs_directory(app.clone()) {
                record_app_error(
                    app,
                    "menu.action",
                    &format!("action=open-logs error={error}"),
                );
            }
        }
        _ => {}
    }
}

fn restore_main_window_state(app: &AppHandle) {
    let connection = match open_workspace_database(app) {
        Ok(connection) => connection,
        Err(error) => {
            record_app_error(app, "window.state", &format!("restore_db_error={error}"));
            return;
        }
    };

    let Some(saved_state) =
        read_app_setting_json::<PersistedWindowState>(&connection, APP_WINDOW_STATE_KEY)
            .ok()
            .flatten()
    else {
        return;
    };

    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if let (Some(width), Some(height)) = (saved_state.width, saved_state.height) {
        let _ = window.set_size(Size::Physical(PhysicalSize::new(width, height)));
    }

    if let (Some(x), Some(y)) = (saved_state.x, saved_state.y) {
        let _ = window.set_position(Position::Physical(PhysicalPosition::new(x, y)));
    }

    if saved_state.maximized {
        let _ = window.maximize();
    }

    record_app_info(
        app,
        "window.state",
        &format!(
            "restored maximized={} width={:?} height={:?} x={:?} y={:?}",
            saved_state.maximized,
            saved_state.width,
            saved_state.height,
            saved_state.x,
            saved_state.y
        ),
    );
}

fn snapshot_main_window_state(window: &Window) -> Result<PersistedWindowState, String> {
    let connection = open_workspace_database(window.app_handle())?;
    let mut next_state =
        read_app_setting_json::<PersistedWindowState>(&connection, APP_WINDOW_STATE_KEY)?
            .unwrap_or_default();

    let is_maximized = window.is_maximized().map_err(|error| error.to_string())?;
    next_state.maximized = is_maximized;

    if !is_maximized {
        let position = window.outer_position().map_err(|error| error.to_string())?;
        let size = window.inner_size().map_err(|error| error.to_string())?;
        next_state.x = Some(position.x);
        next_state.y = Some(position.y);
        next_state.width = Some(size.width);
        next_state.height = Some(size.height);
    }

    Ok(next_state)
}

fn persist_main_window_state(window: &Window, reason: &str, log_success: bool) {
    let app = window.app_handle();
    let state = match snapshot_main_window_state(window) {
        Ok(state) => state,
        Err(error) => {
            record_app_error(
                app,
                "window.state",
                &format!("snapshot_error reason={reason} error={error}"),
            );
            return;
        }
    };

    let connection = match open_workspace_database(app) {
        Ok(connection) => connection,
        Err(error) => {
            record_app_error(
                app,
                "window.state",
                &format!("persist_db_error reason={reason} error={error}"),
            );
            return;
        }
    };

    if let Err(error) = write_app_setting_json(&connection, APP_WINDOW_STATE_KEY, &state) {
        record_app_error(
            app,
            "window.state",
            &format!("persist_error reason={reason} error={error}"),
        );
        return;
    }

    if log_success {
        record_app_info(
            app,
            "window.state",
            &format!(
                "persisted reason={} maximized={} width={:?} height={:?} x={:?} y={:?}",
                reason, state.maximized, state.width, state.height, state.x, state.y
            ),
        );
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .menu(build_desktop_menu)
        .on_menu_event(|app, event| {
            handle_desktop_menu_event(app, event.id().as_ref());
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }

            match event {
                WindowEvent::Focused(false) => persist_main_window_state(window, "blur", false),
                WindowEvent::CloseRequested { .. } => {
                    persist_main_window_state(window, "close-requested", true)
                }
                WindowEvent::Destroyed => persist_main_window_state(window, "destroyed", false),
                _ => {}
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            restore_main_window_state(&app_handle);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_background_color(Some(tauri::window::Color(31, 84, 217, 255)));
            }
            Ok(())
        })
        .manage(WorkspaceSourceWatchState::default())
        .invoke_handler(tauri::generate_handler![
            create_agent_session_delete_plan,
            delete_workspace,
            execute_agent_session_delete_plan,
            export_workspace_config,
            export_logs_file,
            get_default_docs_path,
            import_workspace_config,
            list_workspace_details,
            mark_workspace_opened,
            open_agent_session_backups_directory,
            open_external_url,
            open_app_data_directory,
            open_logs_directory,
            pick_folder_path,
            pick_folder_paths,
            scan_agent_sessions,
            save_markdown_document,
            scan_video_directory,
            scan_workspace_sources,
            set_window_background_color,
            unwatch_workspace_sources,
            validate_source_path,
            watch_workspace_sources,
            upsert_workspace
        ])
        .run(tauri::generate_context!())
        .expect("error while running Docs Atlas desktop application");
}

fn open_workspace_database(app: &AppHandle) -> Result<Connection, String> {
    let data_directory = resolve_app_data_directory(app)?;
    std::fs::create_dir_all(&data_directory).map_err(|error| error.to_string())?;

    let database_path = data_directory.join("docs-atlas.db");
    let mut connection = Connection::open(database_path).map_err(|error| error.to_string())?;
    connection
        .execute_batch("pragma foreign_keys = on;")
        .map_err(|error| error.to_string())?;
    connection
        .execute_batch(WORKSPACE_DB_SCHEMA)
        .map_err(|error| error.to_string())?;
    migrate_workspace_database(&connection)?;
    maybe_migrate_legacy_seed_workspaces(app, &mut connection)?;
    Ok(connection)
}

fn resolve_app_data_directory(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|error| error.to_string())
}

fn ensure_logs_directory(app: &AppHandle) -> Result<PathBuf, String> {
    let logs_directory = resolve_app_data_directory(app)?.join(APP_LOGS_DIR_NAME);
    std::fs::create_dir_all(&logs_directory).map_err(|error| error.to_string())?;
    Ok(logs_directory)
}

fn ensure_log_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let log_file_path = ensure_logs_directory(app)?.join(APP_LOG_FILE_NAME);
    if !log_file_path.exists() {
        std::fs::write(&log_file_path, "").map_err(|error| error.to_string())?;
    }
    Ok(log_file_path)
}

fn parse_hex_color(color: &str) -> Result<tauri::window::Color, String> {
    let trimmed = color.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);

    match hex.len() {
        6 => {
            let red = u8::from_str_radix(&hex[0..2], 16).map_err(|error| error.to_string())?;
            let green = u8::from_str_radix(&hex[2..4], 16).map_err(|error| error.to_string())?;
            let blue = u8::from_str_radix(&hex[4..6], 16).map_err(|error| error.to_string())?;
            Ok(tauri::window::Color(red, green, blue, 255))
        }
        8 => {
            let red = u8::from_str_radix(&hex[0..2], 16).map_err(|error| error.to_string())?;
            let green = u8::from_str_radix(&hex[2..4], 16).map_err(|error| error.to_string())?;
            let blue = u8::from_str_radix(&hex[4..6], 16).map_err(|error| error.to_string())?;
            let alpha = u8::from_str_radix(&hex[6..8], 16).map_err(|error| error.to_string())?;
            Ok(tauri::window::Color(red, green, blue, alpha))
        }
        _ => Err(format!("unsupported color format: {trimmed}")),
    }
}

fn append_app_log(app: &AppHandle, level: &str, scope: &str, message: &str) -> Result<(), String> {
    let log_file_path = ensure_log_file_path(app)?;
    let mut log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .map_err(|error| error.to_string())?;

    writeln!(
        log_file,
        "[{}] {:<5} {} {}",
        current_timestamp(),
        level.to_uppercase(),
        scope,
        sanitize_log_message(message)
    )
    .map_err(|error| error.to_string())
}

fn sanitize_log_message(message: &str) -> String {
    message.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn record_app_info(app: &AppHandle, scope: &str, message: &str) {
    let _ = append_app_log(app, "info", scope, message);
}

fn record_app_error(app: &AppHandle, scope: &str, message: &str) {
    let _ = append_app_log(app, "error", scope, message);
}

fn open_path_in_file_manager(path: &Path) -> Result<(), String> {
    let mut command = if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(path);
        command
    } else if cfg!(target_os = "windows") {
        let mut command = Command::new("explorer");
        command.arg(path);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };

    command
        .status()
        .map_err(|error| error.to_string())
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("failed to open path {}", path.to_string_lossy()))
            }
        })
}

fn open_url_in_browser(url: &str) -> Result<(), String> {
    let mut command = if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(url);
        command
    } else if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", url]);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    };

    command
        .status()
        .map_err(|error| error.to_string())
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("failed to open url {url}"))
            }
        })
}

fn migrate_workspace_database(connection: &Connection) -> Result<(), String> {
    add_column_if_missing(
        connection,
        "alter table workspaces add column default_search_scope text not null default 'global'",
    )?;
    add_column_if_missing(
        connection,
        "alter table workspaces add column sort_order integer not null default 0",
    )?;
    Ok(())
}

fn add_column_if_missing(connection: &Connection, statement: &str) -> Result<(), String> {
    match connection.execute(statement, []) {
        Ok(_) => Ok(()),
        Err(error) => {
            let message = error.to_string();
            if message.contains("duplicate column name") {
                Ok(())
            } else {
                Err(message)
            }
        }
    }
}

fn maybe_migrate_legacy_seed_workspaces(
    app: &AppHandle,
    connection: &mut Connection,
) -> Result<(), String> {
    let saved_seed_version = read_app_setting_string(connection, DESKTOP_SEED_VERSION_KEY)?;
    if saved_seed_version.as_deref() == Some(CURRENT_DESKTOP_SEED_VERSION) {
        return Ok(());
    }

    if should_reset_legacy_seed_workspaces(app, connection)? {
        reset_to_default_workspace(app, connection)?;
    }

    write_app_setting_string(
        connection,
        DESKTOP_SEED_VERSION_KEY,
        CURRENT_DESKTOP_SEED_VERSION,
    )?;
    Ok(())
}

fn should_reset_legacy_seed_workspaces(
    app: &AppHandle,
    connection: &Connection,
) -> Result<bool, String> {
    let workspaces = list_workspace_summaries(connection)?;
    if workspaces.is_empty() {
        return Ok(false);
    }

    if workspaces.len() == 1 && workspaces[0].id == "workspace:default" {
        let detail = load_workspace_detail(connection, workspaces[0].clone())?;
        return Ok(!matches_current_default_workspace(
            &detail,
            &resolve_default_docs_path(app),
        ));
    }

    for workspace in workspaces {
        if is_legacy_workspace_id(&workspace.id) {
            continue;
        }

        let detail = load_workspace_detail(connection, workspace)?;
        if !detail
            .sources
            .iter()
            .any(|source| contains_legacy_seed_marker(source))
        {
            return Ok(false);
        }
    }

    Ok(true)
}

fn reset_to_default_workspace(app: &AppHandle, connection: &mut Connection) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let now = current_timestamp();
    let default_docs_path = resolve_default_docs_path(app);

    transaction
        .execute("delete from workspace_source_nodes", [])
        .map_err(|error| error.to_string())?;
    transaction
        .execute("delete from recent_workspace_entries", [])
        .map_err(|error| error.to_string())?;
    transaction
        .execute("delete from workspaces", [])
        .map_err(|error| error.to_string())?;

    transaction
        .execute(
            r#"
      insert into workspaces (
        id,
        name,
        description,
        icon,
        color,
        default_search_scope,
        sort_order,
        created_at,
        updated_at,
        last_opened_at
      )
      values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
      "#,
            params![
                "workspace:default",
                "项目文档",
                "默认文档仓库，指向内置示例文档目录。",
                "folder",
                "#1f54d9",
                "workspace",
                0,
                now,
                now,
                now
            ],
        )
        .map_err(|error| error.to_string())?;

    replace_workspace_source_nodes(
        &transaction,
        "workspace:default",
        vec![WorkspaceSourceNodeInput {
            id: "node:project-docs".to_string(),
            parent_id: None,
            kind: "folder".to_string(),
            name: "项目文档".to_string(),
            path: Some(default_docs_path),
            enabled: Some(true),
            position: Some(0),
            children: Some(Vec::new()),
        }],
    )?;

    transaction.commit().map_err(|error| error.to_string())
}

fn list_workspace_summaries(connection: &Connection) -> Result<Vec<WorkspaceSummaryRow>, String> {
    let mut statement = connection
        .prepare(
            r#"
      select
        id,
        name,
        description,
        icon,
        color,
        default_search_scope,
        sort_order,
        created_at,
        updated_at,
        last_opened_at
      from workspaces
      order by sort_order asc, name asc
      "#,
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            Ok(WorkspaceSummaryRow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                default_search_scope: row.get(5)?,
                sort_order: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_opened_at: row.get(9)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn matches_current_default_workspace(
    workspace: &WorkspaceDetailPayload,
    default_docs_path: &str,
) -> bool {
    if workspace.name != "项目文档" || workspace.sources.len() != 1 {
        return false;
    }

    let source = &workspace.sources[0];
    source.id == "node:project-docs"
        && source.kind == "folder"
        && source.name == "项目文档"
        && normalize_path(&source.path) == normalize_path(default_docs_path)
        && source.children.is_empty()
}

fn contains_legacy_seed_marker(source: &WorkspaceSourceNodePayload) -> bool {
    if is_legacy_source_id(&source.id)
        || matches!(
            source.name.as_str(),
            "AI-Agent" | "Another Project" | "Local Workspace"
        )
    {
        return true;
    }

    let normalized_path = normalize_path(&source.path);
    if normalized_path.contains("config.yaml") || normalized_path.contains("config.yml") {
        return true;
    }

    source
        .children
        .iter()
        .any(|child| contains_legacy_seed_marker(child))
}

fn read_app_setting_string(connection: &Connection, key: &str) -> Result<Option<String>, String> {
    read_app_setting_json(connection, key)
}

fn write_app_setting_string(connection: &Connection, key: &str, value: &str) -> Result<(), String> {
    write_app_setting_json(connection, key, &value.to_string())
}

fn read_app_setting_json<T>(connection: &Connection, key: &str) -> Result<Option<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    let raw_value = connection
        .query_row(
            "select value_json from app_settings where key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    match raw_value {
        Some(value) => serde_json::from_str::<T>(&value)
            .map(Some)
            .map_err(|error| error.to_string()),
        None => Ok(None),
    }
}

fn write_app_setting_json<T>(connection: &Connection, key: &str, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    let now = current_timestamp();
    let value_json = serde_json::to_string(value).map_err(|error| error.to_string())?;

    connection
        .execute(
            r#"
      insert into app_settings (key, value_json, updated_at)
      values (?1, ?2, ?3)
      on conflict(key) do update set
        value_json = excluded.value_json,
        updated_at = excluded.updated_at
      "#,
            params![key, value_json, now],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn resolve_default_docs_path(app: &AppHandle) -> String {
    if let Some(path) = resolve_bundled_default_docs_path(app) {
        return path;
    }

    normalize_existing_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../docs"))
}

fn resolve_bundled_default_docs_path(app: &AppHandle) -> Option<String> {
    let resource_dir = app.path().resource_dir().ok()?;
    find_docs_root_in_resource_dir(&resource_dir, 4).map(normalize_existing_path)
}

fn find_docs_root_in_resource_dir(path: &Path, remaining_depth: usize) -> Option<PathBuf> {
    if is_docs_root_directory(path) {
        return Some(path.to_path_buf());
    }

    let docs_path = path.join(BUNDLED_DEFAULT_DOCS_DIR_NAME);
    if is_docs_root_directory(&docs_path) {
        return Some(docs_path);
    }

    if remaining_depth == 0 {
        return None;
    }

    let entries = std::fs::read_dir(path).ok()?;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        if let Some(found_path) = find_docs_root_in_resource_dir(&entry_path, remaining_depth - 1) {
            return Some(found_path);
        }
    }

    None
}

fn is_docs_root_directory(path: &Path) -> bool {
    path.is_dir()
        && DEFAULT_DOCS_SENTINELS
            .iter()
            .all(|relative_path| path.join(relative_path).exists())
}

fn normalize_existing_path(path: PathBuf) -> String {
    match path.canonicalize() {
        Ok(resolved) => resolved.to_string_lossy().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    }
}

fn normalize_path(value: &str) -> String {
    value.replace('\\', "/").trim().to_lowercase()
}

fn sanitize_workspace_filename(value: &str) -> String {
    let sanitized = value
        .trim()
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            _ if character.is_control() => '-',
            _ => character,
        })
        .collect::<String>();

    if sanitized.trim().is_empty() {
        "workspace".to_string()
    } else {
        sanitized
    }
}

fn is_legacy_workspace_id(value: &str) -> bool {
    matches!(
        value,
        "workspace:atlas" | "workspace:product" | "workspace:ai"
    )
}

fn is_legacy_source_id(value: &str) -> bool {
    matches!(value, "source:atlas" | "source:product" | "source:ai")
}

fn load_workspace_summary(
    connection: &Connection,
    workspace_id: &str,
) -> Result<WorkspaceSummaryRow, String> {
    connection
        .query_row(
            r#"
      select
        id,
        name,
        description,
        icon,
        color,
        default_search_scope,
        sort_order,
        created_at,
        updated_at,
        last_opened_at
      from workspaces
      where id = ?1
      "#,
            params![workspace_id],
            |row| {
                Ok(WorkspaceSummaryRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon: row.get(3)?,
                    color: row.get(4)?,
                    default_search_scope: row.get(5)?,
                    sort_order: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    last_opened_at: row.get(9)?,
                })
            },
        )
        .map_err(|error| error.to_string())
}

fn load_workspace_detail(
    connection: &Connection,
    workspace: WorkspaceSummaryRow,
) -> Result<WorkspaceDetailPayload, String> {
    let mut statement = connection
        .prepare(
            r#"
      select
        id,
        workspace_id,
        parent_id,
        kind,
        name,
        path,
        enabled,
        position
      from workspace_source_nodes
      where workspace_id = ?1
      order by position asc, name asc
      "#,
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map(params![workspace.id.clone()], |row| {
            Ok(WorkspaceSourceNodeRow {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                parent_id: row.get(2)?,
                kind: row.get(3)?,
                name: row.get(4)?,
                path: row.get(5)?,
                enabled: row.get::<_, i64>(6)? == 1,
                position: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let sources = build_source_tree(
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?,
    );

    Ok(WorkspaceDetailPayload {
        id: workspace.id,
        name: workspace.name,
        description: workspace.description,
        icon: workspace.icon,
        color: workspace.color,
        default_search_scope: workspace.default_search_scope,
        sort_order: workspace.sort_order,
        created_at: workspace.created_at,
        updated_at: workspace.updated_at,
        last_opened_at: workspace.last_opened_at,
        sources,
    })
}

fn replace_workspace_source_nodes(
    connection: &Connection,
    workspace_id: &str,
    nodes: Vec<WorkspaceSourceNodeInput>,
) -> Result<(), String> {
    connection
        .execute(
            "delete from workspace_source_nodes where workspace_id = ?1",
            params![workspace_id],
        )
        .map_err(|error| error.to_string())?;

    let flattened = flatten_source_nodes(workspace_id, None, nodes);
    let now = current_timestamp();

    for node in flattened {
        connection
            .execute(
                r#"
        insert into workspace_source_nodes (
          id,
          workspace_id,
          parent_id,
          kind,
          name,
          path,
          enabled,
          position,
          created_at,
          updated_at
        )
        values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
                params![
                    node.id,
                    node.workspace_id,
                    node.parent_id,
                    node.kind,
                    node.name,
                    node.path,
                    if node.enabled { 1 } else { 0 },
                    node.position,
                    now,
                    now
                ],
            )
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn flatten_source_nodes(
    workspace_id: &str,
    parent_id: Option<String>,
    nodes: Vec<WorkspaceSourceNodeInput>,
) -> Vec<WorkspaceSourceNodeRow> {
    nodes
        .into_iter()
        .enumerate()
        .flat_map(|(index, node)| {
            let current = WorkspaceSourceNodeRow {
                id: node.id.clone(),
                workspace_id: workspace_id.to_string(),
                parent_id: node.parent_id.clone().or_else(|| parent_id.clone()),
                kind: node.kind.clone(),
                name: node.name.clone(),
                path: node.path.clone().unwrap_or_default(),
                enabled: node.enabled.unwrap_or(true),
                position: node.position.unwrap_or(index as i64),
            };

            let children = flatten_source_nodes(
                workspace_id,
                Some(node.id),
                node.children.unwrap_or_default(),
            );
            std::iter::once(current)
                .chain(children.into_iter())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn build_source_tree(rows: Vec<WorkspaceSourceNodeRow>) -> Vec<WorkspaceSourceNodePayload> {
    let by_parent = rows.iter().fold(
        std::collections::HashMap::<Option<String>, Vec<WorkspaceSourceNodeRow>>::new(),
        |mut groups, row| {
            groups
                .entry(row.parent_id.clone())
                .or_default()
                .push(row.clone());
            groups
        },
    );

    let mut roots = build_source_children(None, &by_parent);
    sort_source_nodes(&mut roots);
    roots
}

fn build_source_children(
    parent_id: Option<String>,
    by_parent: &std::collections::HashMap<Option<String>, Vec<WorkspaceSourceNodeRow>>,
) -> Vec<WorkspaceSourceNodePayload> {
    by_parent
        .get(&parent_id)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| WorkspaceSourceNodePayload {
            id: row.id.clone(),
            workspace_id: row.workspace_id.clone(),
            parent_id: row.parent_id.clone(),
            kind: row.kind.clone(),
            name: row.name.clone(),
            path: row.path.clone(),
            enabled: row.enabled,
            position: row.position,
            children: build_source_children(Some(row.id), by_parent),
        })
        .collect()
}

fn sort_source_nodes(nodes: &mut Vec<WorkspaceSourceNodePayload>) {
    nodes.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    for node in nodes {
        sort_source_nodes(&mut node.children);
    }
}

fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("{}", duration.as_secs()),
        Err(_) => "0".to_string(),
    }
}

fn system_time_to_string(value: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    value
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn current_unix_nanos() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn generate_import_id(prefix: &str) -> String {
    let counter = IMPORT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}:{}-{counter}", current_unix_nanos())
}

fn insert_imported_workspace(
    app: &AppHandle,
    payload: WorkspaceTransferPayload,
) -> Result<WorkspaceDetailPayload, String> {
    let mut connection = open_workspace_database(app)?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let workspace_id = generate_import_id("workspace");
    let now = current_timestamp();
    let sort_order = transaction
        .query_row(
            "select coalesce(max(sort_order), -1) + 1 from workspaces",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?;

    transaction
        .execute(
            r#"
      insert into workspaces (
        id,
        name,
        description,
        icon,
        color,
        default_search_scope,
        sort_order,
        created_at,
        updated_at,
        last_opened_at
      )
      values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
      "#,
            params![
                &workspace_id,
                normalize_imported_workspace_name(&payload.workspace.name),
                payload.workspace.description,
                if payload.workspace.icon.trim().is_empty() {
                    "folder".to_string()
                } else {
                    payload.workspace.icon
                },
                if payload.workspace.color.trim().is_empty() {
                    "#1f54d9".to_string()
                } else {
                    payload.workspace.color
                },
                normalize_imported_search_scope(&payload.workspace.default_search_scope),
                sort_order,
                now,
                now,
                now
            ],
        )
        .map_err(|error| error.to_string())?;

    let sources = rekey_imported_source_nodes(payload.workspace.sources, None);
    replace_workspace_source_nodes(&transaction, &workspace_id, sources)?;
    transaction.commit().map_err(|error| error.to_string())?;

    let connection = open_workspace_database(app)?;
    let summary = load_workspace_summary(&connection, &workspace_id)?;
    load_workspace_detail(&connection, summary)
}

fn normalize_imported_workspace_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "导入的文档仓库".to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_imported_search_scope(value: &str) -> String {
    match value {
        "workspace" => "workspace".to_string(),
        _ => "global".to_string(),
    }
}

fn export_workspace_sources(
    nodes: Vec<WorkspaceSourceNodePayload>,
) -> Vec<WorkspaceSourceNodeInput> {
    nodes
        .into_iter()
        .map(|node| WorkspaceSourceNodeInput {
            id: node.id,
            parent_id: node.parent_id,
            kind: node.kind,
            name: node.name,
            path: Some(node.path),
            enabled: Some(node.enabled),
            position: Some(node.position),
            children: Some(export_workspace_sources(node.children)),
        })
        .collect()
}

fn rekey_imported_source_nodes(
    nodes: Vec<WorkspaceSourceNodeInput>,
    parent_id: Option<String>,
) -> Vec<WorkspaceSourceNodeInput> {
    nodes
        .into_iter()
        .enumerate()
        .map(|(index, node)| {
            let next_id = generate_import_id("source-node");
            let is_folder = node.kind == "folder";
            let children = rekey_imported_source_nodes(
                node.children.unwrap_or_default(),
                Some(next_id.clone()),
            );

            WorkspaceSourceNodeInput {
                id: next_id,
                parent_id: parent_id.clone(),
                kind: node.kind,
                name: node.name,
                path: Some(if is_folder {
                    node.path.unwrap_or_default()
                } else {
                    "".to_string()
                }),
                enabled: Some(node.enabled.unwrap_or(true)),
                position: Some(node.position.unwrap_or(index as i64)),
                children: Some(children),
            }
        })
        .collect()
}

fn stop_workspace_sources_watch(state: &WorkspaceSourceWatchState) {
    if let Ok(mut active_stop_signal) = state.active_stop_signal.lock() {
        if let Some(stop_signal) = active_stop_signal.take() {
            stop_signal.store(true, Ordering::Relaxed);
        }
    }
}

fn spawn_workspace_sources_watch(
    app: AppHandle,
    workspace_id: String,
    sources: Vec<EnabledFolderSource>,
    stop_signal: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let mut previous_fingerprint = build_workspace_sources_watch_fingerprint(&sources);

        loop {
            if stop_signal.load(Ordering::Relaxed) {
                return;
            }

            std::thread::sleep(Duration::from_millis(WORKSPACE_SOURCE_WATCH_INTERVAL_MS));
            if stop_signal.load(Ordering::Relaxed) {
                return;
            }

            let next_fingerprint = build_workspace_sources_watch_fingerprint(&sources);
            if next_fingerprint == previous_fingerprint {
                continue;
            }

            previous_fingerprint = next_fingerprint;
            let _ = app.emit(
                WORKSPACE_SOURCES_CHANGED_EVENT,
                WorkspaceSourceWatchEventPayload {
                    workspace_id: workspace_id.clone(),
                    detected_at: current_timestamp(),
                },
            );
        }
    });
}

#[derive(Debug, Clone)]
struct EnabledFolderSource {
    id: String,
    path: String,
}

fn scan_single_source(
    connection: &Connection,
    source: &EnabledFolderSource,
    checked_at: &str,
) -> Result<
    (
        Vec<WorkspaceSourceDocumentPayload>,
        WorkspaceSourceStatusPayload,
    ),
    String,
> {
    let root = PathBuf::from(&source.path);
    let root_key = resolve_source_root_key(&root, &source.path);

    let metadata = match std::fs::metadata(&root) {
        Ok(metadata) => metadata,
        Err(_) => {
            return Ok((
                Vec::new(),
                WorkspaceSourceStatusPayload {
                    source_node_id: source.id.clone(),
                    source_root: source.path.clone(),
                    state: "missing".to_string(),
                    message: "目录不存在".to_string(),
                    document_count: 0,
                    used_cache: false,
                    checked_at: checked_at.to_string(),
                },
            ));
        }
    };

    if !metadata.is_dir() {
        return Ok((
            Vec::new(),
            WorkspaceSourceStatusPayload {
                source_node_id: source.id.clone(),
                source_root: source.path.clone(),
                state: "not_directory".to_string(),
                message: "路径不是目录".to_string(),
                document_count: 0,
                used_cache: false,
                checked_at: checked_at.to_string(),
            },
        ));
    }

    let file_snapshots = collect_markdown_file_snapshots(&root, &root)?;
    if file_snapshots.is_empty() {
        return Ok((
            Vec::new(),
            WorkspaceSourceStatusPayload {
                source_node_id: source.id.clone(),
                source_root: source.path.clone(),
                state: "empty".to_string(),
                message: "目录下没有 Markdown 文档".to_string(),
                document_count: 0,
                used_cache: false,
                checked_at: checked_at.to_string(),
            },
        ));
    }

    let fingerprint = build_markdown_fingerprint(&file_snapshots);
    if let Some(cached_payload) = read_source_scan_cache(connection, &root_key, &fingerprint)? {
        let documents = cached_payload
            .documents
            .into_iter()
            .map(|document| WorkspaceSourceDocumentPayload {
                source_node_id: source.id.clone(),
                source_root: source.path.clone(),
                absolute_path: document.absolute_path,
                relative_path: document.relative_path,
                modified_at: document.modified_at,
                markdown: document.markdown,
            })
            .collect::<Vec<_>>();

        return Ok((
            documents.clone(),
            WorkspaceSourceStatusPayload {
                source_node_id: source.id.clone(),
                source_root: source.path.clone(),
                state: "ready".to_string(),
                message: format!("已加载 {} 篇文档（缓存）", documents.len()),
                document_count: documents.len(),
                used_cache: true,
                checked_at: checked_at.to_string(),
            },
        ));
    }

    let documents = file_snapshots
        .iter()
        .map(|snapshot| {
            let markdown = std::fs::read_to_string(&snapshot.absolute_path)
                .map_err(|error| error.to_string())?;
            Ok(WorkspaceSourceDocumentPayload {
                source_node_id: source.id.clone(),
                source_root: source.path.clone(),
                absolute_path: snapshot.absolute_path.to_string_lossy().to_string(),
                relative_path: snapshot.relative_path.clone(),
                modified_at: snapshot.modified_at,
                markdown,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    write_source_scan_cache(
        connection,
        &root_key,
        &fingerprint,
        &CachedWorkspaceSourcePayload {
            documents: documents.clone(),
        },
    )?;

    Ok((
        documents.clone(),
        WorkspaceSourceStatusPayload {
            source_node_id: source.id.clone(),
            source_root: source.path.clone(),
            state: "ready".to_string(),
            message: format!("已扫描 {} 篇文档", documents.len()),
            document_count: documents.len(),
            used_cache: false,
            checked_at: checked_at.to_string(),
        },
    ))
}

fn collect_enabled_folder_sources(
    parent_enabled: Option<bool>,
    nodes: Vec<WorkspaceSourceNodeInput>,
    include_children: bool,
) -> Vec<EnabledFolderSource> {
    let mut sources = Vec::<EnabledFolderSource>::new();

    for node in nodes {
        let is_enabled = parent_enabled.unwrap_or(true) && node.enabled.unwrap_or(true);
        if !is_enabled {
            continue;
        }

        if node.kind == "folder" {
            let path = node.path.clone().unwrap_or_default();
            if !path.trim().is_empty() {
                sources.push(EnabledFolderSource {
                    id: node.id.clone(),
                    path,
                });
            }
        }

        if include_children {
            sources.extend(collect_enabled_folder_sources(
                Some(is_enabled),
                node.children.unwrap_or_default(),
                true,
            ));
        }
    }

    sources
}

fn collect_markdown_file_snapshots(
    root: &Path,
    base_root: &Path,
) -> Result<Vec<MarkdownFileSnapshot>, String> {
    let mut files = Vec::<MarkdownFileSnapshot>::new();
    let entries = std::fs::read_dir(root).map_err(|error| error.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;

        if file_type.is_dir() {
            files.extend(collect_markdown_file_snapshots(&path, base_root)?);
            continue;
        }

        if file_type.is_file() && is_markdown_file(&path) {
            let relative_path = path
                .strip_prefix(base_root)
                .map_err(|error| error.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let metadata = entry.metadata().map_err(|error| error.to_string())?;
            let modified_at = metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|value| value.as_secs())
                .unwrap_or(0);

            files.push(MarkdownFileSnapshot {
                absolute_path: path,
                relative_path,
                modified_at,
                size: metadata.len(),
            });
        }
    }

    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    Ok(files)
}

fn build_workspace_sources_watch_fingerprint(sources: &[EnabledFolderSource]) -> String {
    let mut hasher = DefaultHasher::new();
    sources.len().hash(&mut hasher);

    for source in sources {
        source.id.hash(&mut hasher);
        normalize_path(&source.path).hash(&mut hasher);

        let root = PathBuf::from(&source.path);
        let root_key = resolve_source_root_key(&root, &source.path);
        root_key.hash(&mut hasher);

        match std::fs::metadata(&root) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    "not_directory".hash(&mut hasher);
                    metadata.len().hash(&mut hasher);
                    continue;
                }

                match collect_markdown_file_snapshots(&root, &root) {
                    Ok(files) => {
                        "ready".hash(&mut hasher);
                        build_markdown_fingerprint(&files).hash(&mut hasher);
                    }
                    Err(error) => {
                        "scan_error".hash(&mut hasher);
                        error.hash(&mut hasher);
                    }
                }
            }
            Err(error) => {
                "missing".hash(&mut hasher);
                error.to_string().hash(&mut hasher);
            }
        }
    }

    format!("{:016x}", hasher.finish())
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

fn collect_video_tree_nodes(
    root: &Path,
    base_root: &Path,
) -> Result<Vec<VideoTreeNodePayload>, String> {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()),
    };
    let mut nodes = Vec::<VideoTreeNodePayload>::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            let children = collect_video_tree_nodes(&path, base_root)?;
            if children.is_empty() {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            nodes.push(VideoTreeNodePayload {
                id: build_video_node_id(base_root, &path),
                name: path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("未命名目录")
                    .to_string(),
                path: path.to_string_lossy().to_string(),
                relative_path: path
                    .strip_prefix(base_root)
                    .map_err(|error| error.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/"),
                kind: "folder".to_string(),
                size: 0,
                modified_at: metadata_modified_at(&metadata),
                children,
            });
            continue;
        }

        if file_type.is_file() && is_video_file(&path) {
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            nodes.push(VideoTreeNodePayload {
                id: build_video_node_id(base_root, &path),
                name: path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("未命名视频")
                    .to_string(),
                path: path.to_string_lossy().to_string(),
                relative_path: path
                    .strip_prefix(base_root)
                    .map_err(|error| error.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/"),
                kind: "file".to_string(),
                size: metadata.len(),
                modified_at: metadata_modified_at(&metadata),
                children: Vec::new(),
            });
        }
    }

    nodes.sort_by(|left, right| {
        if left.kind != right.kind {
            return video_node_kind_rank(&left.kind).cmp(&video_node_kind_rank(&right.kind));
        }

        left.name.to_lowercase().cmp(&right.name.to_lowercase())
    });

    Ok(nodes)
}

fn build_video_node_id(base_root: &Path, path: &Path) -> String {
    path.strip_prefix(base_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn count_video_tree_files(nodes: &[VideoTreeNodePayload]) -> usize {
    nodes
        .iter()
        .map(|node| {
            if node.kind == "file" {
                1
            } else {
                count_video_tree_files(&node.children)
            }
        })
        .sum()
}

fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| {
            matches!(
                value.to_lowercase().as_str(),
                "mp4" | "m4v" | "mov" | "webm" | "mkv" | "avi" | "wmv" | "flv" | "mpeg" | "mpg"
            )
        })
        .unwrap_or(false)
}

fn video_node_kind_rank(kind: &str) -> u8 {
    if kind == "folder" {
        0
    } else {
        1
    }
}

fn metadata_modified_at(metadata: &std::fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn build_markdown_fingerprint(files: &[MarkdownFileSnapshot]) -> String {
    let mut hasher = DefaultHasher::new();
    files.len().hash(&mut hasher);

    for file in files {
        file.relative_path.hash(&mut hasher);
        file.modified_at.hash(&mut hasher);
        file.size.hash(&mut hasher);
    }

    format!("{:016x}", hasher.finish())
}

fn resolve_source_root_key(root: &Path, fallback: &str) -> String {
    root.canonicalize()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|_| fallback.to_string())
}

fn read_source_scan_cache(
    connection: &Connection,
    source_root: &str,
    fingerprint: &str,
) -> Result<Option<CachedWorkspaceSourcePayload>, String> {
    let raw_payload = connection
        .query_row(
            r#"
      select payload_json
      from workspace_source_scan_cache
      where source_root = ?1 and fingerprint = ?2
      "#,
            params![source_root, fingerprint],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    match raw_payload {
        Some(payload) => serde_json::from_str::<CachedWorkspaceSourcePayload>(&payload)
            .map(Some)
            .map_err(|error| error.to_string()),
        None => Ok(None),
    }
}

fn write_source_scan_cache(
    connection: &Connection,
    source_root: &str,
    fingerprint: &str,
    payload: &CachedWorkspaceSourcePayload,
) -> Result<(), String> {
    let payload_json = serde_json::to_string(payload).map_err(|error| error.to_string())?;
    let now = current_timestamp();

    connection
        .execute(
            r#"
      insert into workspace_source_scan_cache (source_root, fingerprint, payload_json, updated_at)
      values (?1, ?2, ?3, ?4)
      on conflict(source_root) do update set
        fingerprint = excluded.fingerprint,
        payload_json = excluded.payload_json,
        updated_at = excluded.updated_at
      "#,
            params![source_root, fingerprint, payload_json, now],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_session_path_protection_rejects_config_files() {
        assert!(is_protected_agent_session_path(
            "codex",
            Path::new("/tmp/.codex/auth.json")
        ));
        assert!(is_protected_agent_session_path(
            "claude-code",
            Path::new("/tmp/.claude/settings.json")
        ));
        assert!(!is_protected_agent_session_path(
            "codex",
            Path::new("/tmp/.codex/sessions/example.jsonl")
        ));
        assert!(!is_protected_agent_session_path(
            "claude-code",
            Path::new("/tmp/.claude/projects/example/session.jsonl")
        ));
    }

    #[test]
    fn codex_delete_plan_adds_manual_consistency_items_when_index_files_exist() {
        let temp_root =
            env::temp_dir().join(format!("docs-atlas-codex-test-{}", current_unix_nanos()));
        fs::create_dir_all(temp_root.join("sessions")).unwrap();
        fs::write(temp_root.join("sessions").join("session.jsonl"), "{}\n").unwrap();
        fs::write(temp_root.join("state_5.sqlite"), "db").unwrap();
        fs::write(temp_root.join("session_index.jsonl"), "{}\n").unwrap();
        env::set_var("CODEX_HOME", &temp_root);

        let scanned_at = current_timestamp();
        let scan = scan_codex_sessions(&scanned_at);
        assert_eq!(scan.sessions.len(), 1);

        let items = create_agent_session_plan_items(&scan.sessions[0]);
        assert!(items.iter().any(|item| item.action == "delete-file"));
        assert!(items
            .iter()
            .any(|item| item.action == "manual" && item.target.ends_with("state_5.sqlite")));
        assert!(items
            .iter()
            .any(|item| item.action == "manual" && item.target.ends_with("session_index.jsonl")));

        env::remove_var("CODEX_HOME");
        let _ = fs::remove_dir_all(temp_root);
    }

    #[test]
    fn codex_session_scan_reads_project_from_session_meta_payload() {
        let temp_root = env::temp_dir().join(format!(
            "docs-atlas-codex-meta-test-{}",
            current_unix_nanos()
        ));
        let project_path = "/Users/xiangliu/Downloads/embed-pdf-viewer-main";
        fs::create_dir_all(temp_root.join("sessions")).unwrap();
        fs::write(
            temp_root.join("sessions").join("session.jsonl"),
            format!(
                "{{\"timestamp\":\"2026-06-02T01:28:15.381Z\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"019e85f1-b0d1-7c91-ba11-17535c256e6e\",\"cwd\":\"{project_path}\",\"originator\":\"codex-tui\"}}}}\n"
            ),
        )
        .unwrap();
        env::set_var("CODEX_HOME", &temp_root);

        let scan = scan_codex_sessions(&current_timestamp());
        assert_eq!(scan.sessions.len(), 1);
        assert_eq!(scan.sessions[0].id, "019e85f1-b0d1-7c91-ba11-17535c256e6e");
        assert_eq!(scan.sessions[0].project_path, project_path);

        env::remove_var("CODEX_HOME");
        let _ = fs::remove_dir_all(temp_root);
    }

    #[test]
    fn codex_session_scan_includes_archived_sessions_directory() {
        let temp_root = env::temp_dir().join(format!(
            "docs-atlas-codex-archived-test-{}",
            current_unix_nanos()
        ));
        fs::create_dir_all(temp_root.join("archived_sessions")).unwrap();
        fs::write(
            temp_root.join("archived_sessions").join("session.jsonl"),
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"archived-session-id\",\"cwd\":\"/Users/xiangliu/Projects/archived-demo\"}}\n",
        )
        .unwrap();
        env::set_var("CODEX_HOME", &temp_root);

        let scan = scan_codex_sessions(&current_timestamp());
        assert_eq!(scan.sessions.len(), 1);
        assert_eq!(scan.sessions[0].id, "archived-session-id");
        assert_eq!(scan.sessions[0].status, "归档会话，高风险");
        assert!(scan.sessions[0]
            .metadata
            .iter()
            .any(|item| item == "archived_sessions"));

        env::remove_var("CODEX_HOME");
        let _ = fs::remove_dir_all(temp_root);
    }

    #[test]
    fn claude_code_session_scan_reads_ai_title_message() {
        let temp_root = env::temp_dir().join(format!(
            "docs-atlas-claude-title-test-{}",
            current_unix_nanos()
        ));
        fs::create_dir_all(&temp_root).unwrap();
        let session_path = temp_root.join("session.jsonl");
        let mut lines = Vec::new();
        for index in 0..9 {
            lines.push(format!("{{\"type\":\"message\",\"index\":{index}}}"));
        }
        lines.push(
            "{\"type\":\"ai-title\",\"sessionId\":\"9f37c3aa-6175-4429-96af-6d41df7d64dd\",\"aiTitle\":\"优化桌面端文档聚合应用界面设计\"}"
                .to_string(),
        );
        fs::write(&session_path, format!("{}\n", lines.join("\n"))).unwrap();

        let session =
            build_file_agent_session("claude-code", &temp_root, &session_path, "file", "medium");
        assert_eq!(session.title, "优化桌面端文档聚合应用界面设计");

        let _ = fs::remove_dir_all(temp_root);
    }

    #[test]
    fn codex_title_index_reads_session_index_jsonl() {
        let temp_root = env::temp_dir().join(format!(
            "docs-atlas-codex-title-index-test-{}",
            current_unix_nanos()
        ));
        fs::create_dir_all(&temp_root).unwrap();
        fs::write(
            temp_root.join("session_index.jsonl"),
            "{\"id\":\"019e85f1-b0d1-7c91-ba11-17535c256e6e\",\"title\":\"实现 PDF 阅读器目录树\"}\n",
        )
        .unwrap();

        let titles = build_codex_session_title_index(&temp_root);
        assert_eq!(
            titles.get("019e85f1-b0d1-7c91-ba11-17535c256e6e"),
            Some(&"实现 PDF 阅读器目录树".to_string())
        );

        let _ = fs::remove_dir_all(temp_root);
    }

    #[test]
    fn codex_title_index_reads_state_sqlite() {
        let temp_root = env::temp_dir().join(format!(
            "docs-atlas-codex-title-sqlite-test-{}",
            current_unix_nanos()
        ));
        fs::create_dir_all(&temp_root).unwrap();
        let db_path = temp_root.join("state_5.sqlite");
        let connection = Connection::open(&db_path).unwrap();
        connection
            .execute(
                "create table sessions (id text primary key, title text not null)",
                [],
            )
            .unwrap();
        connection
            .execute(
                "insert into sessions (id, title) values (?1, ?2)",
                params![
                    "019e85f1-b0d1-7c91-ba11-17535c256e6e",
                    "整理 Codex 会话索引"
                ],
            )
            .unwrap();
        drop(connection);

        let titles = build_codex_session_title_index(&temp_root);
        assert_eq!(
            titles.get("019e85f1-b0d1-7c91-ba11-17535c256e6e"),
            Some(&"整理 Codex 会话索引".to_string())
        );

        let _ = fs::remove_dir_all(temp_root);
    }

    #[test]
    fn opencode_backup_source_requires_existing_source() {
        let temp_root =
            env::temp_dir().join(format!("docs-atlas-opencode-test-{}", current_unix_nanos()));
        let missing_cli_path = temp_root.join("missing.sqlite");
        let default_dir = temp_root.join("opencode");
        fs::create_dir_all(&default_dir).unwrap();

        let source = resolve_existing_opencode_backup_source(
            Some(missing_cli_path.clone()),
            default_dir.clone(),
        );
        assert_eq!(source, Some(default_dir.clone()));

        fs::remove_dir_all(&default_dir).unwrap();
        let source = resolve_existing_opencode_backup_source(Some(missing_cli_path), default_dir);
        assert!(source.is_none());

        let _ = fs::remove_dir_all(temp_root);
    }
}
