use std::path::Path;

use audrey::read::Reader;
use audrey::sample::interpolate::{Converter, Linear};
use audrey::sample::signal::{from_iter, Signal};
use deepspeech::Model;


// These constants are taken from the C++ sources of the client.
const BEAM_WIDTH :u16 = 500;
const LM_WEIGHT :f32 = 0.75;
const VALID_WORD_COUNT_WEIGHT :f32 = 1.85;
// The model has been trained on this specific sample rate.
const SAMPLE_RATE :u32 = 16_000;

pub fn convert_audio_to_text(audio_file: std::fs::File) -> (String, String) {
    // Read audio from temporary file.
    let mut reader = match Reader::new(&audio_file) {
        Ok(r) => r,
        Err(e) => {
            return (format!("failed to load audio file ({:?}): {}", audio_file, e), "unknown".to_string());
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
        return (format!("{:?}\n", errors), "unknown".to_string());
    }
    // @TODO: get this from ENV
    let model_dir_str = "/home/jandrews/devel/speech/DeepSpeech-0.6.0/models/";
    let dir_path = Path::new(&model_dir_str);
    // @TODO: gracefull handle errors
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

    let extension = match desc.format() {
        audrey::Format::Flac => "flac".to_string(),
        audrey::Format::OggVorbis => "ogg".to_string(),
        audrey::Format::Wav => "wav".to_string(),
        audrey::Format::CafAlac => "caf".to_string(),
    };

    // Run the speech to text algorithm
    // @TODO handle errors
    (m.speech_to_text(&audio_buf).unwrap(), extension)
}