//! Creation Cloud 各业务模块共享的值对象、分页与错误语义。
//! 本模块不依赖数据库，不承载任何具体业务流程。

mod admin_actor;
mod admin_login_name;
mod auth;
mod error;
mod http_range;
mod page;
mod release_contract;
mod request_context;
mod semantic_version;

pub use admin_actor::AdminActor;
pub use admin_login_name::{
    ADMIN_LOGIN_NAME_MAX_LEN, ADMIN_LOGIN_NAME_MIN_LEN, normalize_admin_login_name,
};
pub use auth::AuthenticatedSession;
pub use error::{AppError, AppResult, SyncResyncReason};
pub use http_range::{ByteRange, InvalidByteRange};
pub use page::{Page, PageQuery};
pub use release_contract::{
    FormalReleaseAssetIdentity, MANUAL_RELEASE_ASSETS, formal_release_asset_identities,
    release_asset_set_is_complete, is_desktop_updater_asset,
};
pub use request_context::{
    current_request_id, mark_semantic_audit_recorded, with_request_id, with_semantic_audit_tracking,
};
pub use semantic_version::{
    MAX_SEMANTIC_VERSION_LENGTH, SemanticVersion, normalize_semantic_version,
};
