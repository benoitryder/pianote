use std::sync::mpsc::{self, Sender};
use std::path::Path;
use anyhow::{Context, Result};
use crate::midi::{MidiInput, MidiInputPort, MidiMessage, MidiSource};
use crate::audio::AudioOutput;


pub struct Piano {
    _input: Box<dyn std::any::Any>,
    audio_output: AudioOutput,
}

impl Piano {
    /// Use the first available input MIDI port
    pub fn with_default_port() -> Result<Self> {
        let midi = MidiInput::new()?;
        let port = midi.default_port().context("no MIDI input port")?;
        Self::from_input((midi, port))
    }

    /// Use input MIDI port with the given name
    pub fn with_port_name(port: &str) -> Result<Self> {
        let midi = MidiInput::new()?;
        let port = midi.ports()?.into_iter().find(|p| p.name() == port)
            .with_context(|| format!("MIDI input port not found: {}", port))?;
        Self::from_input((midi, port))
    }

    /// Don't use a midi input (intended for tests/debug)
    pub fn without_input() -> Result<Self> {
        Self::from_input(())
    }

    fn from_input<I: PianoInput>(input: I) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        Ok(Self {
            _input: Box::new(input.with_queue(tx)?),
            audio_output: AudioOutput::new(rx)?,
        })
    }


    pub fn play(&self) -> Result<()> {
        self.audio_output.play()
    }

    pub fn pause(&self) -> Result<()> {
        self.audio_output.pause()
    }

    pub fn load_sfont<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.audio_output.lock_synth().load_sfont(path)?;
        Ok(())
    }

    pub fn set_gain(&self, gain: f32) {
        self.audio_output.lock_synth().set_gain(gain);
    }
}

trait PianoInput {
    type Holder: 'static;

    /// Create a piano input sending MIDI message to a queue
    fn with_queue(self, queue: Sender<MidiMessage>) -> Result<Self::Holder>;
}

impl PianoInput for (MidiInput, MidiInputPort) {
    type Holder = MidiSource;

    fn with_queue(self, queue: Sender<MidiMessage>) -> Result<Self::Holder> {
        let (midi, port) = self;
        midi.connect_queue(port, queue)
    }
}

impl PianoInput for () {
    type Holder = ();

    fn with_queue(self, _queue: Sender<MidiMessage>) -> Result<Self::Holder> {
        Ok(())  // No input
    }
}


