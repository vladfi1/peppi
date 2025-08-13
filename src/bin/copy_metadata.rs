use std::env;
use std::fs::File;
use std::io::{Cursor, Read};

use peppi::game::immutable::Game;
use peppi::io::slippi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut quiet = false;
    let mut file_args = Vec::new();

    for arg in args.iter().skip(1) {
        if arg == "--quiet" || arg == "-q" {
            quiet = true;
        } else {
            file_args.push(arg);
        }
    }

    if file_args.len() != 2 {
        eprintln!("Usage: {} [--quiet|-q] <source.slp> <target.slp>", args[0]);
        eprintln!("Copies metadata from source to target:");
        eprintln!("  - Player Display Names");
        eprintln!("  - Connect Codes");
        eprintln!("  - Slippi UIDs");
        eprintln!("  - Session ID");
        eprintln!("  - Game Number");
        eprintln!("  - Tiebreaker Number");
        eprintln!("\nOptions:");
        eprintln!("  --quiet, -q    Suppress informational output");
        std::process::exit(1);
    }

    let source_path = file_args[0];
    let target_path = file_args[1];

    if !quiet {
        println!("Reading source file: {}", source_path);
    }
    let source_game = read_slippi_file(source_path)?;

    if !quiet {
        println!("Reading target file: {}", target_path);
    }
    let mut target_game = read_slippi_file(target_path)?;

    if !quiet {
        println!("Copying metadata...");
    }
    copy_metadata(&source_game, &mut target_game);

    if !quiet {
        println!("Writing modified file back to: {}", target_path);
    }
    write_slippi_file(&target_game, target_path)?;

    if !quiet {
        println!("Successfully copied metadata from {} to {}", source_path, target_path);
    }

    Ok(())
}

fn read_slippi_file(path: &str) -> Result<Game, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let game = slippi::read(Cursor::new(buf), None)?;
    Ok(game)
}

fn copy_metadata(source: &Game, target: &mut Game) {
    let source_start = &source.start;
    let target_start = &mut target.start;

    for (source_player, target_player) in source_start.players.iter().zip(target_start.players.iter_mut()) {
        target_player.name_tag = source_player.name_tag.clone();
        target_player.netplay = source_player.netplay.clone();
    }

    if let Some(ref source_match) = source_start.r#match {
        target_start.r#match = Some(source_match.clone());
    }
}

fn write_slippi_file(game: &Game, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::create(path)?;
    slippi::write(&mut file, game)?;
    Ok(())
}
