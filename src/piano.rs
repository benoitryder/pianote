use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::{self, Sender};
use std::path::Path;
use anyhow::{Context, Result};
use crate::audio::{AudioOutput, AudioOutputConfig};
use crate::midi::{MidiInput, MidiMessage};
use crate::synth::Synth;


pub struct Piano {
    output: AudioOutput,
    synth: Arc<Mutex<Synth>>,
    input_tx: Sender<MidiMessage>,
    input: Option<Box<dyn std::any::Any>>,
}

impl Piano {
    pub fn new() -> Result<Self> {
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
            output,
            synth,
            input_tx: tx,
            input: None,
        })
    }

    pub fn set_input<I: PianoInput>(&mut self, input: I) -> Result<()> {
        self.input.replace(input.connect_input(self.input_tx.clone())?);
        Ok(())
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


/// Piano input, generating MIDI events 
pub trait PianoInput {
    fn connect_input(self, queue: Sender<MidiMessage>) -> Result<Box<dyn std::any::Any>>;
}

/// MIDI input, with an optional port name to use
pub struct PianoMidiInput<'a>(pub Option<&'a str>);

impl<'a> PianoInput for PianoMidiInput<'a> {
    fn connect_input(self, queue: Sender<MidiMessage>) -> Result<Box<dyn std::any::Any>> {
        let midi = MidiInput::new()?;
        let port = if let Some(port_name) = self.0 {
            midi.ports()?.into_iter().find(|p| p.name() == port_name)
                .with_context(|| format!("MIDI input port not found: {}", port_name))?
        } else {
            midi.default_port().context("no MIDI input port")?
        };
        let source = midi.connect_queue(port, queue)?;
        Ok(Box::new(source))
    }
}

