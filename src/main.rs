use actix_web::{HttpServer, App, Responder, web, FromRequest};
use serde::Deserialize;

#[derive(Deserialize)]
struct Audio {
    filename: String,
    data: String,
}

fn audio_to_text(audio: web::Json<Audio>) -> impl Responder {
    let audio_bytes = match base64::decode(&audio.data) {
        Ok(audio) => audio,
        Err(e) => {
            // @TODO: logging, properly handle this error
            eprintln!("failed to decode audio.data: {}", e);
            vec![]
        }
    };
    format!("audio file: '{}', bytes: '{:?}'", audio.filename, audio_bytes)
}

fn main() {
    HttpServer::new(|| {
        App::new().service(
            web::resource("/convert/audio/text").data(
                web::Json::<Audio>::configure(|cfg| {
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
