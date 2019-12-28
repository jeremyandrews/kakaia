use actix_web::{HttpServer, App, Responder, web, FromRequest};
use serde::Deserialize;

#[derive(Deserialize)]
struct AudioDetail {
    filename: String,
    //audio: String,
}

fn audio_to_text(audio_detail: web::Json<AudioDetail>) -> impl Responder {
    format!("processing audio file: {}\n", audio_detail.filename)
}

fn main() {
    HttpServer::new(|| {
        App::new().service(
            web::resource("/convert/audio/text").data(
                web::Json::<AudioDetail>::configure(|cfg| {
                    // for now limit audio file size to 4MB
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
