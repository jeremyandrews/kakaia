# Kakaia

Kakaia strives to be a personal assistant, implemented primarily in Rust.

## Usage

Still a work in progress:

1. launch server: `cargo run`
1. POST: 
    ```
    curl --header "Content-Type: application/json" --request POST \
         --data '{"filename": "test.wav", "data": "aGVsbG8gd29ybGQ="}' \
         http://127.0.0.1:8088/convert/audio/text
    audio file: 'test.wav', bytes: '[104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]'
    ```

(For now we're passing in the base64 encoded string "hello world".)

Next steps:

- validate audio file and store
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