use std::env;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use actix_web::{web, HttpResponse};
use audrey::read::Reader;
use audrey::sample::interpolate::{Converter, Linear};
use audrey::sample::signal::{from_iter, Signal};
use chrono::{DateTime, Utc};
use deepspeech::Model;
use natural::tokenize::tokenize;
use tempfile::NamedTempFile;

use crate::Configuration;
use crate::stopwords::StopWords;


// These constants are taken from the C++ sources of the client.
const BEAM_WIDTH :u16 = 500;
const LM_WEIGHT :f32 = 0.75;
const VALID_WORD_COUNT_WEIGHT :f32 = 1.85;
// The provided model was trained on this specific sample rate.
const SAMPLE_RATE :u32 = 16_000;

#[derive(Debug)]
pub struct AudioAsText<'a> {
    pub text: String,
    pub tokenized: Option<Vec<&'a str>>,
    pub filtered: Option<Vec<&'a str>>,
    pub extension: String,
}

pub struct KakaiaDeepSpeech {
    model: deepspeech::Model,
}
unsafe impl Send for KakaiaDeepSpeech {}

impl KakaiaDeepSpeech {
    pub fn new() -> Self {
        const DEEPSPEECH_MODELS_ENV: &str = "DEEPSPEECH_MODELS";
        let model_dir = match env::var(DEEPSPEECH_MODELS_ENV) {
            Ok(d) => d,
            Err(_) => {
                let default_dir = env::current_dir().unwrap().join("models/");
                eprintln!("DeepSpeechModel: {} isn't set, defaulting to {:?}", DEEPSPEECH_MODELS_ENV, default_dir);
                default_dir.to_str().unwrap().to_string()
            }
        };
        let dir_path = Path::new(&model_dir);
        let mut deepspeech_model = match Model::load_from_files(&dir_path.join("output_graph.pb"), BEAM_WIDTH) {
            Ok(m) => m,
            Err(_) => {
                eprintln!("FATAL ERROR, {:?} is an invalid models path", dir_path);
                std::process::exit(1);
            }
        };
        deepspeech_model.enable_decoder_with_lm(&dir_path.join("lm.binary"), &dir_path.join("trie"), LM_WEIGHT, VALID_WORD_COUNT_WEIGHT);
        KakaiaDeepSpeech {
            model: deepspeech_model,
        }
    }

    pub fn convert_audio_to_text(&mut self, audio_file: std::fs::File) -> AudioAsText {
        // Read audio from temporary file.
        let mut reader = match Reader::new(&audio_file) {
            Ok(r) => r,
            Err(e) => {
                let error = format!("failed to load audio file ({:?}): {}\n", audio_file, e);
                return AudioAsText {
                    text: error,
                    tokenized: None,
                    filtered: None,
                    extension: "unknown".to_string(),
                }
            }
        };

        let desc = reader.description();
        // Validate the audio file.
        let mut errors: Vec<String> = vec![];
        if desc.channel_count() != 1 {
            let error = format!("audio file must have exactly 1 track, not {}", desc.channel_count());
            eprintln!("{}", &error);
            errors.push(error);
        }
        if desc.sample_rate() != SAMPLE_RATE {
            let error = format!("audio sample rate must be {}, not {}", SAMPLE_RATE, desc.sample_rate());
            eprintln!("{}", &error);
            errors.push(error);
        }
        if errors.len() > 0 {
            let error = format!("{:?}", errors);
            return AudioAsText {
                text: error,
                tokenized: None,
                filtered: None,
                extension: "unknown".to_string(),
            }
        }

        // Obtain the buffer of samples
        let audio_buffer :Vec<_> = if desc.sample_rate() == SAMPLE_RATE {
            reader.samples().map(|s| s.unwrap()).collect()
        } else {
            // We need to interpolate to the target sample rate
            let interpolator = Linear::new([0i16], [0]);
            let conv = Converter::from_hz_to_hz(
                from_iter(reader.samples::<i16>().map(|s| [s.unwrap()])),
                interpolator,
                desc.sample_rate() as f64,
                SAMPLE_RATE as f64);
            conv.until_exhausted().map(|v| v[0]).collect()
        };

        let extension = match desc.format() {
            audrey::Format::Flac => "flac".to_string(),
            audrey::Format::OggVorbis => "ogg".to_string(),
            audrey::Format::Wav => "wav".to_string(),
            audrey::Format::CafAlac => "caf".to_string(),
        };

        let text = match self.model.speech_to_text(audio_buffer.as_slice()) {
            Ok(t) => t,
            Err(e) => {
                // @TODO: handle this gracefully
                eprintln!("Unexpected error converting audio to text: {}", e);
                "Unexpected error: failed to convert audio to text".to_string()
            }
        };

        AudioAsText {
            text: text,
            tokenized: None,
            filtered: None,
            extension: extension,
        }
    }
}

pub async fn _audio_to_text(
        config: web::Data<Configuration>,
        stop_words: web::Data<StopWords>,
        deepspeech_data: web::Data<Mutex<KakaiaDeepSpeech>>,
        base64_audio: String
    ) -> HttpResponse {
    let mut kakaia_deepspeech = deepspeech_data.lock().unwrap();

    // Load audio.bytes from String
    let audio_bytes = match base64::decode(&base64_audio) {
        Ok(audio) => audio,
        Err(e) => {
            // @TODO: logging, properly handle this error
            let error = format!("failed to decode audio.data: {}\n", e);
            eprint!("{}", &error);
            return HttpResponse::InternalServerError()
                .content_type("plain/text")
                .body(error)
        }
    };

    // Create a temporary file.
    let mut temporary_file = match NamedTempFile::new() {
        Ok(f) => f,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("plain/text")
                .body(format!("failed to create temporary file: {}\n", e))
        }
    };

    // Grab a pointer to the beginning of the file.
    let audio_file = match temporary_file.reopen() {
        Ok(a) => a,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("plain/text")
                .body(format!("failed to open temporary file: {}\n", e))
        }
    };

    // Write audio.bytes into temporary file.
    let mut pos = 0;
    while pos < audio_bytes.len() {
        let bytes_written = match temporary_file.write(&audio_bytes[pos..]) {
            Ok(b) => b,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .content_type("plain/text")
                    .body(format!("failed to create temporary file: {}\n", e))
            }
        };
        pos += bytes_written;
    }

    // Convert audio file to text.
    let mut converted: AudioAsText = kakaia_deepspeech.convert_audio_to_text(audio_file);

    // Optionally store a copy of the audio and text
    if config.store {
        let now: DateTime<Utc> = Utc::now();
        let archive_directory = format!("archive/{}/{}/{}/", now.format("%Y"), now.format("%m"), now.format("%d"));
        match std::fs::create_dir_all(&archive_directory) {
            Ok(_) =>  {
                let hour = now.format("%H");
                let minute = now.format("%M");
                let second = now.format("%S");
                let mut buffer = match std::fs::File::create(format!("{}/audio-{}-{}-{}.{}", archive_directory, &hour, &minute, &second, converted.extension)) {
                    Ok(b) => b,
                    Err(e) => {
                        // @TODO: deal with this gracefully
                        eprintln!("failed to create archive copy of audio file: {}", e);
                        return HttpResponse::InternalServerError()
                            .content_type("plain/text")
                            .body(format!("failed to create archive copy of audio file: {}\n", e))
                    }
                };
                // Write audio.bytes into temporary file.
                let mut pos = 0;
                while pos < audio_bytes.len() {
                    let bytes_written = match buffer.write(&audio_bytes[pos..]) {
                        Ok(b) => b,
                        Err(e) => {
                            return HttpResponse::InternalServerError()
                                .content_type("plain/text")
                                .body(format!("failed to write archive file: {}\n", e))
                        }
                    };
                    pos += bytes_written;
                }
                let mut buffer = match std::fs::File::create(format!("{}/audio-{}-{}-{}.txt", archive_directory, &hour, &minute, &second)) {
                    Ok(b) => b,
                    Err(e) => {
                        // @TODO: deal with this gracefully
                        eprintln!("failed to create archive text conversion of audio file: {}", e);
                        return HttpResponse::InternalServerError()
                            .content_type("plain/text")
                            .body("error archiving text conversion of audio file\n".to_string())
                    }
                };
                match writeln!(buffer, "{}", &converted.text) {
                    Ok(_) => (),
                    Err(e) => eprintln!("failed to archive text conversion of audio file: {}", e),
                }
            }
            Err(e) => {
                eprintln!("failed to create directory: '{}', {}", &archive_directory, e);
            }
        }
    }

    // For now display debug output
    let to_tokenize = converted.text.to_string();
    converted.tokenized = Some(tokenize(&to_tokenize));
    converted.filtered = stop_words.filter(converted.tokenized.clone().unwrap());
    println!("{:?}", converted);

    // Return text
    HttpResponse::Ok()
        .content_type("plain/text")
        .body(converted.text)
}
