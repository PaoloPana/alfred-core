# MQTT broker

Since 0.5 the modules do not talk to each other through the ZeroMQ `daemon`: they all connect to an
**MQTT 5 broker** ([Mosquitto](https://mosquitto.org/) is the reference one, but any MQTT 5 broker works).
Compared to the old PUB/SUB, the broker gives you:

- **Persistence**: a module that is offline (restarting, crashed) receives the messages published meanwhile, as soon as it reconnects.
- **Access control and encryption**: users and passwords, per-topic ACLs, TLS.
- **Interoperability**: the broker can be bridged to another broker, and tools such as Home Assistant speak the same protocol.

The broker is not started by `runner`: install it as a system service (or run it in a container) before starting Alfred.

## Install and start Mosquitto

```shell
sudo apt install mosquitto          # Debian / Ubuntu / Raspberry Pi OS
sudo cp mosquitto.conf /etc/mosquitto/conf.d/alfred.conf   # sample configuration of this repository
sudo systemctl restart mosquitto
```

The sample [`mosquitto.conf`](../mosquitto.conf) is a drop-in file: it makes the broker listen on `127.0.0.1:1883` only,
without authentication. The default configuration of Debian, Ubuntu and Raspberry Pi OS already enables persistence, which is what
lets the queued messages survive a restart of the broker; with a configuration of your own add `persistence true` and a
`persistence_location`. Mosquitto refuses duplicated options, so do not copy the options that your main file already sets.

With Docker:

```shell
docker run -d --name mosquitto -p 127.0.0.1:1883:1883 \
  -v "$PWD/mosquitto-docker.conf:/mosquitto/config/mosquitto.conf:ro" eclipse-mosquitto:2
```

where `mosquitto-docker.conf` contains `listener 1883`, `allow_anonymous true`, `persistence true` and
`persistence_location /mosquitto/data/` (the default configuration of the image only accepts connections from inside the
container; mount a volume on `/mosquitto/data` to keep the data).

## Configure Alfred

In the `[alfred]` section of `config.toml`; every key can be overridden by an environment variable.

| `config.toml`         | Environment variable         | Default                 | Description |
|-----------------------|------------------------------|-------------------------|-------------|
| `broker_url`          | `ALFRED_BROKER_URL`          | `mqtt://127.0.0.1:1883` | `mqtt://host[:port]` (port 1883) or `mqtts://host[:port]` (TLS, port 8883) |
| `username`            | `ALFRED_USERNAME`            | none                    | User name |
| `password`            | `ALFRED_PASSWORD`            | none                    | Password: prefer the environment variable to keeping it in `config.toml` |
| `ca_file`             | `ALFRED_CA_FILE`             | system certificates     | PEM file with the CA to trust (only for `mqtts://`) |
| `qos`                 | `ALFRED_QOS`                 | `1`                     | MQTT QoS of publications and subscriptions: 0 (at most once), 1 (at least once), 2 (exactly once) |
| `session_expiry_secs` | `ALFRED_SESSION_EXPIRY_SECS` | `3600`                  | How long the broker keeps the subscriptions and the queued messages of a disconnected module |
| `instance_id`         | `ALFRED_INSTANCE_ID`         | none                    | See [sessions](#sessions-and-presence) |

The `url`, `pub_port` and `sub_port` keys (and `ALFRED_URL`, `ALFRED_PUB_PORT`, `ALFRED_SUB_PORT`) do not exist anymore:
a configuration that still contains them stops with an error that explains how to migrate. Do not commit passwords:
keep `config.toml` free of secrets and set `ALFRED_PASSWORD` in the environment of the service (e.g. `Environment=` or
`EnvironmentFile=` in the systemd unit).

If the broker is not reachable, a module retries for 30 seconds before failing, so it can be started together with the broker.
A refused login or an untrusted TLS certificate fail immediately, with the reason in the error.

## Sessions and presence

- Every module connects with a **stable client id**: its name, followed by `-<instance_id>` if `instance_id` is set.
  Two connections with the same client id kick each other out, so set `instance_id` (e.g. the host name) when you run the
  same module more than once against one broker.
- The connection uses `clean_start = false` and a session expiry (`session_expiry_secs`): the broker keeps the subscriptions and
  queues the messages (QoS 1/2) for a module that is disconnected, and delivers them when a module with the same client id reconnects.
  A module that stays offline longer than the expiry loses what was queued.
- The subscriptions are part of the session: a module that stops listening to a topic keeps receiving it until the session expires.
- A module without a name (`Config::read(None)`) uses a throwaway client with no session.
- **Presence**: on connect a module publishes the retained message `online` on `module/<client id>/status`; the broker publishes
  `offline` (retained, last will) when the connection drops. `Connection::disconnect()` closes cleanly and publishes `offline` itself.
  In Alfred these topics are `module.<client id>.status`, and listening to them (or to everything) returns a `Text` message with the status as payload.
- `module.info.request` / `module.info.response` work as before.

## Topics

Modules keep using dot separated topics (`event.telegram.new_message`); on the wire they are MQTT topics with `/`
(`event/telegram/new_message`). The conversion happens in the `alfred_core::topic` module only.

Subscriptions work on whole segments: `listen("event.telegram")` receives `event.telegram` and everything below it
(`event.telegram.new_message`), but **not** `event.telegramx`. Before 0.5 ZeroMQ matched any string prefix, so
`listen("console")` also received `consolex`: that does not work anymore. An empty topic (`listen("")`) receives everything.
Topics cannot contain `/`, `+`, `#`, empty segments or `\0`, and wildcards are not supported by `listen`.

Because of the wildcard `#`, `listen("a.b")` is a single MQTT subscription `a/b/#` (which also matches `a/b`), not two.

## Messages and duplicates

`Connection::send` gives every publication a new unique `id`. With QoS 1 the broker may deliver a message more than once;
`Connection::receive_all` (and therefore `receive`) remembers the last 1024 ids and drops the repeats.
A message that is not an Alfred message (e.g. published by another tool) is logged and skipped by `receive`.
The byte layout of a message is in [protocol.md](protocol.md).

## Authentication, ACL and TLS

```shell
sudo mosquitto_passwd -c /etc/mosquitto/alfred.passwd alfred     # then: mosquitto_passwd -b ... another-user password
```

```
# /etc/mosquitto/conf.d/alfred.conf
allow_anonymous false
password_file /etc/mosquitto/alfred.passwd
acl_file /etc/mosquitto/alfred.acl

listener 8883
cafile   /etc/mosquitto/certs/ca.crt
certfile /etc/mosquitto/certs/server.crt
keyfile  /etc/mosquitto/certs/server.key
```

```
# /etc/mosquitto/alfred.acl
user alfred
topic readwrite #

# a user that can only read the events of one module
user dashboard
topic read event/telegram/#
```

```toml
[alfred]
broker_url = "mqtts://broker.lan:8883"
username = "alfred"
ca_file = "/etc/alfred/ca.crt"
```

The server certificate must be signed by the CA of `ca_file` and contain the host of `broker_url` among its subject alternative names.
Remember that a module needs, besides its own topics, to read `module/info/request`, to write `module/info/response` and to write its `module/<client id>/status`.

Limitations of Mosquitto's `acl_file` to know about: a subscription to a topic the user cannot read is *granted* but never
receives anything, and a publication that is not allowed is dropped by the broker without an error for the sender
(the client only logs a warning). Use the Mosquitto dynamic security plugin if you need the broker to refuse them.

## Troubleshooting

- `unable to connect to the broker: ... NotAuthorized`: wrong or missing `username`/`password`, or the user is not in the password file.
- `... UnknownIssuer`: set `ca_file` to the CA that signed the broker certificate.
- Messages published while a module was down are missing: check that the module always uses the same name/`instance_id`, that
  the session did not expire, and that the broker keeps sessions across restarts (`persistence true`).
- Run with `RUST_LOG=debug` to see the connection, subscriptions and every message.
