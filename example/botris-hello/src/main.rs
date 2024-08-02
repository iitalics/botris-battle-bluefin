//! Botris API example.

#[macro_use]
extern crate log;

use tokio_tungstenite::tungstenite;
use tungstenite::http;

use anyhow::{Context, Result};
use clap::Parser;
use futures::StreamExt;
use http::Uri;
use tokio_tungstenite::connect_async as ws_connect_async;
use tungstenite::Message as WSMessage;

use botris::Message as BotrisMessage;

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
        .parse_filters("botris_api=debug")
        .init();

    let Args { room_key, token } = Args::parse();

    let uri = format!("wss://botrisbattle.com/ws?token={token}&roomKey={room_key}");
    trace!("uri={uri:?}");
    let uri: Uri = uri.parse().context("invalid URI")?;

    let (mut ws, _res) = ws_connect_async(&uri).await.context("connect error")?;

    println!("{}", botris::GameInfo::default());

    use std::time::Instant;
    let t0 = Instant::now();

    while let Some(msg) = ws.next().await {
        let msg: WSMessage = msg.context("read error")?;
        let dt = Instant::now() - t0;

        if msg.is_text() {
            trace!("{msg:?}");
            let msg = msg.to_text().unwrap().parse::<BotrisMessage>()?;
            println!();
            println!("[{:>8.2}] {msg:?}", dt.as_secs_f32());
        } else {
            debug!("{msg:?}");
        }
    }

    Ok(())
}
