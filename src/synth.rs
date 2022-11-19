use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::midi::MidiMessage;


#[derive(Debug)]
pub enum SynthCommand {
    Midi(MidiMessage),
    LoadSfont(PathBuf),
    SetGain(f32),
}


pub struct Synth {
    synth: fluidlite::Synth,
    /// Currently loaded and active FontId
    sfont: Option<fluidlite::FontId>,
}

impl Synth {
    pub fn new(sample_rate: f64) -> Result<Self> {
        use fluidlite::IsSettings;

        let settings = fluidlite::Settings::new()?;
        settings.num("synth.sample-rate")
            .expect("synth.sample-rate setting not available")
            .set(sample_rate);

        let synth = fluidlite::Synth::new(settings)?;
        synth.set_gain(1.5); //TODO
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

    pub fn process_command(&mut self, command: SynthCommand) -> Result<()> {
        match command {
            SynthCommand::Midi(message) => self.send_midi_message(message),
            SynthCommand::LoadSfont(path) => self.load_sfont(path),
            SynthCommand::SetGain(gain) => { self.synth.set_gain(gain); Ok(()) },
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

    pub fn write_samples(&self, samples: &mut [f32]) -> Result<()> {
        self.synth.write(samples)?;
        Ok(())
    }
}

