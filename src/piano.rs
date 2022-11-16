use std::sync::mpsc::{self, Sender};
use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::midi::{MidiInput, MidiInputPort, MidiSource};
use crate::audio::AudioOutput;
use crate::synth::SynthCommand;


pub struct Piano {
    #[allow(dead_code)]
    midi_source: MidiSource,
    audio_output: AudioOutput,
    synth_queue: Sender<SynthCommand>,
}

impl Piano {
    pub fn new() -> Result<Self> {
        let midi = MidiInput::new()?;
        let port = midi.default_port().context("no MIDI input port")?;
        Self::from_midi_and_port(midi, port)
    }

    pub fn with_port(port: &str) -> Result<Self> {
        let midi = MidiInput::new()?;
        let port = midi.ports()?.into_iter().find(|p| p.name() == port)
            .with_context(|| format!("MIDI input port not found: {}", port))?;
        Self::from_midi_and_port(midi, port)
    }

    fn from_midi_and_port(midi: MidiInput, port: MidiInputPort) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let midi_source = midi.connect_queue(port, tx.clone())?;
        let audio_output = AudioOutput::new(rx)?;
        Ok(Self { midi_source, audio_output, synth_queue: tx })
    }

    pub fn play(&self) -> Result<()> {
        self.audio_output.play()
    }

    pub fn pause(&self) -> Result<()> {
        self.audio_output.pause()
    }

    pub fn load_sfont(&self, path: PathBuf) -> Result<()> {
        self.synth_queue.send(SynthCommand::LoadSfont(path))?;
        Ok(())
    }
}

