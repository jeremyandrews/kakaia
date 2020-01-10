use std::io::Write;
use std::sync::Mutex;

use actix_web::{HttpServer, App, Responder, web, FromRequest};
use chrono::{DateTime, Utc};
use structopt::StructOpt;
use tempfile::NamedTempFile;

pub mod speech;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "kakaia")]
struct Configuration {
    /*
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
    */

    /// Listen on IP:port
    #[structopt(short, long, default_value = "0.0.0.0:8088")]
    listen: String,

    /// Max bytes for audio files
    #[structopt(short, long, default_value = "4194304")]
    bytes: usize,

    /// Permanently store a copy of audio and text
    #[structopt(short, long)]
    store: bool,
}

fn audio_to_text(config_data: web::Data<Mutex<Configuration>>, deepspeech_data: web::Data<Mutex<speech::KakaiaDeepSpeech>>, base64_audio: String) -> impl Responder {
    let config = config_data.lock().unwrap();
    let mut kakaia_deepspeech = deepspeech_data.lock().unwrap();

    // Load audio.bytes from String
    let audio_bytes = match base64::decode(&base64_audio) {
        Ok(audio) => audio,
        Err(e) => {
            // @TODO: logging, properly handle this error
            let error = format!("failed to decode audio.data: {}", e);
            eprintln!("{}", &error);
            return error;
        }
    };

    // Create a temporary file.
    let mut temporary_file = match NamedTempFile::new() {
        Ok(f) => f,
        Err(e) => {
            return format!("failed to create temporary file: {}", e);
        }
    };

    // Grab a pointer to the beginning of the file.
    let audio_file = match temporary_file.reopen() {
        Ok(a) => a,
        Err(e) => {
            return format!("failed to open temporary file: {}", e);
        }
    };

    // Write audio.bytes into temporary file.
    let mut pos = 0;
    while pos < audio_bytes.len() {
        let bytes_written = match temporary_file.write(&audio_bytes[pos..]) {
            Ok(b) => b,
            Err(e) => {
                return format!("failed to create temporary file: {}", e);
            }
        };
        pos += bytes_written;
    }

    // Convert audio file to text.
    let converted: speech::AudioAsText = kakaia_deepspeech.convert_audio_to_text(audio_file);

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
                        return "error archiving audio file".to_string();
                    }
                };
                // Write audio.bytes into temporary file.
                let mut pos = 0;
                while pos < audio_bytes.len() {
                    let bytes_written = match buffer.write(&audio_bytes[pos..]) {
                        Ok(b) => b,
                        Err(e) => {
                            return format!("failed to write archive file: {}", e);
                        }
                    };
                    pos += bytes_written;
                }
                let mut buffer = match std::fs::File::create(format!("{}/audio-{}-{}-{}.txt", archive_directory, &hour, &minute, &second)) {
                    Ok(b) => b,
                    Err(e) => {
                        // @TODO: deal with this gracefully
                        eprintln!("failed to create archive text conversion of audio file: {}", e);
                        return "error archiving text conversion of audio file".to_string();
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

    // Debug output for now:
    println!("{}", &converted.text);
    // Return text
    format!("{}\n", converted.text)
}

fn main() {
    // @TODO: do we really need three copies of this?
    // Configuration structure for server configuration
    let config_server = Configuration::from_args();
    // Configuration structure for client configuration
    let config_web = config_server.clone();
    // Configuration structure for client process
    let config_data = web::Data::new(Mutex::new(config_server.clone()));
    let deepspeech_data = web::Data::new(Mutex::new(speech::KakaiaDeepSpeech::new()));

    let server = HttpServer::new(move || {
        App::new()
            .register_data(config_data.clone())
            .register_data(deepspeech_data.clone())
            .service(
                web::resource("/convert/audio/text").data(
                    String::configure(|cfg| {
                        // limit audio file size in bytes (defaults to 4MB)
                        cfg.limit(config_web.bytes)
                    }))
                    .route(web::post().to(audio_to_text)),
                )
    });
    // @TODO: handle errors
    server.bind(&config_server.listen)
        .unwrap()
        .run()
        .unwrap();
}
