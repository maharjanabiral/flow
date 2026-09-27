use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    pub fn load(model_path: &str) -> Self {
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .expect("failed to loadd whisper model");

        Transcriber { ctx }
    }

    pub fn transcribe(&self, audio: &[f32]) -> String {
        const MIN_SAMPLES: usize = 1600;

        if audio.len() < MIN_SAMPLES {
            println!("Skipping transcription");
        }

        let params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });

        let mut state = match self.ctx.create_state() {
            Ok(state) => state,
            Err(e) => {
                eprintln!("failed to create state: {e}");
                return String::new();
            }
        };

        if let Err(e) = state.full(params, audio) {
            eprintln!("whisper inference failed: {e}");
            return String::new();
        }

        let mut output = String::new();

        for segment in state.as_iter() {
            output.push_str(&segment.to_string());
            output.push(' ');
        }

        output.trim().to_string()
    }
}
