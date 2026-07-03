use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub const EVENT_COMPRESS_PROGRESS: &str = "compress-progress";

#[derive(Clone, Serialize)]
pub struct ProgressPayload {
    pub job_id: String,
    pub percent: f32,
    pub message: String,
}

pub struct ProgressEmitter {
    app: AppHandle,
    job_id: String,
}

impl ProgressEmitter {
    pub fn new(app: AppHandle, job_id: String) -> Self {
        Self { app, job_id }
    }

    pub fn emit(&self, percent: f32, message: impl Into<String>) {
        let _ = self.app.emit(
            EVENT_COMPRESS_PROGRESS,
            ProgressPayload {
                job_id: self.job_id.clone(),
                percent: percent.clamp(0.0, 100.0),
                message: message.into(),
            },
        );
    }
}
