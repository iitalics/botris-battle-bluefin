//! Botris API example.

#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use clap::Parser;
use futures::{SinkExt as _, StreamExt as _};
use tokio_tungstenite::connect_async as ws_connect_async;
use tokio_tungstenite::tungstenite;
use tungstenite::http::Uri;

use botris::api::{ClientMessage, Message, RoomData, SessionId, UnknownMessage};
use botris::game::{Board, Command, PieceData};

/// Botris API example.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    token: String,
    room_key: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .parse_filters("botris_hello=debug")
        .format_timestamp_millis()
        .init();

    info!("hello");

    let mut ws = {
        let Args { room_key, token } = Args::parse();

        let uri = format!("wss://botrisbattle.com/ws?token={token}&roomKey={room_key}");
        let uri: Uri = uri.parse().context("invalid URI")?;
        debug!("uri={uri}");

        let (ws, _res) = ws_connect_async(&uri).await.context("connect error")?;
        ws
    };

    let mut preauth_room_data = None;
    let mut preauth_session_id = None;
    let mut session = None;

    while let Some(ws_msg) = ws.next().await {
        let ws_msg = ws_msg.context("read error")?;

        if !ws_msg.is_text() {
            debug!("{ws_msg:?}");
            if ws_msg.is_close() {
                if let Ok(reason) = ws_msg.to_text() {
                    error!("closed: {reason:?}");
                } else {
                    error!("closed");
                }
                break;
            }
            continue;
        }

        let ws_msg = ws_msg.to_text().unwrap();
        let msg = ws_msg
            .parse::<Message>()
            .with_context(|| format!("{ws_msg:?}"))?;

        if session.is_none() {
            match &msg {
                Message::RoomData { room_data } => {
                    preauth_room_data = Some(room_data.clone());
                }
                Message::Authenticated { session_id } => {
                    preauth_session_id = Some(session_id.clone());
                }
                _ => {
                    trace!("{msg:?}");
                    if log_enabled!(log::Level::Warn) {
                        if let Ok(msg) = ws_msg.parse::<UnknownMessage>() {
                            warn!("got {:?} message but wasn't authenticated", msg.type_);
                        }
                    }
                }
            }

            if preauth_room_data.is_some() && preauth_session_id.is_some() {
                let room_data = preauth_room_data.take().unwrap();
                let session_id = preauth_session_id.take().unwrap();
                let new_session = Session::new(session_id, room_data);
                info!("new session: {}", new_session.session_id);
                session = Some(new_session);
            }

            continue;
        }

        let session = session.as_mut().unwrap();
        trace!("> {msg:?}");

        match msg {
            Message::GameStarted => {
                info!("game started");
            }

            Message::RoomData { .. } | Message::Authenticated { .. } => {
                warn!("didn't expect 'room_data' or 'authenticated'");
            }

            Message::SettingsChanged { room_data } => {
                info!("settings changed");
                session.room_data = room_data;
            }

            Message::GameReset { room_data } => {
                info!("game reset");
                debug!("{room_data:?}");
                session.room_data = room_data;
            }

            Message::RoundStarted {
                starts_at,
                room_data,
            } => {
                use std::time::{Duration, SystemTime};

                let t0 = SystemTime::now();
                let t1 = SystemTime::UNIX_EPOCH + Duration::from_millis(starts_at);

                let dt = t1.duration_since(t0).map_or(0.0, |d| d.as_secs_f32());
                info!("round starting in {dt:.3}");
                debug!("{room_data:?}");
                session.room_data = room_data;
            }

            Message::RoundOver { winner_id } => {
                if winner_id == session.session_id {
                    info!("round over: i won");
                } else {
                    info!("round over: i lost");
                }
            }

            Message::GameOver { winner_id } => {
                if winner_id == session.session_id {
                    info!("game over: i won");
                } else {
                    info!("game over: i lost");
                }
            }

            Message::RequestMove { game_state } => {
                info!("move requested");
                info!(
                    "> {:?} {:?} {:?}",
                    game_state.current.piece, game_state.held, game_state.queue
                );

                if game_state.current != PieceData::spawn(game_state.current.piece) {
                    warn!("not spawn: {:?}", game_state.current);
                    warn!(" != {:?}", PieceData::spawn(game_state.current.piece));
                }

                let commands = &[
                    Command::MoveLeft,
                    Command::RotateCw,
                    Command::RotateCcw,
                    Command::RotateCcw,
                    Command::MoveRight,
                    Command::RotateCw,
                ];
                let cmsg = ClientMessage::Action { commands };
                let ws_cmsg = tungstenite::Message::text(cmsg.to_string());
                info!("< {cmsg:?}");
                ws.send(ws_cmsg).await.context("send error")?;

                let mut expected_board = game_state.board.clone();
                let mut piece = game_state.current;
                for &cmd in commands {
                    piece = cmd.apply(piece, &game_state.board).unwrap_or(piece);
                }
                piece = piece.sonic_drop(&game_state.board);
                expected_board.place_piece(piece);

                session.expected_board = Some(expected_board);
            }

            Message::PlayerAction {
                session_id,
                commands,
                game_state,
            } => {
                debug!("{session_id}: {commands:?}, {game_state:?}");

                if session_id == session.session_id {
                    if let Some(expected_board) = session.expected_board.take() {
                        if expected_board != game_state.board {
                            warn!("board differed from expected");
                            warn!("   {:?}", game_state.board);
                            warn!("!= {:?}", expected_board);
                        }
                    }
                }

                for pl in session.room_data.players.iter_mut() {
                    if pl.session_id == session_id {
                        pl.game_state = Some(game_state);
                        break;
                    }
                }
            }

            Message::PlayerDamageReceived {
                session_id,
                damage,
                game_state,
            } => {
                debug!("{session_id}: damage {damage:?}, {game_state:?}");
            }

            Message::HostChanged {}
            | Message::PlayerBanned {}
            | Message::PlayerJoined {}
            | Message::PlayerLeft {}
            | Message::PlayerUnbanned {} => {
                if log_enabled!(log::Level::Debug) {
                    debug!("ignoring: {msg:?}");
                }
            }

            Message::Error(reason) => {
                error!("error: {reason}");
            }

            Message::Unknown => {
                let msg = ws_msg.parse::<UnknownMessage>().unwrap();
                warn!("unknown message: {:?}", msg.type_);
                debug!("{ws_msg}");
            }
        }
    }

    info!("bye");

    Ok(())
}

#[derive(Debug)]
struct Session {
    session_id: SessionId,
    room_data: RoomData,
    expected_board: Option<Board>,
}

impl Session {
    fn new(session_id: SessionId, room_data: RoomData) -> Self {
        Self {
            session_id,
            room_data,
            expected_board: None,
        }
    }
}
