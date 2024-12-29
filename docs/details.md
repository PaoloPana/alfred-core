# What is Alfred?
Alfred is a free framework able to manage multi-channel user interactions with AI and other services.
The main idea behind Alfred is to allow users to add features easily, like installing applications on a device.

The core of the framwork is written in Rust, but each applications (named "modules") can be written in different languages.

Each module does a specific job: there is a module which receives and sends messages using Telegram, there is a module which interact with OpenAI, another module interacts with HomeAssistant and so on. The modules exchange messages using a PUB/SUB architecture (implemented with [ZeroMQ](https://zeromq.org/)). This system allows to be technology-independent: for example, you can use the SpeechToText by OpenAI or choosing the one from Google, running the proper module without changing anything.

# Core Structure

## Library
### message
### module

## Daemon
## Cron
## Downloader
## Logs
## Routing
## Runner

## Config files
### config.toml
### cron.toml
### repositories.toml
### routing.toml
