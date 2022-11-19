use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender};
use std::path::Path;
use anyhow::{Context, Result};
use fluidlite::{IsFont, IsPreset};
use crate::audio::{AudioOutput, AudioOutputConfig};
use crate::midi::{MidiInput, MidiMessage};
use crate::synth::Synth;


pub struct Piano {
    /// Output audio stream
    output: AudioOutput,
    /// Queue to be used by inputs
    input_tx: Sender<MidiMessage>,
    /// Currently active input
    input: Option<Box<dyn std::any::Any>>,
    /// Synth used to generate output samples
    synth: Arc<Mutex<Synth>>,
    /// Currently loaded and active FontId
    sfont_id: Option<fluidlite::FontId>,
    /// Data of currently available presets
    presets_data: Vec<PresetData>,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Preset {
    pub bank: u32,
    pub num: u32,
}

pub struct PresetData {
    pub bank: u32,  // 7-bit value
    pub num: u32,  // 7-bit value
    pub name: Option<String>,
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
            input_tx: tx,
            input: None,
            synth,
            sfont_id: None,
            presets_data: vec![],
        })
    }

    pub fn set_input<I: PianoInput>(&mut self, input: I) -> Result<()> {
        self.input.replace(input.connect_input(self.input_tx.clone())?);
        Ok(())
    }

    pub fn has_input(&self) -> bool {
        self.input.is_some()
    }

    pub fn play(&self) -> Result<()> {
        self.output.play()
    }

    pub fn pause(&self) -> Result<()> {
        self.output.pause()
    }

    /// Change synth gain
    pub fn set_gain(&self, gain: f32) {
        let synth = &self.synth.lock().unwrap().synth;
        synth.set_gain(gain);
    }

    /// Load a new SoundFont file
    pub fn load_sfont<P: AsRef<Path>>(&mut self, filename: P) -> Result<()> {
        let synth = &self.synth.lock().unwrap().synth;

        // Load the new SoundFont file
        if let Some(sfont_id) = self.sfont_id {
            synth.sfunload(sfont_id, true)?;
            self.sfont_id = None;
        }
        let sfont_id = synth.sfload(filename, true)?;
        let sfont = synth.get_sfont_by_id(sfont_id).unwrap();

        // Get presets data
        let presets_data = (0..=127)
            .flat_map(|bank| (0..=127).map(move |num| (bank, num)))
            .filter_map(move |(bank, num)| {
                sfont
                    .get_preset(bank, num)
                    .map(|preset| PresetData {
                        bank,
                        num,
                        name: preset.get_name().map(|s| s.into()),
                    })
            })
            .collect();

        // Update instance fields
        self.sfont_id = Some(sfont_id);
        self.presets_data = presets_data;

        Ok(())
    }

    /// Return the current preset
    pub fn get_active_preset(&self) -> Result<Preset> {
        let synth = &self.synth.lock().unwrap().synth;
        let (_, bank, num) = synth.get_program(0)?;
        Ok(Preset { bank, num })
    }

    /// Change currently active preset
    pub fn set_active_preset(&self, preset: Preset) -> Result<()> {
        let sfont_id = self.sfont_id.context("no active SoundFont")?;
        let synth = &self.synth.lock().unwrap().synth;
        synth.program_select(0, sfont_id, preset.bank, preset.num)?;
        Ok(())
    }

    /// Return data of all available presets
    pub fn presets_data(&self) -> &[PresetData] {
        &self.presets_data
    }
}


impl From<&PresetData> for Preset {
    fn from(o: &PresetData) -> Self {
        Self { bank: o.bank, num: o.num }
    }
}


/// Piano input, generating MIDI events 
pub trait PianoInput {
    /// Connect the input to the given queue
    ///
    /// Input must be disconnected with returned data is dropped.
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

