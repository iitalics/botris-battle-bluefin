#[macro_use]
extern crate tracing;

use anyhow::Result;
use botris::{Command, Game, GameState, Piece};
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("bluefin=info")
        .with_ansi(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .compact()
        .init();

    const PPS: f64 = 2.5;
    let delta = Duration::from_secs_f64(1.0 / PPS);

    let mut game = Game::new();

    loop {
        if game.dead {
            error!("died!");
            break;
        }

        let time = Instant::now();

        match request_move(&game) {
            Some(cmds) => {
                // TODO: list of events
                game.perform_commands(&cmds)
            }
            None => {
                error!("bot gave up!");
                break;
            }
        }

        info!("move calculated in {:.3}s", time.elapsed().as_secs_f64());
        print_game_state(&game);

        if let Some(wait) = delta.checked_sub(time.elapsed()) {
            std::thread::sleep(wait);
        }
    }

    Ok(())
}

fn print_game_state(game: &GameState) {
    let pcs = game.pieces_placed;
    let atk = game.score;
    let eff = if pcs > 0 {
        atk as f64 / pcs as f64
    } else {
        0.0
    };
    println!("pcs: {pcs}, atk: {atk}, eff: {eff:.3} app");

    let hold = game.held.map_or("", |pc| pc.name());
    let curr = game.current.piece.name();
    let next = game.queue.iter().map(|pc| pc.name()).collect::<String>();
    println!("queue: [{hold}]({curr}){next}");

    let mut rows = [[" "; 10]; 16];
    for (y, row) in rows.iter_mut().enumerate() {
        for (x, col) in row.iter_mut().enumerate() {
            if let Some(block) = game.board[(x as i8, y as i8)] {
                *col = block.name();
            }
        }
    }

    for row in rows.iter().rev() {
        let row_concat = row.iter().copied().collect::<String>();
        println!("|{row_concat}|");
    }
    println!("+----------+");
    println!();
}

fn request_move(game: &GameState) -> Option<Vec<Command>> {
    fn mino_piece_type(pc: Piece) -> mino::standard_rules::Piece {
        match pc {
            Piece::I => mino::standard_rules::I,
            Piece::J => mino::standard_rules::J,
            Piece::L => mino::standard_rules::L,
            Piece::O => mino::standard_rules::O,
            Piece::S => mino::standard_rules::S,
            Piece::T => mino::standard_rules::T,
            Piece::Z => mino::standard_rules::Z,
        }
    }

    fn botris_command(inp: mino::input::Input) -> Command {
        match inp {
            mino::Input::Left => Command::MoveLeft,
            mino::Input::Right => Command::MoveRight,
            mino::Input::Cw => Command::RotateCw,
            mino::Input::Ccw => Command::RotateCcw,
            mino::Input::SonicDrop => Command::SonicDrop,
        }
    }

    let current = mino_piece_type(game.current.piece);
    let hold = game.held.map(mino_piece_type);
    let queue = game
        .queue
        .iter()
        .map(|&x| mino_piece_type(x))
        .collect::<Vec<_>>();

    let mut matrix = mino::MatBuf::new();
    for (y, row) in game.board.rows().iter().enumerate() {
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
}
