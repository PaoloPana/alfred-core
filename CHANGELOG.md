# Changelog

## [Unreleased] - yyyy-mm-dd

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
