use std::sync::mpsc::Sender;
use anyhow::{Context, Result};

pub type MidiMessage = wmidi::MidiMessage<'static>;

pub struct MidiInput {
    midi: midir::MidiInput,
}

pub struct MidiInputPort(String);

pub struct MidiSource(midir::MidiInputConnection<()>);

impl MidiInput {
    pub fn new() -> Result<Self> {
        let midi = midir::MidiInput::new("midi-input")?;
        Ok(Self { midi })
    }

    pub fn default_port(&self) -> Option<MidiInputPort> {
        self.ports().ok().and_then(|ports| ports.into_iter().next())
    }

    pub fn ports(&self) -> Result<Vec<MidiInputPort>> {
        let ports = self.midi
            .ports()
            .into_iter()
            // 'port_name()' fails if port is not available anymore, ignore error
            .filter_map(move |p| self.midi.port_name(&p).ok())
            .map(MidiInputPort)
            .collect();
        Ok(ports)
    }

    pub fn connect_callback<F>(self, port: MidiInputPort, mut callback: F) -> Result<MidiSource>
    where
        F: FnMut(&[u8]) + Send + 'static,
    {
        let port_impl = self.midi
            .ports()
            .into_iter()
            .find(|p| self.midi.port_name(p).ok().as_ref() == Some(&port.0))
            .context("cannot find port")?;
        let connection = self.midi.connect(
            &port_impl,
            "input",
            move |_, data, ()| { callback(data); },
            (),
        )?;
        Ok(MidiSource(connection))
    }

    pub fn connect_queue(self, port: MidiInputPort, queue: Sender<MidiMessage>) -> Result<MidiSource> {
        self.connect_callback(port, move |data| {
            let message = wmidi::MidiMessage::try_from(data).expect("failed to parse MIDI message");
            if let Some(message) = message.drop_unowned_sysex() {
                queue.send(message).expect("failed to send MIDI message to the queue");
            }
        })
    }
}

impl MidiInputPort {
    pub fn name(&self) -> &str {
        &self.0
    }
}

