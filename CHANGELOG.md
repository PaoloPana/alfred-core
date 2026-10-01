# Changelog

## [Unreleased] - yyyy-mm-dd

## [0.3.0] - 2026-10-01

### Added
- Added `PROTOCOL_VERSION` (`3`), sent as the first byte of every message header: `Message::decode` rejects any other version with `MessageEncodingError::ProtocolVersion`. Messages sent by older alfred-core versions are always rejected: they have no payload frame, and apart from `Photo` their first byte is not `3`
- Added `Message::text`, which returns the payload as `&str` or `Error::PayloadNotText`, and `Message::payload_description`, which returns the text or `<N bytes>` for binary payloads
- Re-exported `bytes`

### Modified
- **Breaking:** messages are now binary. Every message is sent as three ZeroMQ frames: topic, header (protocol version followed by the metadata encoded in MessagePack with named fields) and payload (raw bytes). Modules built on alfred-core 0.2.x or older cannot exchange messages with this version
- **Breaking:** replaced `Message::text: String` with `Message::payload: Bytes`, so a message can carry any binary content. `Audio` and `Photo` messages still carry a file path as text
- **Breaking:** replaced `Message::compress`/`decompress` with `Message::encode`/`decode`, and `MessageType::compress`/`decompress` with `MessageType::encode`/`decode` (`u8` instead of `char`); removed `impl Display for Message`
- **Breaking:** `Message::reply` and `Message::reply_chunk` accept any `impl Into<Bytes>` as payload
- **Breaking:** renamed `MessageCompressionError` to `MessageEncodingError` and `Error::MessageCompressionError` to `Error::MessageEncodingError`; `Error::PublishError` now contains only the topic
- **Breaking:** `MessageType` is now `#[non_exhaustive]`: a `match` on it needs a `_` arm (e.g. `MessageType::Photo | _`), so new message types can be added in minor releases without breaking modules
- Missing header fields are decoded with their default value and unknown header fields are ignored: new header fields can be added in minor releases, and the protocol version changes only for incompatible changes
- Params, response topics, sender and stream id can contain any char (including `\0`), and a message can have any number of params and response topics
- `Connection::receive` (and so `AlfredModule::receive`) now logs and discards messages that cannot be decoded instead of returning an error, so a module built on an older alfred-core cannot stop the other modules
- `logs` now shows a warning for messages that cannot be decoded instead of exiting, and shows binary payloads as `<N bytes>`
- Moved tests out of the source files: public API tests in `tests/`, tests of private functions in a `tests.rs` file next to their module

### Removed
- Removed `itertools` dependency

### Fixed
- Fixed memory leak in `Connection::send_event`, which leaked the topic string of every event sent
- Fixed `ModuleDetailsBuilder::build` setting the module name as version

## [0.2.1] - 2026-09-10

### Added
- Added `StreamText`, `StreamAudio` and `StreamPhoto` variants to `MessageType`, for messages sent as a stream of chunks
- Added `is_final`, `stream_id` and `sequence` fields to `Message` (all ignored for non-stream types): `is_final` marks the last chunk of a `Stream*` message, `stream_id` lets a receiver tell apart chunks of concurrently in-flight streams, and `sequence` is a chunk's 0-based position within its stream. The new `Message::reply_chunk` sets all three explicitly; `Message::reply` delegates to it with `is_final: true` (a plain reply is a complete, one-chunk message)
- Added `AlfredModule::send_stream(topic, message)` and `AlfredModule::send_event_stream(publisher_name, event_name, message)`, which send one stream chunk and fill in `message.stream_id` when it's empty, returning the id used so the caller can pass it into the next chunk

### Fixed
- Fixed `Message::decompress` panicking/corrupting data for message types encoded above `0x7F` (e.g. `ModuleInfo`, and now the `Stream*` types), which byte-sliced the compressed string instead of slicing by char

## [0.2.0] - 2026-09-06

### Modified
- Improved message compression
- Renamed project from alfred-rs to alfred-core
- Improved install script
- Improved documentation

### Updated
- Updated itertools requirement from 0.13 to 0.14

## [0.1.9] - 2025-01-03

### Added
- Add service and default config in [installation script](scripts/install-alfred.sh)
- Removed default sudo privileges from installation script command in README.md
- Managed empty cron configuration file

### Modified
- Updated CI/CD

## [0.1.8] - 2024-12-30

### Modified
- Updated CI/CD

## [0.1.7] - 2024-12-30

### Modified
- Updated cron version (to v0.14)
- Updated CI/CD

## [0.1.6] - 2024-12-30

## [0.1.5] - 2024-12-30

### Modified
- Updated cron version (to v0.14)
- Updated CI/CD

## [0.1.3] - 2024-12-30

### Modified
- CI/CD settings

## [0.1.2] - 2024-12-30

### Modified
- CI/CD properties

## [0.1.1] - 2024-12-30

### Added

- alfred-core library for managing modules interactions
- daemon bin for creating the environment
- cron bin for scheduling messages
- routing bin for redirecting a message from a topic to another
- logs bin for a simple inspection of the exchanged messages
- downloader bin for managing the download of a remote module
- runner bin for running a single module or the configured modules
