use std::rc::{Rc, Weak};
use std::sync::mpsc::Sender;
use anyhow::Result;
use iced::{
    keyboard,
    keyboard::KeyCode,
    event,
    executor,
    subscription,
    Application,
    Command,
    Element,
    Event,
    Settings,
    Subscription,
    Theme,
};
use crate::piano::{Piano, PianoInput};
use crate::midi::MidiMessage;

struct Ui {
    piano: Piano,
    gain: f32,
    keyboard_input: Weak<PianoUiInput>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    GainChanged(f32),
    KeyNoteOn(wmidi::Note),
    KeyNoteOff(wmidi::Note),
}

impl Application for Ui {
    type Executor = executor::Default;
    type Flags = Piano;
    type Message = Message;
    type Theme = Theme;

    fn new(piano: Piano) -> (Self, Command<Self::Message>) {
        let mut ui = Self {
            piano,
            gain: 1.5,  // FluidSynth default "synth.gain" value
            keyboard_input: Weak::new(),
        };
        ui.piano.set_gain(ui.gain);

        // Enable the UI input if there is none yet 
        if !ui.piano.has_input() {
            ui.piano.set_input(&mut ui.keyboard_input)
                .unwrap_or_else(|err| eprintln!("failed to setup UI MIDI input: {}", err));
        }

        (ui, Command::none())
    }

    fn title(&self) -> String {
        "Pianote".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::GainChanged(gain) => {
                self.gain = gain;
                self.piano.set_gain(self.gain);
            }
            Message::KeyNoteOn(note) => {
                if let Some(input) = self.keyboard_input.upgrade() {
                    input.queue.send(MidiMessage::NoteOn(wmidi::Channel::Ch1, note, wmidi::U7::MAX)).unwrap();
                }
            }
            Message::KeyNoteOff(note) => {
                if let Some(input) = self.keyboard_input.upgrade() {
                    input.queue.send(MidiMessage::NoteOff(wmidi::Channel::Ch1, note, wmidi::U7::MAX)).unwrap();
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        use iced::widget::{row, slider, text};
        row![
            text(format!("Gain {:4.1}", self.gain)),
            slider(0.0..=10.0, self.gain, Message::GainChanged).step(0.1),
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| {
            if status == event::Status::Ignored {
                match event {
                    Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
                        Self::key_code_to_note(key_code).map(Message::KeyNoteOn)
                    },
                    Event::Keyboard(keyboard::Event::KeyReleased { key_code, .. }) => {
                        Self::key_code_to_note(key_code).map(Message::KeyNoteOff)
                    },
                    _ => None,
                }
            } else {
                None
            }
        })
    }
}

impl Ui {
    fn key_code_to_note(key_code: KeyCode) -> Option<wmidi::Note> {
        match key_code {
            KeyCode::E => Some(wmidi::Note::C4),
            KeyCode::Key4 => Some(wmidi::Note::Db4),
            KeyCode::R => Some(wmidi::Note::D4),
            KeyCode::Key5 => Some(wmidi::Note::Eb4),
            KeyCode::T => Some(wmidi::Note::E4),
            KeyCode::Y => Some(wmidi::Note::F4),
            KeyCode::Key7 => Some(wmidi::Note::Gb4),
            KeyCode::U => Some(wmidi::Note::G4),
            KeyCode::Key8 => Some(wmidi::Note::Ab4),
            KeyCode::I => Some(wmidi::Note::A4),
            KeyCode::Key9 => Some(wmidi::Note::Bb4),
            KeyCode::O => Some(wmidi::Note::B4),
            KeyCode::P => Some(wmidi::Note::C5),
            _ => None,
        }
    }
}


struct PianoUiInput {
    queue: Sender<MidiMessage>,
}

impl PianoInput for &mut Weak<PianoUiInput> {
    fn connect_input(self, queue: Sender<MidiMessage>) -> Result<Box<dyn std::any::Any>> {
        println!("connecting input");
        let input = Rc::new(PianoUiInput { queue });
        *self = Rc::downgrade(&input);
        Ok(Box::new(input))
    }
}


pub fn run(piano: Piano) -> iced::Result {
    Ui::run(Settings::with_flags(piano))
}

