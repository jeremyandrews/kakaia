# Kakaia

Kakaia strives to be a personal assistant supporting voice commands without sending any data to third-parties.

## Usage

Build Kakaia as follows:

1. Download the appropriate native_client from https://github.com/mozilla/DeepSpeech/releases/tag/v0.6.0 and extract locally
1. export `LD_LIBRARY_PATH` and `LIBRARY_PATH` both pointing to the files extracted in the previous step. For example:

    ```
    export LD_LIBRARY_PATH=/opt/deepspeech/native_client/
    export LIBRARY_PATH=/opt/deepspeech/native_client/
    ```

1. Download the [0.6.0 models](https://github.com/mozilla/DeepSpeech/releases/download/v0.6.0/deepspeech-0.6.0-models.tar.gz) from https://github.com/mozilla/DeepSpeech/releases/tag/v0.6.0 and extract locally
1. export `DEEPSPEECH_MODELS` pointing to the files extracted in the previous step. (By default it will look for `models/` in the current working directory.) For example:

    ```
    export DEEPSPEECH_MODELS=/opt/deepspeech/models/
    ```

1. Build the Kakaia engine: `cargo build --release`

Run the Kakaia engine using all defaults as follows:

    ./targets/release/kakaia

Learn about available options by passing in the `-h` parameter.

    ./targets/release/kakaia -h

For example, to save a copy of all audio files and text conversions, pass in the `-s` parameter:

    ./targets/release/kakaia -s

Or, to listen on a different port, you could pass in the following:

    ./targets/release/kakaia -s --listen 0.0.0.0:8089

You can use `curl` to test the Kakaia engine without a client as follows:

```
curl --request POST --data @test/test.base64 http://127.0.0.1:8088/convert/audio/text
test
```

### Kakaia client

Currently there is only one Kakaia client, it runs on Apple's watchOS:

- https://github.com/jeremyandrews/kakaia-watchos/

The client provides a simplistic interface to record audio on the watch, which is then base64 encoded and pushed to the Kakaia engine via an Actix API. The engine decodes the file and uses DeepSpeech to convert the audio to text, returning the text on success.

### Notes

The Kakaia engine is written in Rust, using the [Rust bindings for the deepspeech library](https://github.com/RustAudio/deepspeech-rs) for speech-to-text conversions, and [Actix](https://actix.rs/) for the API.

The model included with DeepSpeech 0.6.0 was mostly trained with American English data and does not work will with other accents. https://hacks.mozilla.org/2019/12/deepspeech-0-6-mozillas-speech-to-text-engine/

Deepspeech-rs currently requires that audio be recorded with a single mono-track, at 16,000 Hz. It uses [audrey](https://github.com/RustAudio/audrey) to support the following audio file types:

- FLAC (`.flac`)
- Ogg Vorbis (`.ogg`)
- WAV (`.wav`)
- ALAC within CAF (`.caf`)

## Roadmap

### Phase 1: Proof of Concept

The Proof of Concept is functional, but IPs etc are hard-coded to my environment. The PoC is a success when it can be configured and run by multiple people without requiring code-level changes.

Kakaia engine:

- accept encoded audio files through API endpoint (done)
- invoke DeepSpeech to convert the audio file to text (done)
- return text version via API (done)
- configurable (done)

Kakaia watchOS app:

- provide a simplistic UI for recording voice (done)
- send audio recording to API (done)
- wait for conversion (done)
- display text version on app screen (done)
- add configuration (currently it's hard-coded to my environment)

### Phase 2: Basic functionality

Once the above is fully working, the next step will be to give Kakaia some very simplistic usefulness. This will require the following additions/changes:

Kakaia engine:

- engine return data via API as JSON
- match text against a single phrase: "set timer for n seconds/minutes/hours"
- on success, return a machine-readable command to client
- on failure, return a machine-readable error

Kakaia watchOS app:

- parse returned JSON, set timer when the command is received
- display errors if no command was matched
- submit to App Store

### Phase n: Future plans

Functionality:

- stream audio so conversion to text happens while user is speaking
- encrypt data sent between client(s) and engine so it can be sent across public networks without fear of eavesdropping
- training: allow training of words/phrases, explore how to improve audio to text conversion (and to add support for unrecognized words, such as "kakaia")
- add commmand for getting weather information
- add command for setting reminders
- add command for creating/updating todo lists
- add commands for controlling smart devices
- add support for dictating emails/lengthy texts
- add punctuation (either spoken words ie "comma", or automatically based on the metadata identifying pauses, etc)
- attempt to recognize different speakers and identify them in the text
- push notifications from engine to client(s)

Additional clients:

- iOS client
- Linux client
- MacOS client

Contributions welcome!