use iced::{
    executor,
    Application,
    Command,
    Element,
    Settings,
    Theme,
};
use crate::piano::Piano;

struct Ui {
    piano: Piano,
    gain: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    GainChanged(f32),
}

impl Application for Ui {
    type Executor = executor::Default;
    type Flags = Piano;
    type Message = Message;
    type Theme = Theme;

    fn new(piano: Piano) -> (Self, Command<Self::Message>) {
        let ui = Self {
            piano,
            gain: 1.5,  // FluidSynth default "synth.gain" value
        };
        ui.piano.set_gain(ui.gain);
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
}

pub fn run(piano: Piano) -> iced::Result {
    Ui::run(Settings::with_flags(piano))
}

