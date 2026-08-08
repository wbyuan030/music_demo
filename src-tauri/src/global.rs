use std::sync::{Arc, OnceLock};

use anyhow::{anyhow, Result};
use native_db::{Builder, Database};

use crate::{playback::catalog::TrackCatalog, storage::TRACK_MODEL};

pub static TRACK_STATE: OnceLock<Arc<TrackCatalog>> = OnceLock::new();

pub fn init_track_state() -> Result<()> {
    TRACK_STATE
        .set(Arc::new(TrackCatalog::new()))
        .map_err(|_| anyhow!("track state init error"))
}

pub fn get_track_state() -> Result<Arc<TrackCatalog>> {
    TRACK_STATE
        .get()
        .cloned()
        .ok_or_else(|| anyhow!("TRACK MAP IS NOT INIT"))
}

pub static DB_INSTANCE: OnceLock<Database<'static>> = OnceLock::new();

pub fn init_db() -> Result<()> {
    init_db_at("./local.db")
}

/// 以指定路径初始化全局 DB。测试用它避开应用正在使用的 `local.db`（redb 文件锁）。
pub fn init_db_at(path: &str) -> Result<()> {
    let db = Builder::new().create(&TRACK_MODEL, path).unwrap();
    if DB_INSTANCE.set(db).is_err() {
        return Err(anyhow!("FAILED TO INIT DB"));
    };
    Ok(())
}

pub fn get_db() -> &'static Database<'static> {
    DB_INSTANCE.get().expect("Database Not Initialize")
}
