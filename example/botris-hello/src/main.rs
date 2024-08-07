//! Botris API example.

#[macro_use]
extern crate tracing;

use anyhow::{Context, Result};
use clap::Parser;
use futures::{SinkExt as _, StreamExt as _};
use tokio_tungstenite::connect_async as ws_connect_async;
use tokio_tungstenite::tungstenite;
use tungstenite::http::Uri;

use botris::api::{ClientMessage, Message, RoomData, SessionId, UnknownMessage};
use botris::game::Command;

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
    tracing_subscriber::fmt()
        //.with_env_filter("botris=debug")
        .with_env_filter("botris=info,bluefin=debug")
        .with_ansi(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .compact()
        .init();

    debug!("hello");

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
                    if enabled!(tracing::Level::WARN) {
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

                let game_state = session.room_data.players.iter().find_map(|p| {
                    if p.session_id == session.session_id {
                        p.game_state.as_ref()
                    } else {
                        None
                    }
                });

                if let Some(game_state) = game_state {
                    debug!(
                        "> {:?} {:?} {:?}",
                        &game_state.current.piece, &game_state.held, &game_state.queue
                    );
                } else {
                    warn!("did not get to peek at pre-round game state");
                }
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
                debug!("move requested");
                debug!(
                    "> {:?} {:?} {:?}",
                    game_state.current.piece, game_state.held, game_state.queue
                );

                let result = {
                    fn mino_piece_type(pc: botris::Piece) -> mino::standard_rules::PieceType {
                        match pc {
                            botris::Piece::I => mino::standard_rules::I,
                            botris::Piece::J => mino::standard_rules::J,
                            botris::Piece::L => mino::standard_rules::L,
                            botris::Piece::O => mino::standard_rules::O,
                            botris::Piece::S => mino::standard_rules::S,
                            botris::Piece::T => mino::standard_rules::T,
                            botris::Piece::Z => mino::standard_rules::Z,
                        }
                    }

                    fn botris_command(inp: mino::input::Input) -> botris::Command {
                        match inp {
                            mino::Input::Left => botris::Command::MoveLeft,
                            mino::Input::Right => botris::Command::MoveRight,
                            mino::Input::Cw => botris::Command::RotateCw,
                            mino::Input::Ccw => botris::Command::RotateCcw,
                            mino::Input::SonicDrop => botris::Command::SonicDrop,
                        }
                    }

                    let current = mino_piece_type(game_state.current.piece);
                    let hold = game_state.held.map(mino_piece_type);
                    let queue = game_state
                        .queue
                        .iter()
                        .map(|&x| mino_piece_type(x))
                        .collect::<Vec<_>>();

                    let mut matrix = mino::MatBuf::new();
                    for (y, row) in game_state.board.rows().iter().enumerate() {
                        for (x, cell) in row.iter().enumerate() {
                            if cell.is_some() {
                                matrix.set(y as i8, 1u16 << x);
                            }
                        }
                    }

                    bluefin::bot(current, &queue, hold, &matrix).map(|(hold, inputs)| {
                        let mut cmds = Vec::with_capacity(inputs.len() + 1);
                        if hold {
                            cmds.push(Command::Hold);
                        }
                        cmds.extend(inputs.iter().map(|&i| botris_command(i)));
                        cmds
                    })
                };

                if let Some(commands) = &result {
                    let msg = ClientMessage::Action { commands };
                    let ws_msg = tungstenite::Message::text(msg.to_string());
                    ws.send(ws_msg).await.context("send error")?;
                } else {
                    warn!("bot did not return a move; giving up");
                }
            }

            Message::PlayerAction {
                session_id,
                commands,
                game_state,
            } => {
                debug!("{session_id}: {commands:?}, {game_state:?}");
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
                debug!("ignoring: {msg:?}");
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
}

impl Session {
    fn new(session_id: SessionId, room_data: RoomData) -> Self {
        Self {
            session_id,
            room_data,
        }
    }
}
