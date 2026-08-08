pub mod bilibili;
pub mod catalog;
pub mod model;
pub mod resolver;
pub mod runtime;
pub mod search;
pub mod service;
pub(crate) mod spool;
pub(crate) mod trace;
pub mod wechat;
pub mod youtube;
// ==== sync-generated:begin playback_mod_decl ====
pub mod audius;
// ==== sync-generated:end playback_mod_decl ====

pub use model::{PlayableEntry, SourceKind, SourceRef, TrackId};
pub use runtime::BackendRuntime;
pub use service::PlaybackService;
