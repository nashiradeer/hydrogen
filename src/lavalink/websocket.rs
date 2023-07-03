use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkReadyEvent {
    pub resumed: bool,
    pub session_id: String
}