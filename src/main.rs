use actix_web::{HttpServer, App, Responder, web, FromRequest};
use tempfile::NamedTempFile;
use std::io::Write;
use audrey::read::Reader;

fn audio_to_text(base64_audio: String) -> impl Responder {
    // Load audio.bytes from String
    let audio_bytes = match base64::decode(&base64_audio) {
        Ok(audio) => audio,
        Err(e) => {
            // @TODO: logging, properly handle this error
            eprintln!("failed to decode audio.data: {}", e);
            return format!("failed to decode audio.data: {}", e);
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

    // Load audio from temporary file.
    let reader = match Reader::new(audio_file) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to load audio from temporary file: {}", e);
            return format!("failed to load audio from temporary file: {}", e);
        }
    };
    let desc = reader.description();
    format!("audio desc: '{:?}'", desc)
}

fn main() {
    HttpServer::new(|| {
        App::new().service(
            web::resource("/convert/audio/text").data(
                String::configure(|cfg| {
                    // limit audio file size to 4MB
                    // @TODO: expose as configuration
                    cfg.limit(4194304)
                }))
                .route(web::post().to(audio_to_text)),
            )
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
