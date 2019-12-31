use std::io::Write;
use std::path::Path;

use actix_web::{HttpServer, App, Responder, web, FromRequest};
use audrey::read::Reader;
use audrey::sample::interpolate::{Converter, Linear};
use audrey::sample::signal::{from_iter, Signal};
use chrono::{DateTime, Utc};
use deepspeech::Model;
use structopt::StructOpt;
use tempfile::NamedTempFile;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "kakaia")]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// listen on IP:port
    #[structopt(short, long, default_value = "0.0.0.0:8088")]
    listen: String,

    /// max bytes for audio files
    #[structopt(short, long, default_value = "4194304")]
    bytes: usize,
}

fn audio_to_text(base64_audio: String) -> impl Responder {
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

    // Load audio from temporary file.
    let mut reader = match Reader::new(audio_file) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to load audio from temporary file: {}", e);
            return format!("failed to load audio from temporary file: {}", e);
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
        return format!("{:?}\n", errors);
    }

    // These constants are taken from the C++ sources of the client.
    const BEAM_WIDTH :u16 = 500;
    const LM_WEIGHT :f32 = 0.75;
    const VALID_WORD_COUNT_WEIGHT :f32 = 1.85;
    // The model has been trained on this specific sample rate.
    const SAMPLE_RATE :u32 = 16_000;

    let model_dir_str = "/home/jandrews/devel/speech/DeepSpeech-0.6.0/models/";
    let dir_path = Path::new(&model_dir_str);
    let mut m = Model::load_from_files(
        &dir_path.join("output_graph.pb"),
        BEAM_WIDTH).unwrap();
        m.enable_decoder_with_lm(
        &dir_path.join("lm.binary"),
        &dir_path.join("trie"),
        LM_WEIGHT,
        VALID_WORD_COUNT_WEIGHT);

    // Obtain the buffer of samples
    let audio_buf :Vec<_> = if desc.sample_rate() == SAMPLE_RATE {
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

    // Run the speech to text algorithm
    // @TODO handle errors
    let message = m.speech_to_text(&audio_buf).unwrap();

    // @TODO make saving a copy of the audio file optional
    let now: DateTime<Utc> = Utc::now();
    let archive_directory = format!("archive/{}/{}/{}/", now.format("%Y"), now.format("%m"), now.format("%d"));
    match std::fs::create_dir_all(&archive_directory) {
        Ok(_) =>  {
            let hour = now.format("%H");
            let minute = now.format("%M");
            let second = now.format("%S");
            let mut buffer = match std::fs::File::create(format!("{}/audio-{}-{}-{}.wav", archive_directory, &hour, &minute, &second)) {
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
            match writeln!(buffer, "{}", &message) {
                Ok(_) => (),
                Err(e) => eprintln!("failed to archive text conversion of audio file: {}", e),
            }
        }
        Err(e) => {
            eprintln!("failed to create directory: '{}', {}", &archive_directory, e);
        }
    }

    // Debug output for now:
    println!("{}", &message);
    // Return text
    format!("{}\n", message)
}

fn main() {
    let opt = Opt::from_args();
    let opt_clone = opt.clone();

    let server = HttpServer::new(move || {
        App::new().service(
            web::resource("/convert/audio/text").data(
                String::configure(|cfg| {
                    // limit audio file size in bytes (defaults to 4MB)
                    cfg.limit(opt_clone.bytes)
                }))
                .route(web::post().to(audio_to_text)),
            )
    });
    server.bind(&opt.listen)
        .unwrap()
        .run()
        .unwrap();
}
