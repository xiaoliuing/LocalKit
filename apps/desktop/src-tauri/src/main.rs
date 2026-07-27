#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, State, Window, WindowEvent};

const DESKTOP_SEED_VERSION_KEY: &str = "desktop_seed_version";
const CURRENT_DESKTOP_SEED_VERSION: &str = "3";
const BUNDLED_DEFAULT_DOCS_DIR_NAME: &str = "docs";
const DEFAULT_DOCS_SENTINELS: [&str; 2] = ["what-is-docs-atlas.md", "getting-started/README.md"];
const WORKSPACE_SOURCES_CHANGED_EVENT: &str = "workspace-sources-changed";
const WORKSPACE_SOURCE_WATCH_INTERVAL_MS: u64 = 1_500;
const APP_LOGS_DIR_NAME: &str = "logs";
const APP_LOG_FILE_NAME: &str = "docs-atlas.log";
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

  rows
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| error.to_string())?
    .into_iter()
    .map(|workspace| load_workspace_detail(&connection, workspace))
    .collect()
}

#[tauri::command]
fn upsert_workspace(app: AppHandle, input: WorkspaceSaveInput) -> Result<WorkspaceDetailPayload, String> {
  let mut connection = open_workspace_database(&app)?;
  let transaction = connection.transaction().map_err(|error| error.to_string())?;
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
        input.default_search_scope.clone().unwrap_or_else(|| "global".to_string()),
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
fn mark_workspace_opened(app: AppHandle, workspace_id: String) -> Result<Option<WorkspaceDetailPayload>, String> {
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
  let transaction = connection.transaction().map_err(|error| error.to_string())?;
  let workspace_count = transaction
    .query_row("select count(*) from workspaces", [], |row| row.get::<_, i64>(0))
    .map_err(|error| error.to_string())?;

  if workspace_count <= 1 {
    return Ok(false);
  }

  let deleted = transaction
    .execute("delete from workspaces where id = ?1", params![workspace_id])
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
          &format!("source_id={} path={} error={message}", source.id, source.path),
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
  let payload =
    serde_json::from_str::<WorkspaceTransferPayload>(&raw_value).map_err(|error| error.to_string())?;

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
      &format!("skip workspace_id={} reason=no_enabled_sources", workspace_id),
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
fn unwatch_workspace_sources(app: AppHandle, state: State<'_, WorkspaceSourceWatchState>) -> Result<(), String> {
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
    &format!("kind=app_data path={}", app_data_directory.to_string_lossy()),
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
      &Submenu::with_items(
        app,
        "View",
        true,
        &[
          &open_search,
          &open_settings,
        ],
      )?,
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
        &[&PredefinedMenuItem::about(app, Some("关于 Docs Atlas"), None)?],
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
        record_app_error(app, "menu.action", &format!("action=open-app-data error={error}"));
      }
    }
    MENU_ID_OPEN_LOGS_DIRECTORY => {
      if let Err(error) = open_logs_directory(app.clone()) {
        record_app_error(app, "menu.action", &format!("action=open-logs error={error}"));
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

  let Some(saved_state) = read_app_setting_json::<PersistedWindowState>(&connection, APP_WINDOW_STATE_KEY)
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
      saved_state.maximized, saved_state.width, saved_state.height, saved_state.x, saved_state.y
    ),
  );
}

fn snapshot_main_window_state(window: &Window) -> Result<PersistedWindowState, String> {
  let connection = open_workspace_database(window.app_handle())?;
  let mut next_state =
    read_app_setting_json::<PersistedWindowState>(&connection, APP_WINDOW_STATE_KEY)?.unwrap_or_default();

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
      record_app_error(app, "window.state", &format!("snapshot_error reason={reason} error={error}"));
      return;
    }
  };

  let connection = match open_workspace_database(app) {
    Ok(connection) => connection,
    Err(error) => {
      record_app_error(app, "window.state", &format!("persist_db_error reason={reason} error={error}"));
      return;
    }
  };

  if let Err(error) = write_app_setting_json(&connection, APP_WINDOW_STATE_KEY, &state) {
    record_app_error(app, "window.state", &format!("persist_error reason={reason} error={error}"));
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
        WindowEvent::CloseRequested { .. } => persist_main_window_state(window, "close-requested", true),
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
      delete_workspace,
      export_workspace_config,
      export_logs_file,
      get_default_docs_path,
      import_workspace_config,
      list_workspace_details,
      mark_workspace_opened,
      open_external_url,
      open_app_data_directory,
      open_logs_directory,
      pick_folder_path,
      pick_folder_paths,
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

  command.status().map_err(|error| error.to_string()).and_then(|status| {
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

  command.status().map_err(|error| error.to_string()).and_then(|status| {
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

fn maybe_migrate_legacy_seed_workspaces(app: &AppHandle, connection: &mut Connection) -> Result<(), String> {
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

fn should_reset_legacy_seed_workspaces(app: &AppHandle, connection: &Connection) -> Result<bool, String> {
  let workspaces = list_workspace_summaries(connection)?;
  if workspaces.is_empty() {
    return Ok(false);
  }

  if workspaces.len() == 1 && workspaces[0].id == "workspace:default" {
    let detail = load_workspace_detail(connection, workspaces[0].clone())?;
    return Ok(!matches_current_default_workspace(&detail, &resolve_default_docs_path(app)));
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
  let transaction = connection.transaction().map_err(|error| error.to_string())?;
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

  rows
    .collect::<Result<Vec<_>, _>>()
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
    || matches!(source.name.as_str(), "AI-Agent" | "Another Project" | "Local Workspace")
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
  matches!(value, "workspace:atlas" | "workspace:product" | "workspace:ai")
}

fn is_legacy_source_id(value: &str) -> bool {
  matches!(value, "source:atlas" | "source:product" | "source:ai")
}

fn load_workspace_summary(connection: &Connection, workspace_id: &str) -> Result<WorkspaceSummaryRow, String> {
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

      let children = flatten_source_nodes(workspace_id, Some(node.id), node.children.unwrap_or_default());
      std::iter::once(current).chain(children.into_iter()).collect::<Vec<_>>()
    })
    .collect()
}

fn build_source_tree(rows: Vec<WorkspaceSourceNodeRow>) -> Vec<WorkspaceSourceNodePayload> {
  let by_parent = rows.iter().fold(
    std::collections::HashMap::<Option<String>, Vec<WorkspaceSourceNodeRow>>::new(),
    |mut groups, row| {
      groups.entry(row.parent_id.clone()).or_default().push(row.clone());
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
  let transaction = connection.transaction().map_err(|error| error.to_string())?;
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

fn export_workspace_sources(nodes: Vec<WorkspaceSourceNodePayload>) -> Vec<WorkspaceSourceNodeInput> {
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
      let children = rekey_imported_source_nodes(node.children.unwrap_or_default(), Some(next_id.clone()));

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
) -> Result<(Vec<WorkspaceSourceDocumentPayload>, WorkspaceSourceStatusPayload), String> {
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
      let markdown = std::fs::read_to_string(&snapshot.absolute_path).map_err(|error| error.to_string())?;
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

fn collect_markdown_file_snapshots(root: &Path, base_root: &Path) -> Result<Vec<MarkdownFileSnapshot>, String> {
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

fn collect_video_tree_nodes(root: &Path, base_root: &Path) -> Result<Vec<VideoTreeNodePayload>, String> {
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
  path
    .strip_prefix(base_root)
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
  root
    .canonicalize()
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
