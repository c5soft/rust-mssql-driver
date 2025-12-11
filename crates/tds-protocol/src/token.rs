//! TDS token stream definitions.
//!
//! Tokens are the fundamental units of TDS response data. The server sends
//! a stream of tokens that describe metadata, rows, errors, and other information.

/// Token type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TokenType {
    /// Column metadata (COLMETADATA).
    ColMetaData = 0x81,
    /// Error message (ERROR).
    Error = 0xAA,
    /// Informational message (INFO).
    Info = 0xAB,
    /// Login acknowledgment (LOGINACK).
    LoginAck = 0xAD,
    /// Row data (ROW).
    Row = 0xD1,
    /// Null bitmap compressed row (NBCROW).
    NbcRow = 0xD2,
    /// Environment change (ENVCHANGE).
    EnvChange = 0xE3,
    /// SSPI authentication (SSPI).
    Sspi = 0xED,
    /// Done (DONE).
    Done = 0xFD,
    /// Done in procedure (DONEINPROC).
    DoneInProc = 0xFF,
    /// Done procedure (DONEPROC).
    DoneProc = 0xFE,
    /// Return status (RETURNSTATUS).
    ReturnStatus = 0x79,
    /// Return value (RETURNVALUE).
    ReturnValue = 0xAC,
    /// Order (ORDER).
    Order = 0xA9,
    /// Feature extension acknowledgment (FEATUREEXTACK).
    FeatureExtAck = 0xAE,
    /// Session state (SESSIONSTATE).
    SessionState = 0xE4,
    /// Federated authentication info (FEDAUTHINFO).
    FedAuthInfo = 0xEE,
    /// Column info (COLINFO).
    ColInfo = 0xA5,
    /// Table name (TABNAME).
    TabName = 0xA4,
    /// Offset (OFFSET).
    Offset = 0x78,
}

impl TokenType {
    /// Create a token type from a raw byte.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x81 => Some(Self::ColMetaData),
            0xAA => Some(Self::Error),
            0xAB => Some(Self::Info),
            0xAD => Some(Self::LoginAck),
            0xD1 => Some(Self::Row),
            0xD2 => Some(Self::NbcRow),
            0xE3 => Some(Self::EnvChange),
            0xED => Some(Self::Sspi),
            0xFD => Some(Self::Done),
            0xFF => Some(Self::DoneInProc),
            0xFE => Some(Self::DoneProc),
            0x79 => Some(Self::ReturnStatus),
            0xAC => Some(Self::ReturnValue),
            0xA9 => Some(Self::Order),
            0xAE => Some(Self::FeatureExtAck),
            0xE4 => Some(Self::SessionState),
            0xEE => Some(Self::FedAuthInfo),
            0xA5 => Some(Self::ColInfo),
            0xA4 => Some(Self::TabName),
            0x78 => Some(Self::Offset),
            _ => None,
        }
    }
}

/// Parsed TDS token.
///
/// This enum represents all possible tokens that can be received from SQL Server.
/// Each variant contains the parsed token data.
#[derive(Debug, Clone)]
pub enum Token {
    /// Column metadata describing result set structure.
    ColMetaData(ColMetaData),
    /// Row data.
    Row(RawRow),
    /// Null bitmap compressed row.
    NbcRow(NbcRow),
    /// Completion of a SQL statement.
    Done(Done),
    /// Completion of a stored procedure.
    DoneProc(DoneProc),
    /// Completion within a stored procedure.
    DoneInProc(DoneInProc),
    /// Return status from stored procedure.
    ReturnStatus(i32),
    /// Return value from stored procedure.
    ReturnValue(ReturnValue),
    /// Error message from server.
    Error(ServerError),
    /// Informational message from server.
    Info(ServerInfo),
    /// Login acknowledgment.
    LoginAck(LoginAck),
    /// Environment change notification.
    EnvChange(EnvChange),
    /// Column ordering information.
    Order(Order),
    /// Feature extension acknowledgment.
    FeatureExtAck(FeatureExtAck),
    /// SSPI authentication data.
    Sspi(SspiToken),
    /// Session state information.
    SessionState(SessionState),
    /// Federated authentication info.
    FedAuthInfo(FedAuthInfo),
}

/// Column metadata token.
#[derive(Debug, Clone, Default)]
pub struct ColMetaData {
    /// Column definitions.
    pub columns: Vec<ColumnData>,
}

/// Column definition within metadata.
#[derive(Debug, Clone)]
pub struct ColumnData {
    /// Column name.
    pub name: String,
    /// Column data type.
    pub col_type: u8,
    /// Column flags.
    pub flags: u16,
    /// Type-specific metadata.
    pub type_info: TypeInfo,
}

/// Type-specific metadata.
#[derive(Debug, Clone, Default)]
pub struct TypeInfo {
    /// Maximum length for variable-length types.
    pub max_length: Option<u32>,
    /// Precision for numeric types.
    pub precision: Option<u8>,
    /// Scale for numeric types.
    pub scale: Option<u8>,
    /// Collation for string types.
    pub collation: Option<Collation>,
}

/// SQL Server collation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Collation {
    /// Locale ID.
    pub lcid: u32,
    /// Sort ID.
    pub sort_id: u8,
}

/// Raw row data (not yet decoded).
#[derive(Debug, Clone)]
pub struct RawRow {
    /// Raw column values.
    pub data: bytes::Bytes,
}

/// Null bitmap compressed row.
#[derive(Debug, Clone)]
pub struct NbcRow {
    /// Null bitmap.
    pub null_bitmap: Vec<u8>,
    /// Raw non-null column values.
    pub data: bytes::Bytes,
}

/// Done token indicating statement completion.
#[derive(Debug, Clone, Copy)]
pub struct Done {
    /// Status flags.
    pub status: DoneStatus,
    /// Current command.
    pub cur_cmd: u16,
    /// Row count (if applicable).
    pub row_count: u64,
}

/// Done status flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct DoneStatus {
    /// More results follow.
    pub more: bool,
    /// Error occurred.
    pub error: bool,
    /// Transaction in progress.
    pub in_xact: bool,
    /// Row count is valid.
    pub count: bool,
    /// Attention acknowledgment.
    pub attn: bool,
    /// Server error caused statement termination.
    pub srverror: bool,
}

/// Done in procedure token.
#[derive(Debug, Clone, Copy)]
pub struct DoneInProc {
    /// Status flags.
    pub status: DoneStatus,
    /// Current command.
    pub cur_cmd: u16,
    /// Row count.
    pub row_count: u64,
}

/// Done procedure token.
#[derive(Debug, Clone, Copy)]
pub struct DoneProc {
    /// Status flags.
    pub status: DoneStatus,
    /// Current command.
    pub cur_cmd: u16,
    /// Row count.
    pub row_count: u64,
}

/// Return value from stored procedure.
#[derive(Debug, Clone)]
pub struct ReturnValue {
    /// Parameter ordinal.
    pub param_ordinal: u16,
    /// Parameter name.
    pub param_name: String,
    /// Status flags.
    pub status: u8,
    /// User type.
    pub user_type: u32,
    /// Type flags.
    pub flags: u16,
    /// Type info.
    pub type_info: TypeInfo,
    /// Value data.
    pub value: bytes::Bytes,
}

/// Server error message.
#[derive(Debug, Clone)]
pub struct ServerError {
    /// Error number.
    pub number: i32,
    /// Error state.
    pub state: u8,
    /// Error severity class.
    pub class: u8,
    /// Error message text.
    pub message: String,
    /// Server name.
    pub server: String,
    /// Procedure name.
    pub procedure: String,
    /// Line number.
    pub line: i32,
}

/// Server informational message.
#[derive(Debug, Clone)]
pub struct ServerInfo {
    /// Info number.
    pub number: i32,
    /// Info state.
    pub state: u8,
    /// Info class (severity).
    pub class: u8,
    /// Info message text.
    pub message: String,
    /// Server name.
    pub server: String,
    /// Procedure name.
    pub procedure: String,
    /// Line number.
    pub line: i32,
}

/// Login acknowledgment token.
#[derive(Debug, Clone)]
pub struct LoginAck {
    /// Interface type.
    pub interface: u8,
    /// TDS version.
    pub tds_version: u32,
    /// Program name.
    pub prog_name: String,
    /// Program version.
    pub prog_version: u32,
}

/// Environment change token.
#[derive(Debug, Clone)]
pub struct EnvChange {
    /// Type of environment change.
    pub env_type: EnvChangeType,
    /// New value.
    pub new_value: EnvChangeValue,
    /// Old value.
    pub old_value: EnvChangeValue,
}

/// Environment change type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EnvChangeType {
    /// Database changed.
    Database = 1,
    /// Language changed.
    Language = 2,
    /// Character set changed.
    CharacterSet = 3,
    /// Packet size changed.
    PacketSize = 4,
    /// Unicode data sorting locale ID.
    UnicodeSortingLocalId = 5,
    /// Unicode comparison flags.
    UnicodeComparisonFlags = 6,
    /// SQL collation.
    SqlCollation = 7,
    /// Begin transaction.
    BeginTransaction = 8,
    /// Commit transaction.
    CommitTransaction = 9,
    /// Rollback transaction.
    RollbackTransaction = 10,
    /// Enlist DTC transaction.
    EnlistDtcTransaction = 11,
    /// Defect DTC transaction.
    DefectTransaction = 12,
    /// Real-time log shipping.
    RealTimeLogShipping = 13,
    /// Promote transaction.
    PromoteTransaction = 15,
    /// Transaction manager address.
    TransactionManagerAddress = 16,
    /// Transaction ended.
    TransactionEnded = 17,
    /// Reset connection completion acknowledgment.
    ResetConnectionCompletionAck = 18,
    /// User instance started.
    UserInstanceStarted = 19,
    /// Routing information.
    Routing = 20,
}

/// Environment change value.
#[derive(Debug, Clone)]
pub enum EnvChangeValue {
    /// String value.
    String(String),
    /// Binary value.
    Binary(bytes::Bytes),
    /// Routing information.
    Routing {
        /// Host name.
        host: String,
        /// Port number.
        port: u16,
    },
}

/// Column ordering information.
#[derive(Debug, Clone)]
pub struct Order {
    /// Ordered column indices.
    pub columns: Vec<u16>,
}

/// Feature extension acknowledgment.
#[derive(Debug, Clone)]
pub struct FeatureExtAck {
    /// Acknowledged features.
    pub features: Vec<FeatureAck>,
}

/// Individual feature acknowledgment.
#[derive(Debug, Clone)]
pub struct FeatureAck {
    /// Feature ID.
    pub feature_id: u8,
    /// Feature data.
    pub data: bytes::Bytes,
}

/// SSPI authentication token.
#[derive(Debug, Clone)]
pub struct SspiToken {
    /// SSPI data.
    pub data: bytes::Bytes,
}

/// Session state token.
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Session state data.
    pub data: bytes::Bytes,
}

/// Federated authentication info.
#[derive(Debug, Clone)]
pub struct FedAuthInfo {
    /// STS URL.
    pub sts_url: String,
    /// Service principal name.
    pub spn: String,
}

// Implement Vec for no_std compatibility
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
