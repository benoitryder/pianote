use std::path::Path;
use anyhow::Result;
use fluidlite::{IsFont, IsPreset};
use crate::midi::MidiMessage;


/// Synthetizer, using SoundFont data and processing MIDI commands
pub struct Synth {
    synth: fluidlite::Synth,
    /// Currently loaded and active FontId
    sfont: Option<fluidlite::FontId>,
}

pub struct PresetData {
    pub bank: u32,  // 7-bit value
    pub num: u32,  // 7-bit value
    pub name: Option<String>,
}

impl Synth {
    pub fn new(sample_rate: f64) -> Result<Self> {
        use fluidlite::IsSettings;

        let settings = fluidlite::Settings::new()?;
        settings.num("synth.sample-rate")
            .expect("synth.sample-rate setting not available")
            .set(sample_rate);

        let synth = fluidlite::Synth::new(settings)?;
        synth.set_gain(1.5);  //XXX Arbitrary value
        Ok(Self { synth, sfont: None })
    }

    pub fn load_sfont<P: AsRef<Path>>(&mut self, filename: P) -> Result<()> {
        if let Some(font_id) = self.sfont.take() {
            self.synth.sfunload(font_id, true)?;
        }
        let font_id = self.synth.sfload(filename, true)?;
        self.sfont = Some(font_id);
        Ok(())
    }

    pub fn set_gain(&self, gain: f32) {
        self.synth.set_gain(gain);
    }

    /// Iterate on presets of the current SFont
    pub fn presets(&self) -> Vec<PresetData> {
        let sfont = self.sfont.and_then(|id| self.synth.get_sfont_by_id(id));
        if let Some(sfont) = sfont {
            (0..=127)
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
                .collect()
        } else {
            vec![]
        }
    }

    pub fn send_midi_message(&self, message: MidiMessage) -> Result<()> {
        use fluidlite::{Chan, Ctrl, Key, Prog, Val, Vel};
        match message {
            MidiMessage::NoteOff(chan, key, _) => self.synth.note_off(chan as Chan, key as Key),
            MidiMessage::NoteOn(chan, key, vel) => self.synth.note_on(chan as Chan, key as Key, u8::from(vel) as Vel),
            MidiMessage::PolyphonicKeyPressure(chan, key, vel) => self.synth.key_pressure(chan as Chan, key as Key, u8::from(vel) as Vel),
            MidiMessage::ControlChange(chan, ctrl, val) => self.synth.cc(chan as Chan, u8::from(ctrl) as Ctrl, u8::from(val) as Val),
            MidiMessage::ProgramChange(chan, prog) => self.synth.program_change(chan as Chan, u8::from(prog) as Prog),
            MidiMessage::ChannelPressure(chan, vel) => self.synth.channel_pressure(chan as Chan, u8::from(vel) as Vel),
            MidiMessage::PitchBendChange(chan, val) => self.synth.pitch_bend(chan as Chan, u16::from(val) as Val),
            MidiMessage::Reset => self.synth.system_reset(),
            _ => Ok(()),
        }?;
        Ok(())
    }

    /// Consume and write the next samples
    pub fn write_samples(&self, samples: &mut [f32]) -> Result<()> {
        self.synth.write(samples)?;
        Ok(())
    }
}

