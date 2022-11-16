use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use pianote::{MidiInput, Piano};


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
    /// Input port to use (default: first one)
    #[arg(short, long, name = "NAME")]
    input: Option<String>,

    /// SoundFont file to use
    #[arg(short, long, name = "FILE")]
    sound_font: Option<PathBuf>,

    /// List ports and exit
    #[arg(long)]
    list_ports: bool,
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_ports {
        list_ports()?;
        return Ok(());
    }

    let piano = if let Some(input) = cli.input {
        Piano::with_port(&input)
    } else {
        Piano::new()
    }?;

    if let Some(path) = cli.sound_font {
        piano.load_sfont(path.to_owned())?;
    } else {
        println!("No SoundFont provided, using system default (if any)");
    }
    piano.play()?;

    println!("Playing...");
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

