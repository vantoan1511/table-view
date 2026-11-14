//! Pure-Rust workspace layout state, tab lifecycle, and status models.

use db_bridge::QueryResult;

use crate::db::ConnectionMetadata;
use crate::theme::{ConnectionColors, ShellDimensions, ThemeColor};

/// Minimum allowed width for the explorer sidebar in logical pixels.
pub const MIN_SIDEBAR_WIDTH_PX: f32 = 150.0;
/// Maximum allowed width for the explorer sidebar in logical pixels.
pub const MAX_SIDEBAR_WIDTH_PX: f32 = 500.0;
/// Default width for the explorer sidebar in logical pixels.
pub const DEFAULT_SIDEBAR_WIDTH_PX: f32 = ShellDimensions::standard().sidebar_width_px;

/// Named connection color tags matching [`ConnectionColors`].
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionColorTag {
    Indigo,
    Blue,
    Teal,
    Green,
    Amber,
    Orange,
    Pink,
    Gray,
    Purple,
    Rose,
    Emerald,
    Cyan,
    Violet,
    Red,
}

impl ConnectionColorTag {
    /// Resolves this tag to its concrete [`ThemeColor`] from [`ConnectionColors`].
    pub const fn resolve(self, colors: &ConnectionColors) -> ThemeColor {
        match self {
            Self::Indigo => colors.indigo,
            Self::Blue => colors.blue,
            Self::Teal => colors.teal,
            Self::Green => colors.green,
            Self::Amber => colors.amber,
            Self::Orange => colors.orange,
            Self::Pink => colors.pink,
            Self::Gray => colors.gray,
            Self::Purple => colors.purple,
            Self::Rose => colors.rose,
            Self::Emerald => colors.emerald,
            Self::Cyan => colors.cyan,
            Self::Violet => colors.violet,
            Self::Red => colors.red,
        }
    }
}

/// Kind of workspace tab.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    Table,
    Sql,
    Diagram,
    Index,
}

impl TabKind {
    /// Returns a compact uppercase badge label for the tab strip.
    pub const fn badge_label(self) -> &'static str {
        match self {
            Self::Table => "TBL",
            Self::Sql => "SQL",
            Self::Diagram => "ERD",
            Self::Index => "IDX",
        }
    }
}

/// Metadata for a single open tab in the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    pub id: u64,
    pub kind: TabKind,
    pub title: String,
    pub is_dirty: bool,
    pub connection_color: Option<ConnectionColorTag>,
    pub connection_name: Option<String>,
    pub schema: Option<String>,
    pub object_name: Option<String>,
}

/// Collapsible and resizable state for the left explorer sidebar.
#[derive(Debug, Clone, PartialEq)]
pub struct SidebarState {
    pub is_visible: bool,
    pub width_px: f32,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            is_visible: true,
            width_px: DEFAULT_SIDEBAR_WIDTH_PX,
        }
    }
}

impl SidebarState {
    /// Toggles sidebar visibility while preserving `width_px`.
    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
    }

    /// Sets sidebar width, ignoring non-finite values and clamping to `[150.0, 500.0]`.
    pub fn set_width(&mut self, width_px: f32) {
        if !width_px.is_finite() {
            return;
        }
        self.width_px = width_px.clamp(MIN_SIDEBAR_WIDTH_PX, MAX_SIDEBAR_WIDTH_PX);
    }

    /// Applies a drag delta relative to `start_width_px`, clamping to `[150.0, 500.0]`.
    pub fn apply_drag_delta(&mut self, start_width_px: f32, delta_x_px: f32) {
        self.set_width(start_width_px + delta_x_px);
    }
}

/// Summary of the active database connection shown in the status bar.
#[allow(dead_code)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting {
        connection_name: String,
    },
    Connected {
        connection_name: String,
        engine: String,
        host_port: String,
        schema: String,
        color: ConnectionColorTag,
    },
}

/// Summary of the latest query execution shown in the status bar.
#[allow(dead_code)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum QueryStatus {
    #[default]
    Idle,
    Running,
    Completed {
        elapsed_ms: u64,
        row_count: usize,
    },
    Error {
        message: String,
    },
}

/// Root pure-Rust state for the desktop workspace shell.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceState {
    pub sidebar: SidebarState,
    pub tabs: Vec<TabItem>,
    pub active_tab_id: Option<u64>,
    pub active_connection_id: Option<String>,
    pub connection_status: ConnectionStatus,
    pub query_status: QueryStatus,
    next_tab_id: u64,
    next_sql_number: u64,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceState {
    /// Creates an empty workspace with default sidebar state and no open tabs.
    pub fn new() -> Self {
        Self {
            sidebar: SidebarState::default(),
            tabs: Vec::new(),
            active_tab_id: None,
            active_connection_id: None,
            connection_status: ConnectionStatus::Disconnected,
            query_status: QueryStatus::Idle,
            next_tab_id: 1,
            next_sql_number: 1,
        }
    }

    /// Toggles sidebar visibility.
    pub fn toggle_sidebar(&mut self) {
        self.sidebar.toggle();
    }

    /// Updates sidebar width with clamping and non-finite protection.
    #[allow(dead_code)]
    pub fn set_sidebar_width(&mut self, width_px: f32) {
        self.sidebar.set_width(width_px);
    }

    /// Opens a new SQL query tab, activates it, and returns its allocated ID.
    pub fn open_new_sql_tab(&mut self) -> u64 {
        let id = self.next_tab_id;
        self.next_tab_id += 1;

        let sql_number = self.next_sql_number;
        self.next_sql_number += 1;

        let (title, connection_color, connection_name, schema) = match &self.connection_status {
            ConnectionStatus::Connected {
                connection_name,
                schema,
                color,
                ..
            } => (
                format!("{connection_name}-{sql_number}"),
                Some(*color),
                Some(connection_name.clone()),
                Some(schema.clone()),
            ),
            ConnectionStatus::Disconnected | ConnectionStatus::Connecting { .. } => {
                (format!("query-{sql_number}"), None, None, None)
            }
        };

        self.tabs.push(TabItem {
            id,
            kind: TabKind::Sql,
            title,
            is_dirty: false,
            connection_color,
            connection_name,
            schema,
            object_name: None,
        });
        self.active_tab_id = Some(id);
        id
    }

    /// Activates an existing tab by ID. Returns `true` if the tab exists.
    pub fn activate_tab(&mut self, id: u64) -> bool {
        if self.tabs.iter().any(|tab| tab.id == id) {
            self.active_tab_id = Some(id);
            true
        } else {
            false
        }
    }

    /// Closes a tab by ID, updating `active_tab_id` if the active tab was closed.
    pub fn close_tab(&mut self, id: u64) -> bool {
        let Some(idx) = self.tabs.iter().position(|tab| tab.id == id) else {
            return false;
        };

        self.tabs.remove(idx);
        if self.active_tab_id == Some(id) {
            self.active_tab_id = self
                .tabs
                .get(idx)
                .or_else(|| self.tabs.last())
                .map(|tab| tab.id);
        }
        true
    }

    /// Closes the currently active tab, returning `false` when no tab is active.
    pub fn close_active_tab(&mut self) -> bool {
        let Some(active_id) = self.active_tab_id else {
            return false;
        };
        self.close_tab(active_id)
    }

    /// Returns a reference to the currently active tab, if any.
    pub fn active_tab(&self) -> Option<&TabItem> {
        let active_id = self.active_tab_id?;
        self.tabs.iter().find(|tab| tab.id == active_id)
    }

    /// Updates the dirty flag of a tab by ID. Returns `true` if found.
    #[allow(dead_code)]
    pub fn set_tab_dirty(&mut self, id: u64, is_dirty: bool) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) else {
            return false;
        };
        tab.is_dirty = is_dirty;
        true
    }

    /// Updates the workspace connection status summary.
    #[allow(dead_code)]
    pub fn set_connection_status(&mut self, status: ConnectionStatus) {
        self.connection_status = status;
    }

    /// Updates the workspace query status summary.
    #[allow(dead_code)]
    pub fn set_query_status(&mut self, status: QueryStatus) {
        self.query_status = status;
    }

    /// Transitions the workspace into a connecting state for `connection_name`.
    #[cfg_attr(not(feature = "native-ui"), allow(dead_code))]
    pub fn begin_connecting(&mut self, connection_name: impl Into<String>) {
        self.connection_status = ConnectionStatus::Connecting {
            connection_name: connection_name.into(),
        };
    }

    /// Transitions the workspace into a connected state using resolved connection metadata.
    #[cfg_attr(not(feature = "native-ui"), allow(dead_code))]
    pub fn finish_connecting(&mut self, metadata: &ConnectionMetadata) {
        self.active_connection_id = Some(metadata.connection_id.clone());
        self.connection_status = ConnectionStatus::Connected {
            connection_name: metadata.connection_name.clone(),
            engine: metadata.engine.clone(),
            host_port: metadata.host_port.clone(),
            schema: metadata.schema.clone(),
            color: metadata.color,
        };
    }

    /// Resets connection state to disconnected and surfaces a connection error in query status.
    #[cfg_attr(not(feature = "native-ui"), allow(dead_code))]
    pub fn fail_connecting(&mut self, error_message: impl Into<String>) {
        self.active_connection_id = None;
        self.connection_status = ConnectionStatus::Disconnected;
        self.query_status = QueryStatus::Error {
            message: error_message.into(),
        };
    }

    /// Clears the active connection and resets connection status to disconnected.
    #[cfg_attr(not(feature = "native-ui"), allow(dead_code))]
    pub fn set_disconnected(&mut self) {
        self.active_connection_id = None;
        self.connection_status = ConnectionStatus::Disconnected;
    }

    /// Marks query status as running.
    #[cfg_attr(not(feature = "native-ui"), allow(dead_code))]
    pub fn begin_query(&mut self) {
        self.query_status = QueryStatus::Running;
    }

    /// Records completed query execution metrics from `result`.
    #[cfg_attr(not(feature = "native-ui"), allow(dead_code))]
    pub fn finish_query(&mut self, result: &QueryResult) {
        self.query_status = QueryStatus::Completed {
            elapsed_ms: result.execution_time,
            row_count: result.row_count,
        };
    }

    /// Records a query execution failure message.
    #[cfg_attr(not(feature = "native-ui"), allow(dead_code))]
    pub fn fail_query(&mut self, error_message: impl Into<String>) {
        self.query_status = QueryStatus::Error {
            message: error_message.into(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn test_sidebar_defaults_and_width_clamping() {
        let mut sidebar = SidebarState::default();
        assert!(sidebar.is_visible);
        assert_eq!(sidebar.width_px, 260.0);

        sidebar.set_width(80.0);
        assert_eq!(sidebar.width_px, 150.0);

        sidebar.set_width(900.0);
        assert_eq!(sidebar.width_px, 500.0);

        sidebar.set_width(320.0);
        assert_eq!(sidebar.width_px, 320.0);

        sidebar.set_width(f32::NAN);
        assert_eq!(sidebar.width_px, 320.0);

        sidebar.set_width(f32::INFINITY);
        assert_eq!(sidebar.width_px, 320.0);

        sidebar.toggle();
        assert!(!sidebar.is_visible);
        sidebar.toggle();
        assert!(sidebar.is_visible);
    }

    #[test]
    fn test_tab_lifecycle_open_activate_and_close_fallback() {
        let mut workspace = WorkspaceState::new();
        assert!(workspace.tabs.is_empty());
        assert_eq!(workspace.active_tab_id, None);
        assert!(!workspace.close_active_tab());

        let tab1 = workspace.open_new_sql_tab();
        assert_eq!(workspace.active_tab_id, Some(tab1));
        assert_eq!(workspace.tabs[0].title, "query-1");

        let tab2 = workspace.open_new_sql_tab();
        assert_eq!(workspace.active_tab_id, Some(tab2));
        assert_eq!(workspace.tabs[1].title, "query-2");

        let tab3 = workspace.open_new_sql_tab();
        assert_eq!(workspace.active_tab_id, Some(tab3));
        assert_eq!(workspace.tabs[2].title, "query-3");

        assert!(workspace.activate_tab(tab1));
        assert_eq!(workspace.active_tab_id, Some(tab1));

        // Closing non-active tab keeps tab1 active.
        assert!(workspace.close_tab(tab2));
        assert_eq!(workspace.active_tab_id, Some(tab1));

        // Closing active tab1 falls back cleanly to remaining tab3 at index 0.
        assert!(workspace.close_tab(tab1));
        assert_eq!(workspace.active_tab_id, Some(tab3));

        // Closing last tab clears active_tab_id.
        assert!(workspace.close_active_tab());
        assert_eq!(workspace.active_tab_id, None);
        assert!(workspace.tabs.is_empty());
    }

    #[test]
    fn test_open_sql_tab_uses_connected_name_and_color() {
        let mut workspace = WorkspaceState::new();
        workspace.set_connection_status(ConnectionStatus::Connected {
            connection_name: "prod-pg".to_string(),
            engine: "PostgreSQL".to_string(),
            host_port: "db.internal:5432".to_string(),
            schema: "public".to_string(),
            color: ConnectionColorTag::Emerald,
        });

        workspace.set_query_status(QueryStatus::Error {
            message: "syntax error".to_string(),
        });
        assert_eq!(
            workspace.query_status,
            QueryStatus::Error {
                message: "syntax error".to_string()
            }
        );

        assert_eq!(TabKind::Table.badge_label(), "TBL");
        assert_eq!(TabKind::Sql.badge_label(), "SQL");
        assert_eq!(TabKind::Diagram.badge_label(), "ERD");
        assert_eq!(TabKind::Index.badge_label(), "IDX");

        let tab_id = workspace.open_new_sql_tab();
        let tab = workspace.active_tab().expect("active tab should exist");
        assert_eq!(tab.id, tab_id);
        assert_eq!(tab.title, "prod-pg-1");
        assert_eq!(tab.connection_color, Some(ConnectionColorTag::Emerald));
        assert_eq!(tab.connection_name.as_deref(), Some("prod-pg"));
        assert_eq!(tab.schema.as_deref(), Some("public"));
        assert_eq!(
            ConnectionColorTag::Emerald.resolve(&Theme::dark().connection_colors),
            Theme::dark().connection_colors.emerald
        );
    }

    #[test]
    fn test_status_bar_state_transitions() {
        let mut workspace = WorkspaceState::new();
        assert_eq!(workspace.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(workspace.query_status, QueryStatus::Idle);

        workspace.set_connection_status(ConnectionStatus::Connecting {
            connection_name: "staging".to_string(),
        });
        assert!(matches!(
            workspace.connection_status,
            ConnectionStatus::Connecting { .. }
        ));

        workspace.set_query_status(QueryStatus::Running);
        assert_eq!(workspace.query_status, QueryStatus::Running);

        workspace.set_query_status(QueryStatus::Completed {
            elapsed_ms: 42,
            row_count: 128,
        });
        assert_eq!(
            workspace.query_status,
            QueryStatus::Completed {
                elapsed_ms: 42,
                row_count: 128
            }
        );

        workspace.set_query_status(QueryStatus::Error {
            message: "syntax error".to_string(),
        });
        assert_eq!(
            workspace.query_status,
            QueryStatus::Error {
                message: "syntax error".to_string()
            }
        );

        assert_eq!(TabKind::Table.badge_label(), "TBL");
        assert_eq!(TabKind::Sql.badge_label(), "SQL");
        assert_eq!(TabKind::Diagram.badge_label(), "ERD");
        assert_eq!(TabKind::Index.badge_label(), "IDX");

        let tab_id = workspace.open_new_sql_tab();
        assert!(workspace.set_tab_dirty(tab_id, true));
        assert!(workspace.active_tab().expect("tab exists").is_dirty);
        assert!(!workspace.set_tab_dirty(9999, true));
    }

    #[test]
    fn test_workspace_connection_and_query_lifecycle_transitions() {
        use crate::db::ConnectionMetadata;
        use db_bridge::QueryResult;

        let mut workspace = WorkspaceState::new();
        assert_eq!(workspace.active_connection_id, None);
        assert_eq!(workspace.connection_status, ConnectionStatus::Disconnected);

        workspace.begin_connecting("local-sqlite");
        assert_eq!(
            workspace.connection_status,
            ConnectionStatus::Connecting {
                connection_name: "local-sqlite".to_string(),
            }
        );

        let metadata = ConnectionMetadata {
            connection_id: "conn-1".to_string(),
            connection_name: "local-sqlite".to_string(),
            engine: "SQLite".to_string(),
            host_port: ":memory:".to_string(),
            schema: ":memory:".to_string(),
            color: ConnectionColorTag::Teal,
        };
        workspace.finish_connecting(&metadata);
        assert_eq!(workspace.active_connection_id.as_deref(), Some("conn-1"));
        assert_eq!(
            workspace.connection_status,
            ConnectionStatus::Connected {
                connection_name: "local-sqlite".to_string(),
                engine: "SQLite".to_string(),
                host_port: ":memory:".to_string(),
                schema: ":memory:".to_string(),
                color: ConnectionColorTag::Teal,
            }
        );

        workspace.begin_query();
        assert_eq!(workspace.query_status, QueryStatus::Running);

        let query_result = QueryResult {
            rows: Vec::new(),
            fields: Vec::new(),
            row_count: 3,
            execution_time: 12,
        };
        workspace.finish_query(&query_result);
        assert_eq!(
            workspace.query_status,
            QueryStatus::Completed {
                elapsed_ms: 12,
                row_count: 3,
            }
        );

        workspace.fail_query("syntax error");
        assert_eq!(
            workspace.query_status,
            QueryStatus::Error {
                message: "syntax error".to_string(),
            }
        );

        workspace.set_disconnected();
        assert_eq!(workspace.active_connection_id, None);
        assert_eq!(workspace.connection_status, ConnectionStatus::Disconnected);

        workspace.fail_connecting("timeout");
        assert_eq!(workspace.active_connection_id, None);
        assert_eq!(workspace.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(
            workspace.query_status,
            QueryStatus::Error {
                message: "timeout".to_string(),
            }
        );
    }
}
