# Kakaia

Kakaia strives to be a personal assistant, implemented primarily in Rust.

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