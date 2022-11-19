use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};


/// Elements used to create an audio output stream
pub struct AudioOutputConfig {
    device: cpal::Device,
    config: cpal::StreamConfig,
}

/// An audio output stream
pub struct AudioOutput {
    stream: cpal::Stream,
}

impl AudioOutputConfig {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device().context("no audio output device available")?;
        let config = Self::get_output_config(&device)?;
        Ok(Self { device, config })
    }

    pub fn sample_rate(&self) -> f64 {
        self.config.sample_rate.0 as f64
    }

    /// Create a stream from a function called to write the next output samples  
    pub fn stream<S>(self, mut next_samples: S) -> Result<AudioOutput>
    where
        S: FnMut(&mut [f32]) + Send + 'static,
    {
        let data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            next_samples(data);
        };
        let err_fn = |err| eprintln!("an error occurred on audio stream: {}", err);

        let stream = self.device.build_output_stream(
            &self.config,
            data_fn,
            err_fn,
        )?;

        Ok(AudioOutput { stream })
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

impl AudioOutput {
    pub fn play(&self) -> Result<()> {
        self.stream.play()?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.stream.pause()?;
        Ok(())
    }
}

