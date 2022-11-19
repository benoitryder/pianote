use std::sync::mpsc::{self, Sender};
use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::midi::{MidiInput, MidiInputPort, MidiSource};
use crate::audio::AudioOutput;
use crate::synth::SynthCommand;


pub struct Piano {
    #[allow(dead_code)]
    midi_source: Option<MidiSource>,
    audio_output: AudioOutput,
    synth_queue: Sender<SynthCommand>,
}

impl Piano {
    /// Use the first available input MIDI port
    pub fn with_default_port() -> Result<Self> {
        let midi = MidiInput::new()?;
        let port = midi.default_port().context("no MIDI input port")?;
        Self::from_midi_and_port(midi, port)
    }

    /// Use input MIDI port with the given name
    pub fn with_port_name(port: &str) -> Result<Self> {
        let midi = MidiInput::new()?;
        let port = midi.ports()?.into_iter().find(|p| p.name() == port)
            .with_context(|| format!("MIDI input port not found: {}", port))?;
        Self::from_midi_and_port(midi, port)
    }

    fn from_midi_and_port(midi: MidiInput, port: MidiInputPort) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let midi_source = midi.connect_queue(port, tx.clone())?;
        let audio_output = AudioOutput::new(rx)?;
        Ok(Self {
            midi_source: Some(midi_source),
            audio_output,
            synth_queue: tx,
        })
    }

    /// Don't use a midi input (intended for tests/debug)
    pub fn without_input() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let audio_output = AudioOutput::new(rx)?;
        Ok(Self {
            midi_source: None,
            audio_output,
            synth_queue: tx,
        })
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

    pub fn set_gain(&self, gain: f32) -> Result<()> {
        eprintln!("Set gain: {}", gain);
        self.synth_queue.send(SynthCommand::SetGain(gain))?;
        Ok(())
    }
}

