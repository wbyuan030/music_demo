use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use anyhow::{anyhow, Result};
use native_db::{Builder, Database};
use tokio::sync::Mutex;

use crate::{storage::TRACK_MODEL, types::Track};

pub static TRACK_STATE: OnceLock<Arc<Mutex<HashMap<String, Track>>>> = OnceLock::new();

pub fn init_track_state() -> Result<()> {
    TRACK_STATE
        .set(Arc::new(Mutex::new(HashMap::new())))
        .map_err(|_| anyhow::anyhow!("track state init error"))
}

pub fn get_track_state() -> Result<Arc<Mutex<HashMap<String, Track>>>> {
    let track_map = match TRACK_STATE.get() {
        Some(d) => d.clone(),
        None => return Err(anyhow!("TRACK MAP IS NOT INIT")),
    };

    Ok(track_map)
}

pub static DB_INSTANCE: OnceLock<Database<'static>> = OnceLock::new();

pub fn init_db() -> Result<()> {
    let db = Builder::new().create(&TRACK_MODEL, "./local.db").unwrap();
    if DB_INSTANCE.set(db).is_err() {
        return Err(anyhow!("FAILED TO INIT DB"));
    };
    Ok(())
}

pub fn get_db() -> &'static Database<'static> {
    DB_INSTANCE.get().expect("Database Not Initialize")
}
