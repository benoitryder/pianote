use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::Receiver;
use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::synth::Synth;
use crate::midi::MidiMessage;


pub struct AudioOutput {
    stream: cpal::Stream,
    synth: Arc<Mutex<Synth>>,
}

impl AudioOutput {
    pub fn new(queue: Receiver<MidiMessage>) -> Result<Self> {

        let host = cpal::default_host();
        let device = host.default_output_device().context("no audio output device available")?;

        let config = Self::get_output_config(&device)?;

        let synth = Synth::new(config.sample_rate.0 as f64)?;
        let synth = Arc::new(Mutex::new(synth));

        let audio_synth = Arc::clone(&synth);
        let data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let synth = audio_synth.lock().unwrap();
            // Convert input MIDI messages
            for message in queue.try_iter() {
                synth.send_midi_message(message)
                    .unwrap_or_else(|err| eprintln!("failed to process MIDI message: {}", err));
            }

            // The stream and the synth use the same buffer format
            synth.write_samples(data.as_mut()).expect("failed to write samples");
        };
        let err_fn = |err| eprintln!("an error occurred on audio stream: {}", err);

        let stream = device.build_output_stream(
            &config,
            data_fn,
            err_fn,
        )?;

        Ok(Self { stream, synth })
    }

    pub fn play(&self) -> Result<()> {
        self.stream.play()?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.stream.pause()?;
        Ok(())
    }

    pub fn lock_synth(&self) -> MutexGuard<'_, Synth> {
        self.synth.lock().unwrap()
    }

    /// Get a suitable output config
    fn get_output_config(device: &cpal::Device) -> Result<cpal::StreamConfig> {
        for configs in device.supported_output_configs()? {
            if configs.channels() == 2 && configs.sample_format() == cpal::SampleFormat::F32 {
                return Ok(configs.with_max_sample_rate().config());
            }
        }
        anyhow::bail!("no stereo audio output configuration");
    }
}

