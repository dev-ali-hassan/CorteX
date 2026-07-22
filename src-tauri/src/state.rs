use std::{
    error::Error,
    sync::{atomic::AtomicBool, Mutex},
    time::Duration,
};

use tauri::AppHandle;

use crate::{db::Database, models::PopupPayload};

pub struct AppState {
    pub db: Database,
    pub client: reqwest::Client,
    pub last_popup: Mutex<Option<PopupPayload>>,
    pub last_selection_window: Mutex<Option<isize>>,
    pub shortcuts_paused: AtomicBool,
}

impl AppState {
    pub fn new(app: &AppHandle) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            db: Database::new(app)?,
            client: reqwest::Client::builder()
                .user_agent("CorteX/1.0")
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(45))
                .build()?,
            last_popup: Mutex::new(None),
            last_selection_window: Mutex::new(None),
            shortcuts_paused: AtomicBool::new(false),
        })
    }
}
