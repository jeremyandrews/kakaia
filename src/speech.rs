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
use tempfile::NamedTempFile;
use serde::Serialize;

use crate::nlu::NLU;
use crate::Configuration;

// These constants are taken from the C++ sources of the client.
const BEAM_WIDTH: u16 = 500;
const LM_WEIGHT: f32 = 0.75;
const VALID_WORD_COUNT_WEIGHT: f32 = 1.85;
// The provided model was trained on this specific sample rate.
const SAMPLE_RATE: u32 = 16_000;

#[derive(Debug)]
pub struct AudioAsText {
    pub raw: String,
    pub filetype: String,
}

#[derive(Debug, Serialize)]
pub struct KakaiaResponse {
    command: String,
    human: String,
    raw: String,
    result: f64,
}

#[derive(Debug)]
pub enum KakaiaCommandType {
    None,
    SetTimer,
    ConvertTemperature,
    SimpleCalculation,
}

#[derive(Debug)]
pub struct KakaiaCommand {
    command: KakaiaCommandType,
    string: String,
}

impl KakaiaCommand {
    pub fn from_str(command_str: &str) -> KakaiaCommand {
        let command_type = match command_str {
            "setTimer" => KakaiaCommandType::SetTimer,
            "convertTemperature" => KakaiaCommandType::ConvertTemperature,
            "simpleCalculation" => KakaiaCommandType::SimpleCalculation,
            _ => KakaiaCommandType::None,
        };
        KakaiaCommand {
            command: command_type,
            string: command_str.to_string(),
        }
    }

    pub fn to_string(command_type: KakaiaCommandType) -> String {
        match command_type {
            KakaiaCommandType::None => "none".to_string(),
            KakaiaCommandType::SetTimer => "setTimer".to_string(),
            KakaiaCommandType::ConvertTemperature => "convertTemperature".to_string(),
            KakaiaCommandType::SimpleCalculation => "simpleCalculation".to_string(),
        }
    }
}

impl KakaiaResponse {
    pub fn new(command: &str, human: &str, raw: &str, result: f64) -> Self {
        KakaiaResponse {
            command: command.to_string(),
            human: human.to_string(),
            raw: raw.to_string(),
            result: result,
        }
    }

    pub fn to_json_string(&self) -> String {
        match serde_json::to_string(&self) {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }
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
                eprintln!(
                    "DeepSpeechModel: {} isn't set, defaulting to {:?}",
                    DEEPSPEECH_MODELS_ENV, default_dir
                );
                default_dir.to_str().unwrap().to_string()
            }
        };
        let dir_path = Path::new(&model_dir);
        let mut deepspeech_model =
            match Model::load_from_files(&dir_path.join("output_graph.pb"), BEAM_WIDTH) {
                Ok(m) => m,
                Err(_) => {
                    eprintln!("FATAL ERROR, {:?} is an invalid models path", dir_path);
                    std::process::exit(1);
                }
            };
        deepspeech_model.enable_decoder_with_lm(
            &dir_path.join("lm.binary"),
            &dir_path.join("trie"),
            LM_WEIGHT,
            VALID_WORD_COUNT_WEIGHT,
        );
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
                    raw: error,
                    filetype: "unknown".to_string(),
                };
            }
        };

        let desc = reader.description();
        // Validate the audio file.
        if desc.channel_count() != 1 {
            let error = format!(
                "audio file must have exactly 1 track, not {}",
                desc.channel_count()
            );
            eprintln!("{}", &error);
            return AudioAsText {
                raw: error,
                filetype: "unknown".to_string(),
            };
        }

        // Obtain the buffer of samples
        let audio_buffer: Vec<_> = if desc.sample_rate() == SAMPLE_RATE {
            reader.samples().map(|s| s.unwrap()).collect()
        } else {
            // We need to interpolate to the target sample rate
            let interpolator = Linear::new([0i16], [0]);
            let conv = Converter::from_hz_to_hz(
                from_iter(reader.samples::<i16>().map(|s| [s.unwrap()])),
                interpolator,
                desc.sample_rate() as f64,
                SAMPLE_RATE as f64,
            );
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
            raw: text,
            filetype: extension,
        }
    }
}

pub async fn _audio_to_text(
    config: web::Data<Configuration>,
    deepspeech_data: web::Data<Mutex<KakaiaDeepSpeech>>,
    nlu_data: web::Data<Mutex<NLU>>,
    base64_audio: String,
) -> HttpResponse {
    let mut kakaia_deepspeech = deepspeech_data.lock().unwrap();
    let nlu = nlu_data.lock().unwrap();

    // Load audio.bytes from String
    let audio_bytes = match base64::decode(&base64_audio) {
        Ok(audio) => audio,
        Err(e) => {
            // @TODO: logging, properly handle this error
            let error = format!("failed to decode audio.data: {}\n", e);
            eprint!("{}", &error);
            let kakaia_response = KakaiaResponse::new("none", "unexpected error decoding audio data", &error, 0.0);
            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(kakaia_response.to_json_string());
        }
    };

    // Create a temporary file.
    let mut temporary_file = match NamedTempFile::new() {
        Ok(f) => f,
        Err(e) => {
            let error = format!("failed to create temporary file: {}", e);
            let kakaia_response = KakaiaResponse::new("none", "unexpected error creating a temporary file", &error, 0.0);
            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(kakaia_response.to_json_string());
        }
    };

    // Grab a pointer to the beginning of the file.
    let audio_file = match temporary_file.reopen() {
        Ok(a) => a,
        Err(e) => {
            let error = format!("failed to open temporary file: {}", e);
            let kakaia_response = KakaiaResponse::new("none", "unexpected error opening a temporary file", &error, 0.0);
            return HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(kakaia_response.to_json_string());
        }
    };

    // Write audio.bytes into temporary file.
    let mut pos = 0;
    while pos < audio_bytes.len() {
        let bytes_written = match temporary_file.write(&audio_bytes[pos..]) {
            Ok(b) => b,
            Err(e) => {
                let error = format!("failed to create temporary file: {}", e);
                let kakaia_response = KakaiaResponse::new("none", "unexpected error creating a temporary file", &error, 0.0);
                return HttpResponse::InternalServerError()
                    .content_type("application/json")
                    .body(kakaia_response.to_json_string());
            }
        };
        pos += bytes_written;
    }

    // Convert audio file to text.
    let converted: AudioAsText = kakaia_deepspeech.convert_audio_to_text(audio_file);

    // Optionally store a copy of the audio and text
    if config.store {
        let now: DateTime<Utc> = Utc::now();
        let archive_directory = format!(
            "archive/{}/{}/{}/",
            now.format("%Y"),
            now.format("%m"),
            now.format("%d")
        );
        match std::fs::create_dir_all(&archive_directory) {
            Ok(_) => {
                let hour = now.format("%H");
                let minute = now.format("%M");
                let second = now.format("%S");
                let mut buffer = match std::fs::File::create(format!(
                    "{}/audio-{}-{}-{}.{}",
                    archive_directory, &hour, &minute, &second, converted.filetype
                )) {
                    Ok(b) => b,
                    Err(e) => {
                        // @TODO: deal with this gracefully
                        let error = format!("failed to create archive copy of audio file: {}", e);
                        let kakaia_response = KakaiaResponse::new("none", "unexpected error archiving a copy of audio file", &error, 0.0);
                        return HttpResponse::InternalServerError()
                            .content_type("application/json")
                            .body(kakaia_response.to_json_string());
                    }
                };
                // Write audio.bytes into temporary file.
                let mut pos = 0;
                while pos < audio_bytes.len() {
                    let bytes_written = match buffer.write(&audio_bytes[pos..]) {
                        Ok(b) => b,
                        Err(e) => {
                            let error = format!("failed to write archive file: {}", e);
                            let kakaia_response = KakaiaResponse::new("none", "unexpected error writing a copy of audio file", &error, 0.0);
                            return HttpResponse::InternalServerError()
                                .content_type("application/json")
                                .body(kakaia_response.to_json_string());
                        }
                    };
                    pos += bytes_written;
                }
                let mut buffer = match std::fs::File::create(format!(
                    "{}/audio-{}-{}-{}.txt",
                    archive_directory, &hour, &minute, &second
                )) {
                    Ok(b) => b,
                    Err(e) => {
                        // @TODO: deal with this gracefully
                        let error = format!("failed to create archive text conversion of audio file: {}", e);
                        let kakaia_response = KakaiaResponse::new("none", "unexpected error writing text conversion of audio file", &error, 0.0);
                        return HttpResponse::InternalServerError()
                            .content_type("application/json")
                            .body(kakaia_response.to_json_string());
                    }
                };
                match writeln!(buffer, "{}", &converted.raw) {
                    Ok(_) => (),
                    Err(e) => eprintln!("failed to archive text conversion of audio file: {}", e),
                }
            }
            Err(e) => {
                let error = format!("failed to create directory '{}': {}", &archive_directory, e);
                let kakaia_response = KakaiaResponse::new("none", "unexpected error creating directory", &error, 0.0);
                return HttpResponse::InternalServerError()
                    .content_type("application/json")
                    .body(kakaia_response.to_json_string());
            }
        }
    }

    let parsed_json = nlu.parse(&converted.raw);
    //println!("NLU: {:?}", &parsed_json);

    let command_string = nlu.get_command(&parsed_json);
    let kakaia_command = KakaiaCommand::from_str(&command_string);

    let kakaia_response: KakaiaResponse = match kakaia_command.command {
        // no command, we do nothing
        KakaiaCommandType::None => {
            KakaiaResponse::new(
                &kakaia_command.string,
                "no command",
                &converted.raw,
                0.0
            )
        }
        // setTimer command, return how many seconds the timer should run
        KakaiaCommandType::SetTimer => {
            if nlu.has_expected_slots(&parsed_json, 1) {
                let seconds_value = nlu.get_slot_value(&parsed_json, "snips/duration", "duration");
                let seconds = nlu.duration_as_seconds(seconds_value.unwrap());
                KakaiaResponse::new(
                    &kakaia_command.string,
                    format!("set timer for {} seconds", seconds).as_str(),
                    &converted.raw,
                    seconds
                )
            } else {
                KakaiaResponse::new(
                    &kakaia_command.string,
                    "not understood",
                    &converted.raw,
                    0.0
                )

            }
        }
        // convertTemperature command, return converted temperature
        KakaiaCommandType::ConvertTemperature => {
            println!("ConvertTemperature: {:?}", parsed_json);
            if nlu.has_expected_slots(&parsed_json, 2) {
                let from_value = nlu.get_slot_value(&parsed_json, "snips/temperature", "from");
                let from_degrees = nlu.get_float(from_value);
                let from_scale = nlu.get_string_custom(from_value, "unit");
                let to_value = nlu.get_slot_value(&parsed_json, "temperature_name", "to");
                let to_scale = nlu.get_string(to_value);
                let result = match from_scale.as_str() {
                    "celsius" => {
                        match to_scale.as_str() {
                            "fahrenheit" => {
                                eprintln!("{} * 1.8 + 32", from_degrees);
                                from_degrees * 1.8 + 32.0
                            },
                            "kelvin" => {
                                from_degrees + 273.15
                            }
                            _ => {
                                eprintln!("{} to {} fell through", from_scale, to_scale);
                                0.0
                            }
                        }
                    },
                    "fahrenheit" => {
                        match to_scale.as_str() {
                            "celsius" => {
                                (from_degrees - 32.0) / 1.8
                            },
                            "kelvin" => {
                                (from_degrees - 32.0) / 1.8 + 273.15
                            }
                            _ => {
                                eprintln!("{} to {} fell through", from_scale, to_scale);
                                0.0
                            }
                        }
                    },
                    "kelvin" => {
                        match to_scale.as_str() {
                            "celsius" => {
                                from_degrees - 273.15
                            },
                            "fahrenheit" => {
                                (from_degrees - 273.15) * 1.8 + 32.0
                            }
                            _ => {
                                eprintln!("{} to {} fell through", from_scale, to_scale);
                                0.0
                            }
                        }
                    },
                    _ => {
                        eprintln!("{} to {} fell through", from_scale, to_scale);
                        0.0
                    }
                };
                KakaiaResponse::new(
                    &kakaia_command.string,
                    format!("{} degrees {} is {} degrees {}", from_degrees, from_scale, result, to_scale).as_str(),
                    &converted.raw,
                    result
                )
            } else {
                KakaiaResponse::new(
                    &kakaia_command.string,
                    "not understood",
                    &converted.raw,
                    0.0
                )
            }
        }
        // simpleCalculation command, return result of calculation
        KakaiaCommandType::SimpleCalculation => {
            //println!("SimpleCalculation: {:?}", parsed_json);
            if nlu.has_expected_slots(&parsed_json, 3) {
                let first_value = nlu.get_slot_value(&parsed_json, "snips/number", "first");
                let first = nlu.get_float(first_value);
                let second_value = nlu.get_slot_value(&parsed_json, "snips/number", "second");
                let second = nlu.get_float(second_value);
                let operation_value = nlu.get_slot_value(&parsed_json, "operation", "operation");
                let operation = nlu.get_string(operation_value);
                let operation_string;
                let result = match operation.as_str() {
                    "plus" => { 
                        operation_string = operation.to_string();
                        first + second
                    }
                    "minus" => {
                        operation_string = operation.to_string();
                        first - second
                    }
                    "multiply" => {
                        operation_string = "times".to_string();
                        first * second
                    }
                    "divide" => {
                        operation_string = "divided by".to_string();
                        first / second
                    }
                    _ => {
                        operation_string = "???".to_string();
                        0.0
                    }
                };
                KakaiaResponse::new(
                    &kakaia_command.string,
                    format!("{} {} {} equals {}", first, operation_string, second, result).as_str(),
                    &converted.raw,
                    result
                )
            } else {
                KakaiaResponse::new(
                    &kakaia_command.string,
                    "not understood",
                    &converted.raw,
                    0.0
                )
            }
        }
    };

    // Debug output for now
    println!("{:?}", &kakaia_response);
    return HttpResponse::Ok()
        .content_type("application/json")
        .body(kakaia_response.to_json_string());
}
