use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, SupportedStreamConfig};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

pub struct Recorder {
    _stream: Stream,
    pub config: SupportedStreamConfig,
    pub samples: Arc<Mutex<Vec<f32>>>,
    pub level: Arc<AtomicU32>,
}

impl Recorder {
    pub fn start_default_input() -> Self {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("no input device found");

        println!("Using input device: {}", device.description().unwrap());

        let config = device
            .default_input_config()
            .expect("failed to get default input config");

        let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let samples_clone = Arc::clone(&samples);

        let level: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
        let level_clone = Arc::clone(&level);

        let err_fn = |err| eprintln!("stream error: {}", err);

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                config.clone().into(),
                move |data: &[f32], _| {
                    samples_clone.lock().unwrap().extend_from_slice(data);

                    let peak = data.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
                    level_clone.store((peak * 1000.0) as u32, Ordering::Relaxed);
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                config.clone().into(),
                move |data: &[i16], _| {
                    let mut buf = samples_clone.lock().unwrap();
                    buf.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));

                    let peak = data.iter().fold(0i16, |a, &b| a.max(b.abs()));
                    let normalized = (peak as f32 / i16::MAX as f32) * 1000.0;
                    level_clone.store(normalized as u32, Ordering::Relaxed);
                },
                err_fn,
                None,
            ),

            sample_format => panic!("Unsupported sample format: {:?}", sample_format),
        }
        .expect("failed to build_input_stream");

        stream.play().expect("failed to start stream");

        Recorder {
            _stream: stream,
            config,
            samples,
            level,
        }
    }

    pub fn snapshot(&self) -> Vec<f32> {
        self.samples.lock().unwrap().clone()
    }
}
