use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::{self, Sender};
use std::path::Path;
use anyhow::{Context, Result};
use crate::audio::{AudioOutput, AudioOutputConfig};
use crate::midi::{MidiInput, MidiInputPort, MidiMessage, MidiSource};
use crate::synth::Synth;


pub struct Piano {
    _input: Box<dyn std::any::Any>,
    output: AudioOutput,
    synth: Arc<Mutex<Synth>>,
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

        let output_config = AudioOutputConfig::new()?;
        let synth = Synth::new(output_config.sample_rate())?;
        let synth = Arc::new(Mutex::new(synth));

        let output = {
            let synth = Arc::clone(&synth);
            output_config.stream(move |data: &mut [f32]| {
                let synth = synth.lock().unwrap();
                // Convert input MIDI messages
                for message in rx.try_iter() {
                    synth.send_midi_message(message)
                        .unwrap_or_else(|err| eprintln!("failed to process MIDI message: {}", err));
                }
                // Write the next samples
                synth.write_samples(data)
                    .unwrap_or_else(|err| eprintln!("failed to generate samples: {}", err));
            })
        }?;

        Ok(Self {
            _input: Box::new(input.with_queue(tx)?),
            output,
            synth,
        })
    }

    fn lock_synth(&self) -> MutexGuard<'_, Synth> {
        self.synth.lock().unwrap()
    }

    pub fn play(&self) -> Result<()> {
        self.output.play()
    }

    pub fn pause(&self) -> Result<()> {
        self.output.pause()
    }

    pub fn load_sfont<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.lock_synth().load_sfont(path)?;
        Ok(())
    }

    pub fn set_gain(&self, gain: f32) {
        self.lock_synth().set_gain(gain);
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

