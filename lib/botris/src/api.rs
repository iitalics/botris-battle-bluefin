//! Botris API definitions.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::game::{Board, Command, Piece, PieceData, Queue};

pub type SessionId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Message {
    #[serde(rename_all = "camelCase")]
    RoomData {
        room_data: RoomData,
    },
    #[serde(rename_all = "camelCase")]
    Authenticated {
        session_id: SessionId,
    },
    #[serde(rename_all = "camelCase")]
    PlayerJoined {
        player_data: PlayerData,
    },
    #[serde(rename_all = "camelCase")]
    PlayerLeft {
        session_id: SessionId,
    },
    #[serde(rename_all = "camelCase")]
    SettingsChanged {
        room_data: RoomData,
    },
    GameStarted,
    #[serde(rename_all = "camelCase")]
    RoundStarted {
        starts_at: u64,
        room_data: RoomData,
    },
    #[serde(rename_all = "camelCase")]
    PlayerAction {
        session_id: SessionId,
        game_state: GameState,
        // commands: Command[];
        // events: GameEvent[];
    },
    #[serde(rename_all = "camelCase")]
    PlayerDamageReceived {
        session_id: SessionId,
        game_state: GameState,
        damage: u32,
    },
    #[serde(rename_all = "camelCase")]
    RequestMove {
        game_state: GameState,
        players: Vec<PlayerData>,
    },
    #[serde(rename_all = "camelCase")]
    RoundOver {
        winner_id: SessionId,
        winner_info: PlayerInfo,
        room_data: RoomData,
    },
    #[serde(rename_all = "camelCase")]
    GameOver {
        winner_id: SessionId,
        winner_info: PlayerInfo,
        room_data: RoomData,
    },
    #[serde(rename_all = "camelCase")]
    GameReset {
        room_data: RoomData,
    },
    #[serde(rename_all = "camelCase")]
    PlayerBanned {/* payload unimplemented */},
    #[serde(rename_all = "camelCase")]
    PlayerUnbanned {/* payload unimplemented */},
    #[serde(rename_all = "camelCase")]
    HostChanged {/* payload unimplemented */},
    Error(String),
    #[serde(untagged)]
    Other(UnknownMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownMessage {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
#[error("message parse error: {0}")]
pub struct MessageFromStrError(serde_json::Error);

impl FromStr for Message {
    type Err = MessageFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(MessageFromStrError)
    }
}

impl FromStr for UnknownMessage {
    type Err = MessageFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(MessageFromStrError)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomData {
    pub id: String,
    pub host: PlayerInfo,
    // private: boolean;
    pub initial_pps: f32,
    pub final_pps: f32,
    pub start_margin: f32,
    pub end_margin: f32,
    pub ft: u32,
    pub max_players: u32,
    pub game_ongoing: bool,
    pub round_ongoing: bool,
    // startedAt: number | null;
    // endedAt: number | null;
    // lastWinner: SessionId | null;
    pub players: Vec<PlayerData>,
    pub banned: Vec<PlayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfo {
    pub user_id: String,
    pub creator: String,
    pub bot: String,
    // avatar: null;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerData {
    pub session_id: SessionId,
    pub playing: bool,
    pub info: PlayerInfo,
    pub wins: u32,
    pub game_state: Option<GameState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub board: Board,
    pub queue: Queue,
    pub held: Option<Piece>,
    pub can_hold: bool,
    pub current: PieceData,
    pub combo: u32,
    pub b2b: bool,
    pub score: u32,
    pub pieces_placed: u32,
    pub garbage_queued: u32,
    pub dead: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ClientMessage<'a> {
    #[serde(rename_all = "camelCase")]
    Action { commands: &'a [Command] },
}

impl std::fmt::Display for ClientMessage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)
            .and_then(|s| f.write_str(&s))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub board_width: u32,
    pub board_height: u32,
    pub garbage_messiness: f32,
    pub attack_table: AttackTable,
    pub combo_table: ComboTable,
}

impl Default for GameInfo {
    fn default() -> Self {
        Self {
            board_width: 10,
            board_height: 20,
            garbage_messiness: 0.05,
            attack_table: AttackTable::default(),
            combo_table: ComboTable::default(),
        }
    }
}

impl std::fmt::Display for GameInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self)
            .map_err(|_| std::fmt::Error)
            .and_then(|s| f.write_str(&s))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttackTable {
    pub single: u32,
    pub double: u32,
    pub triple: u32,
    pub quad: u32,
    pub asd: u32,
    pub ass: u32,
    pub ast: u32,
    pub pc: u32,
    pub b2b: u32,
}

impl Default for AttackTable {
    fn default() -> Self {
        Self {
            single: 0,
            double: 1,
            triple: 2,
            quad: 4,
            asd: 4,
            ass: 2,
            ast: 6,
            pc: 10,
            b2b: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComboTable([u32; 10]);

impl std::ops::Index<usize> for ComboTable {
    type Output = u32;
    fn index(&self, index: usize) -> &u32 {
        if index >= self.0.len() {
            self.0.last().unwrap()
        } else {
            &self.0[index]
        }
    }
}

impl Default for ComboTable {
    fn default() -> Self {
        Self([0, 0, 1, 1, 1, 2, 2, 3, 3, 4])
    }
}

impl AsRef<[u32]> for ComboTable {
    fn as_ref(&self) -> &[u32] {
        &self.0
    }
}
