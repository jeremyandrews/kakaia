# Kakaia

Kakaia strives to be a personal assistant, implemented primarily in Rust.

## Usage

Still a work in progress:

1. launch server: `cargo run`
1. POST: 
    ```
    curl --request POST --data @test/test.base64 http://127.0.0.1:8088/convert/audio/text
    audio desc: 'Description { format: Wav, channel_count: 1, sample_rate: 16000 }'
    ```

Next steps:

- optionally store permanent copy of validated audio file
- invoke DeepSpeech to convert to text
- return text version of audio file

## Notes

Uses (audrey)[https://github.com/RustAudio/audrey] to support the following audio file types:

- FLAC (`.flac`)
- Ogg Vorbis (`.ogg`)
- WAV (`.wav`)
- ALAC within CAF (`.caf`)

## Roadmap

### Step 1: Proof of Concept

Kakaia engine:

- accept `wav` (or other supported audio) files through API endpoint
- invoke DeepSpeech to convert the `wav` to text
- return text version of `wav` via API

Kakaia iOS/watchOS app:

- provide a simplistic UI for recording voice
- send audio recording to API
- wait for conversion
- display text version on app screen

### Step 2: Basic functionality

Kakaia engine:

- support a single useful command
- "set timer for n seconds/minutes/hours"
- match the phrase, and return a command via the API

Kakaia iOS/watchOS app:

- receive timer command and set a timer

### Future functionality

- get weather information
- set reminders
- create/update lists
- control smart devices
- dictate emails and other lengthy texts
- punctuation (either spoken "comma", or trying to use metadata to identify pauses etc)
- identify speaker (if recording a meeting, identify the different speakers)
- push notifications from engine to app