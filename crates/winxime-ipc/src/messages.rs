use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    pub str: String,
}

impl Default for Text {
    fn default() -> Self {
        Self { str: String::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateInfo {
    pub current_page: u32,
    pub total_pages: u32,
    pub highlighted: usize,
    pub is_last_page: bool,
    pub candies: Vec<Text>,
    pub comments: Vec<Text>,
    pub labels: Vec<Text>,
}

impl Default for CandidateInfo {
    fn default() -> Self {
        Self {
            current_page: 0,
            total_pages: 0,
            highlighted: 0,
            is_last_page: false,
            candies: Vec::new(),
            comments: Vec::new(),
            labels: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub preedit: Text,
    pub commit: Option<String>,
    pub candidates: CandidateInfo,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            preedit: Text::default(),
            commit: None,
            candidates: CandidateInfo::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub schema_name: String,
    pub schema_id: String,
    pub ascii_mode: bool,
    pub composing: bool,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            schema_name: String::new(),
            schema_id: String::new(),
            ascii_mode: false,
            composing: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcCommand {
    Echo,
    StartSession,
    EndSession,
    ProcessKeyEvent,
    UpdateInputPosition,
    FocusIn,
    FocusOut,
    SelectCandidate,
    ChangePage,
    CommitComposition,
    ClearComposition,
    ShutdownServer,
    ToggleAsciiMode,
    ShowTrayIcon,
    HideTrayIcon,
    HideCandidates,
    ReloadConfig,
    GetSchemaList,
    SelectSchema,
    ShowRoot,
    HideRoot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub command: IpcCommand,
    pub session_id: u32,
    pub data: IpcRequestData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IpcRequestData {
    None,
    KeyEvent(KeyEventData),
    Position(Position),
    SelectIndex(usize),
    ChangePage(bool),
    SelectSchema(String),
    ShowRoot(char),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub schema_id: String,
    pub schema_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEventData {
    pub keycode: i32,
    pub modifiers: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub success: bool,
    pub session_id: u32,
    pub context: Option<Context>,
    pub status: Option<Status>,
    pub schema_list: Option<Vec<SchemaInfo>>,
}

pub const IPC_PIPE_NAME: &str = "WinximeNamedPipe";

pub fn get_pipe_path() -> String {
    format!("\\\\.\\pipe\\{}", IPC_PIPE_NAME)
}
