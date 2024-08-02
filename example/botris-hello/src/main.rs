//! Botris API example.

#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use clap::Parser;
use futures::{SinkExt as _, StreamExt as _};
use tokio_tungstenite::connect_async as ws_connect_async;
use tokio_tungstenite::tungstenite;
use tungstenite::http::Uri;

use botris::{ClientMessage, Command, Message, RoomData, SessionId, UnknownMessage};

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
        .parse_filters("botris_hello=info")
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
        let msg = ws_msg.parse::<Message>()?;

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
                let new_session = Session {
                    room_data: preauth_room_data.take().unwrap(),
                    session_id: preauth_session_id.take().unwrap(),
                };
                info!("new session: {}", new_session.session_id);
                session = Some(new_session);
            } else {
                continue;
            }
        }

        let session = session.as_mut().unwrap();
        trace!("> {msg:?}");

        match msg {
            Message::GameStarted => {
                info!("game started");
            }

            Message::GameReset { room_data } => {
                info!("game reset");
                session.room_data = room_data;
            }

            Message::PlayerJoined { player_data } => {
                session.room_data.players.push(player_data);
            }

            Message::PlayerLeft { session_id } => {
                session
                    .room_data
                    .players
                    .retain(|p| p.session_id != session_id);
            }

            Message::PlayerAction { session_id, game_state } => {
                for pl in session.room_data.players.iter_mut() {
                    if pl.session_id == session_id {
                        pl.game_state = Some(game_state);
                        break;
                    }
                }
            }

            Message::RoundStarted {
                starts_at,
                room_data,
            } => {
                use std::time::{Duration, SystemTime};

                let t0 = SystemTime::now();
                let t1 = SystemTime::UNIX_EPOCH + Duration::from_millis(starts_at);

                let dt = t1.duration_since(t0).map_or(0.0, |d| d.as_secs_f32());
                info!("round starting: {dt:.3}");
                session.room_data = room_data;
            }

            Message::RequestMove {
                game_state,
                players,
            } => {
                info!("move requested");
                debug!("{game_state:?}");
                debug!("{players:?}");

                session.room_data.players = players;

                let commands = &[Command::RotateCw];
                let cmsg = ClientMessage::Action { commands };
                debug!("< {cmsg}");
                let ws_cmsg = tungstenite::Message::text(cmsg.to_string());
                ws.send(ws_cmsg).await.context("send error")?;
            }

            Message::Error(message) => {
                warn!("error: {message}");
            }

            _ => {
                if log_enabled!(log::Level::Debug) {
                    debug!("ignoring: {msg:?}");
                }
            }
        }

        session.print_game();
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
    fn print_game(&self) {
        let players = &self.room_data.players;

        trace!("printing {} players", players.len());
        if players.is_empty() {
            return;
        }

        let mut lines: Vec<Vec<String>> = players.iter().map(|_| vec![]).collect();

        for (lns, pl) in lines.iter_mut().zip(players.iter()) {
            let mut board_lns = [[" "; 12]; 20];
            for ln in board_lns.iter_mut() {
                ln[0] = "|";
                ln[11] = "|";
            }

            let mut held = "";
            let mut current = "";
            let mut queue = String::new();

            if let Some(game) = &pl.game_state {
                for (y, row) in game.board.as_ref().iter().enumerate() {
                    if y >= 20 {
                        break;
                    }
                    for (x, &block) in row.iter().enumerate() {
                        if x >= 10 {
                            break;
                        }
                        if let Some(block) = block {
                            board_lns[19 - y][x + 1] = block.name();
                        }
                    }
                }

                if let Some(p) = game.held {
                    held = p.name();
                }
                current = game.current.piece.name();
                queue = game.queue.iter().map(|p| p.name()).collect();
            }

            let id = &pl.session_id;
            let me = if *id == self.session_id { "*" } else { "" };
            lns.push(format!("id: {me}{id}"));
            lns.push(format!("queue: [{held}]({current}){queue}"));
            lns.push(["-"; 12].into_iter().collect());
            lns.extend(board_lns.into_iter().map(|ln| ln.into_iter().collect()));
            lns.push(["-"; 12].into_iter().collect());
        }

        let widths: Vec<usize> = lines
            .iter()
            .map(|lns| {
                let width = lns.iter().map(|ln| ln.len()).max().unwrap_or(0);
                width + 2
            })
            .collect();

        trace!("widths: {widths:?}");

        for l in 0..lines[0].len() {
            for (i, lns) in lines.iter().enumerate() {
                print!("{:1$}", lns[l], widths[i]);
            }
            println!();
        }
    }
}
