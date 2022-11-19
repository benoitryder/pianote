use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use pianote::{MidiInput, Piano, PianoMidiInput};


fn list_ports() -> Result<()> {
    let midi = MidiInput::new()?;
    let ports = midi.ports()?;
    if ports.is_empty() {
        println!("No input ports");
    } else {
        println!("Input ports");
        for port in ports {
            println!("  {}", port.name());
        }
    }
    Ok(())
}


#[derive(Parser)]
struct Cli {
    /// Input port to use, `NONE` to disable input (default: first input)
    #[arg(short, long, name = "NAME")]
    input: Option<String>,

    /// SoundFont file to use
    #[arg(short, long, name = "FILE")]
    sound_font: Option<PathBuf>,

    /// List ports and exit
    #[arg(long)]
    list_ports: bool,

    /// Run headless (no UI), implied if compiled without it
    #[arg(long)]
    headless: bool,
}

/// Run without UI
fn run_headless() {
    println!("Playing...");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_ports {
        list_ports()?;
        return Ok(());
    }

    let mut piano = Piano::new()?;
    match cli.input.as_deref() {
        Some("NONE") => {}
        input => piano.set_input(PianoMidiInput(input))?,
    };

    if let Some(path) = cli.sound_font {
        piano.load_sfont(path)?;
    } else {
        println!("No SoundFont provided, using system default (if any)");
    }
    piano.play()?;

    if cli.headless || !cfg!(feature = "ui") {
        run_headless();
    } else {
        pianote::ui::run(piano)?;
    }

    Ok(())
}

