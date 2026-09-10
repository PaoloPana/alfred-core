# Changelog

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
