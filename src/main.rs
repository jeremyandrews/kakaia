use std::sync::Mutex;

use actix_web::{HttpServer, App, web, FromRequest};
use structopt::StructOpt;

use crate::speech::KakaiaDeepSpeech;
use crate::stopwords::StopWords;

pub mod speech;
pub mod stopwords;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "kakaia")]
pub struct Configuration {
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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Configuration structure for server configuration
    let config_server = Configuration::from_args();
    // Configuration structure for client configuration
    let config_web = config_server.clone();
    // Initialize stop word hashset
    let stop_words = StopWords::new();
    // Initialize DeepSpeech models
    let deepspeech_data = web::Data::new(Mutex::new(KakaiaDeepSpeech::new()));

    HttpServer::new(move || {
        App::new()
            .service(
                web::resource("/convert/audio/text")
                .data(config_web.clone())
                .data(stop_words.clone())
                .app_data(deepspeech_data.clone())
                .app_data(String::configure(|cfg| {
                    // limit audio file size in bytes (defaults to 4MB)
                    cfg.limit(config_web.bytes)
                }))
                .route(web::post().to(speech::_audio_to_text))
            )
        })
        .bind(&config_server.listen)?
        .run()
        .await
}
