use actix_web::{HttpServer, App, Responder, web, FromRequest};
use tempfile::tempfile;
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
    let temporary_file = match tempfile() {
        Ok(f) => f,
        Err(e) => {
            return format!("failed to create temporary file: {}", e);
        }
    };
    // Write audio.bytes into temporary file.
    match writeln!(&temporary_file, "{:?}", &audio_bytes) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("failed to write to temporary file: {}", e);
            return format!("failed to write to temporary file: {}", e);
        }
    }
    // Load audio from temporary file.
	let reader = match Reader::new(&temporary_file) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to load audio from temporary file: {}", e);
            return format!("failed to load audio from temporary file: {}", e);
        }
    };
	let desc = reader.description();

    format!("audio bytes: '{:?}' desc: '{:?}'", audio_bytes, desc)
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
