use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackActionPayload {
    pub index: usize,
    pub total: usize,
    pub action_type: String,
    pub x: f64,
    pub y: f64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineEvent {
    RecordingStateChanged(bool),
    PlaybackStateChanged(bool),
    PlaybackAction(PlaybackActionPayload),
}
