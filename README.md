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

    cargo run --release

Learn about available options by passing in the `-h` parameter.

    cargo run --release -- -h

For example, to save a copy of all audio files and text conversions, pass in the `-s` parameter:

    cargo run --release -- -s

Or, to listen on a different port, you could pass in the following:

    cargo run --release -- -s --listen 0.0.0.0:8089

### Manually testing

You can use `curl` to test the Kakaia engine without a client as follows:
```
$ curl --data @test/test.base64 http://127.0.0.1:8088/convert/audio/text
{"command":"none","human":"no command","raw":"test","result":0.0}
```

### Set timer
```
$ curl --data @test/set-my-timer.base64 http://127.0.0.1:8088/convert/audio/text
{"command":"setTimer","human":"set timer for 600 seconds","raw":"said my timer for ten minutes","result":600.0}
```

#### Simple math
```
$ curl --data @test/ten-plus-ten.base64 http://127.0.0.1:8088/convert/audio/text
{"command":"simpleCalculation","human":"10 plus 10 equals 20","raw":"what is ten plus ten","result":20.0}
```

#### Temperature conversion

```
$ curl --data @test/convert-temperature.base64 http://127.0.0.1:8088/convert/audio/text
{"command":"convertTemperature","human":"5 degrees celsius is 41 degrees fahrenheit","raw":"convert five degrees celsius to farnie","result":41.0}
```

### Kakaia client

Currently there is only one Kakaia client, it runs on Apple's watchOS:

- https://github.com/jeremyandrews/kakaia-watchos/

The client provides a simplistic interface to record audio on the watch, which is then base64 encoded and pushed to the Kakaia engine via an Actix API. The engine decodes the file and uses DeepSpeech to convert the audio to text, returning the text on success.

[Other clients](https://github.com/jeremyandrews/kakaia/issues?utf8=%E2%9C%93&q=is%3Aissue+label%3AClient+).

### Notes

The Kakaia engine is written in Rust, using the [Rust bindings for the deepspeech library](https://github.com/RustAudio/deepspeech-rs) for speech-to-text conversions, and [Actix](https://actix.rs/) for the API.

The model included with DeepSpeech 0.6.0 was mostly trained with American English data. https://hacks.mozilla.org/2019/12/deepspeech-0-6-mozillas-speech-to-text-engine/

Deepspeech-rs currently requires that audio be recorded with a single mono-track, at 16,000 Hz. It uses [audrey](https://github.com/RustAudio/audrey) to support the following audio file types:

- FLAC (`.flac`)
- Ogg Vorbis (`.ogg`)
- WAV (`.wav`)
- ALAC within CAF (`.caf`)

Contributions welcome!
