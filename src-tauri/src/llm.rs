//use serde::{Deserialize, Serialize};
//
//#[derive(Serialize)]
//struct GenerateRequest<'a> {
//    model: &'a str,
//    prompt: String,
//    stream: bool,
//}
//
//#[derive(Deserialize)]
//struct GenerateResponse {
//    response: String,
//}
//
//pub struct Cleaner {
//    client: reqwest::blocking::Client,
//    model: String,
//    endpoint: String,
//}
//
//impl Cleaner {
//    pub fn new(model: &str) -> Self {
//        Cleaner {
//            client: reqwest::blocking::Client::new(),
//            model: model.to_string(),
//            endpoint: "http://localhost:11434/api/generate".to_string(),
//        }
//    }
//
//    pub fn clean(&self, raw_text: &str) -> String {
//        if raw_text.is_empty() {
//            return String::new();
//        }
//
//        let prompt = format!(
//            "You are a dictation cleanup assistant. Fix punctuation and \
//             capitalization, remove filler words like 'um' and 'uh', and \
//             correct obvious transcription errors. Keep the original \
//             meaning and wording as close as possible. Only output the \
//             cleaned text, nothing else, no explanations.\n\n\
//             Raw transcript: \"{}\"\n\n\
//             Cleaned text:",
//            raw_text
//        );
//
//        let request = GenerateRequest {
//            model: &self.model,
//            prompt,
//            stream: false,
//        };
//
//        let result = self.client.post(&self.endpoint).json(&request).send();
//
//        let cleaned = match result {
//            Ok(resp) => match resp.json::<GenerateResponse>() {
//                Ok(parsed) => parsed.response.trim().to_string(),
//                Err(e) => {
//                    eprintln!("failed to parse Ollama response: {e}");
//                    raw_text.to_string()
//                }
//            },
//
//            Err(e) => {
//                eprintln!("failed to reach Ollama: {e}");
//                raw_text.to_string()
//            }
//        };
//
//        strip_wrapping_quotes(&cleaned)
//    }
//}
//
//fn strip_wrapping_quotes(text: &str) -> String {
//    let trimmed = text.trim();
//    let quote_chars = ['"', '\'', '“', '”'];
//
//    if trimmed.len() >= 2 {
//        let first = trimmed.chars().next().unwrap();
//        let last = trimmed.chars().last().unwrap();
//        if quote_chars.contains(&first) && quote_chars.contains(&last) {
//            return trimmed[first.len_utf8()..trimmed.len() - last.len_utf8()]
//                .trim()
//                .to_string();
//        }
//    }
//
//    trimmed.to_string()
//}
