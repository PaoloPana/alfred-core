use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::sync::{Arc, Mutex as StdMutex, PoisonError};
use std::time::Duration;
use bytes::Bytes;
use log::{debug, error, info, warn};
use rumqttc::v5::mqttbytes::v5::{ConnectReturnCode, Filter, LastWill, LastWillProperties, PublishProperties, SubscribeReasonCode};
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions};
use rumqttc::{Outgoing, TlsConfiguration, Transport};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Instant};
use uuid::Uuid;
use crate::config::{AlfredConfig, Config};
use crate::error::Error;
use crate::message::{Message, MessageType};
use crate::topic::{from_mqtt_topic, parse_status_topic, status_topic, to_mqtt_filter, to_mqtt_topic};

const CONTENT_TYPE: &str = "application/vnd.alfred.message";
const STATUS_ONLINE: &str = "online";
const STATUS_OFFLINE: &str = "offline";
const KEEP_ALIVE: Duration = Duration::from_secs(30);
const REQUEST_CHANNEL_CAPACITY: usize = 256;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const SUBSCRIBE_TIMEOUT: Duration = Duration::from_secs(10);
const MIN_RECONNECT_DELAY: Duration = Duration::from_secs(1);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(10);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BrokerAddress {
    pub host: String,
    pub port: u16,
    pub tls: bool,
}

impl BrokerAddress {
    /// Parses `mqtt://host[:port]` and `mqtts://host[:port]` (IPv6 hosts in brackets).
    pub(crate) fn parse(url: &str) -> Result<Self, Error> {
        let invalid = |reason: &str| Error::InvalidConfig(format!("invalid broker_url `{url}`: {reason}"));
        let (tls, default_port, rest) = if let Some(rest) = url.strip_prefix("mqtt://") {
            (false, 1883, rest)
        } else if let Some(rest) = url.strip_prefix("mqtts://") {
            (true, 8883, rest)
        } else {
            return Err(invalid("the scheme must be mqtt:// or mqtts://"));
        };
        let authority = rest.trim_end_matches('/');
        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) if !port.contains(']') => {
                (host, port.parse::<u16>().map_err(|_| invalid("invalid port"))?)
            },
            _ => (authority, default_port),
        };
        let host = host.trim_start_matches('[').trim_end_matches(']');
        if host.is_empty() || host.contains(['/', '@', ' ']) {
            return Err(invalid("invalid host"));
        }
        Ok(Self { host: host.to_string(), port, tls })
    }
}

pub(crate) fn parse_qos(qos: u8) -> Result<QoS, Error> {
    match qos {
        0 => Ok(QoS::AtMostOnce),
        1 => Ok(QoS::AtLeastOnce),
        2 => Ok(QoS::ExactlyOnce),
        _ => Err(Error::InvalidConfig(format!("qos must be 0, 1 or 2, found {qos}")))
    }
}

/// MQTT client id of a module: the module name, followed by `-<instance id>` if set.
fn client_name(module_name: &str, instance_id: Option<&str>) -> Result<String, Error> {
    let name = instance_id.filter(|instance| !instance.is_empty())
        .map_or_else(|| module_name.to_string(), |instance| format!("{module_name}-{instance}"));
    if name.is_empty() || name.contains(['/', '+', '#', '\0']) {
        return Err(Error::InvalidConfig(format!("invalid module name or instance id `{name}`")));
    }
    Ok(name)
}

struct Settings {
    options: MqttOptions,
    qos: QoS,
    status_topic: Option<String>,
}

fn build_settings(config: &Config) -> Result<Settings, Error> {
    let alfred: &AlfredConfig = &config.alfred;
    let qos = parse_qos(alfred.qos)?;
    let address = BrokerAddress::parse(&alfred.broker_url)?;
    let (client_id, status_topic) = match config.get_module_name() {
        Some(module_name) => {
            let name = client_name(module_name, alfred.instance_id.as_deref())?;
            let status = status_topic(&name);
            (name, Some(status))
        },
        // without a module name there is no stable identity: use a throwaway client
        None => (format!("alfred-{}", Uuid::new_v4().simple()), None),
    };
    let persistent = status_topic.is_some();
    let mut options = MqttOptions::new(client_id, address.host.clone(), address.port);
    options.set_keep_alive(KEEP_ALIVE);
    options.set_clean_start(!persistent);
    options.set_session_expiry_interval(Some(if persistent { alfred.session_expiry_secs } else { 0 }));
    if let Some(username) = &alfred.username {
        options.set_credentials(username.clone(), alfred.password.clone().unwrap_or_default());
    }
    if address.tls {
        let transport = match &alfred.ca_file {
            Some(path) => {
                let ca = std::fs::read(path)
                    .map_err(|err| Error::InvalidConfig(format!("cannot read ca_file `{path}`: {err}")))?;
                Transport::tls_with_config(TlsConfiguration::Simple { ca, alpn: None, client_auth: None })
            },
            None => Transport::tls_with_default_config(),
        };
        options.set_transport(transport);
    } else {
        if alfred.ca_file.is_some() {
            return Err(Error::InvalidConfig(String::from("ca_file requires an mqtts:// broker_url")));
        }
        if alfred.username.is_some() && !matches!(address.host.as_str(), "127.0.0.1" | "localhost" | "::1") {
            warn!("Sending credentials to a remote broker without TLS: use an mqtts:// broker_url");
        }
    }
    if let Some(topic) = &status_topic {
        options.set_last_will(LastWill::new(
            topic.clone(),
            STATUS_OFFLINE,
            QoS::AtLeastOnce,
            true,
            Some(LastWillProperties {
                delay_interval: None,
                payload_format_indicator: None,
                message_expiry_interval: None,
                content_type: Some(String::from("text/plain")),
                response_topic: None,
                correlation_data: None,
                user_properties: Vec::new(),
            }),
        ));
    }
    Ok(Settings { options, qos, status_topic })
}

fn status_properties() -> PublishProperties {
    PublishProperties { content_type: Some(String::from("text/plain")), ..PublishProperties::default() }
}

type Received = (String, Bytes);

/// Outcome of a SUBSCRIBE: `true` if the broker granted every filter
type SubscribeAck = oneshot::Sender<bool>;

/// Topic filters of the connection and the SUBSCRIBE packets waiting for their SUBACK.
///
/// `waiting` has one entry per SUBSCRIBE request queued in the client, in order; the entry moves
/// to `inflight` (by packet id) when the event loop writes the packet, and is answered by the
/// SUBACK. Entries without sender belong to the automatic subscriptions after a (re)connection.
#[derive(Default)]
struct Subscriptions {
    filters: BTreeSet<String>,
    waiting: VecDeque<Option<SubscribeAck>>,
    inflight: HashMap<u16, Option<SubscribeAck>>,
}

type SharedSubscriptions = Arc<StdMutex<Subscriptions>>;

fn lock(subscriptions: &SharedSubscriptions) -> std::sync::MutexGuard<'_, Subscriptions> {
    subscriptions.lock().unwrap_or_else(PoisonError::into_inner)
}

struct EventLoopContext {
    client: AsyncClient,
    subscriptions: SharedSubscriptions,
    qos: QoS,
    status_topic: Option<String>,
    incoming: mpsc::UnboundedSender<Received>,
}

impl EventLoopContext {
    // the lock must cover the queueing of the request, so the waiters keep the order of the packets
    #[allow(clippy::significant_drop_tightening)]
    fn on_connected(&self) {
        {
            // SUBSCRIBE packets are not sent again by the client after a reconnection, and the
            // broker may have lost the session: subscribe again to everything (it is idempotent)
            let mut subscriptions = lock(&self.subscriptions);
            subscriptions.inflight.clear();
            if !subscriptions.filters.is_empty() {
                let filters = subscriptions.filters.iter().map(|filter| Filter::new(filter.clone(), self.qos)).collect::<Vec<_>>();
                subscriptions.waiting.push_back(None);
                if let Err(err) = self.client.try_subscribe_many(filters) {
                    subscriptions.waiting.pop_back();
                    error!("Unable to subscribe again after reconnecting: {err}");
                }
            }
        }
        if let Some(topic) = &self.status_topic {
            let result = self.client.try_publish_with_properties(topic.clone(), QoS::AtLeastOnce, true, STATUS_ONLINE, status_properties());
            if let Err(err) = result {
                warn!("Unable to publish the online status: {err}");
            }
        }
    }

    fn on_subscribe_sent(&self, packet_id: u16) {
        let mut subscriptions = lock(&self.subscriptions);
        if let Some(waiter) = subscriptions.waiting.pop_front() {
            subscriptions.inflight.insert(packet_id, waiter);
        }
    }

    fn on_subscribe_ack(&self, packet_id: u16, granted: bool) {
        let waiter = lock(&self.subscriptions).inflight.remove(&packet_id);
        if let Some(Some(waiter)) = waiter {
            // the receiver is gone if `listen` was cancelled
            let _ = waiter.send(granted);
        }
    }
}

async fn run_event_loop(mut event_loop: EventLoop, context: EventLoopContext, mut ready: Option<oneshot::Sender<Result<(), Error>>>) {
    let deadline = Instant::now() + CONNECT_TIMEOUT;
    let mut delay = MIN_RECONNECT_DELAY;
    loop {
        match event_loop.poll().await {
            Ok(Event::Incoming(Incoming::ConnAck(ack))) => {
                info!("Connected to the broker (session present: {})", ack.session_present);
                delay = MIN_RECONNECT_DELAY;
                context.on_connected();
                if let Some(ready) = ready.take() {
                    // the receiver is gone if Connection::new was cancelled: nothing to report
                    let _ = ready.send(Ok(()));
                }
            },
            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                let topic = String::from_utf8_lossy(&publish.topic).into_owned();
                if context.incoming.send((topic, publish.payload)).is_err() {
                    return;
                }
            },
            Ok(Event::Outgoing(Outgoing::Subscribe(packet_id))) => context.on_subscribe_sent(packet_id),
            Ok(Event::Incoming(Incoming::SubAck(ack))) => {
                let granted = ack.return_codes.iter().all(|code| matches!(code, SubscribeReasonCode::Success(_)));
                context.on_subscribe_ack(ack.pkid, granted);
            },
            Ok(Event::Outgoing(Outgoing::Disconnect)) => {
                debug!("Disconnected from the broker");
                return;
            },
            Ok(_) => {},
            Err(err) => {
                // refused credentials and untrusted certificates are configuration errors: retrying is useless
                let fatal = matches!(err, ConnectionError::ConnectionRefused(code) if code != ConnectReturnCode::Success)
                    || matches!(err, ConnectionError::Tls(_));
                if ready.is_some() && (fatal || Instant::now() >= deadline) {
                    error!("Unable to connect to the broker: {err}");
                    if let Some(ready) = ready.take() {
                        let _ = ready.send(Err(Error::MqttError(format!("unable to connect to the broker: {err}"))));
                    }
                    return;
                }
                warn!("Connection to the broker lost: {err}. Retrying in {} s", delay.as_secs());
                // do not sleep past the deadline of the first connection
                sleep(if ready.is_some() { delay.min(deadline.saturating_duration_since(Instant::now())) } else { delay }).await;
                delay = (delay * 2).min(MAX_RECONNECT_DELAY);
            }
        }
    }
}

pub(crate) struct MqttTransport {
    client: AsyncClient,
    incoming: Mutex<mpsc::UnboundedReceiver<Received>>,
    subscriptions: SharedSubscriptions,
    qos: QoS,
    status_topic: Option<String>,
    task: StdMutex<Option<JoinHandle<()>>>,
}

impl MqttTransport {
    /// Connects to the broker, waiting for the first `ConnAck` (up to 30 s).
    pub(crate) async fn new(config: &Config) -> Result<Self, Error> {
        let Settings { options, qos, status_topic } = build_settings(config)?;
        debug!("Connecting to the broker as `{}`", options.client_id());
        let (client, event_loop) = AsyncClient::new(options, REQUEST_CHANNEL_CAPACITY);
        let (incoming_sender, incoming) = mpsc::unbounded_channel();
        let (ready_sender, ready) = oneshot::channel();
        let subscriptions = SharedSubscriptions::default();
        let context = EventLoopContext {
            client: client.clone(),
            subscriptions: subscriptions.clone(),
            qos,
            status_topic: status_topic.clone(),
            incoming: incoming_sender,
        };
        let task = tokio::spawn(run_event_loop(event_loop, context, Some(ready_sender)));
        // from here on `Drop` aborts the task, also if the connection fails
        let transport = Self {
            client,
            incoming: Mutex::new(incoming),
            subscriptions,
            qos,
            status_topic,
            task: StdMutex::new(Some(task)),
        };
        // the event loop reports its own timeout, this is only a safety net
        match timeout(CONNECT_TIMEOUT + DISCONNECT_TIMEOUT, ready).await {
            Ok(Ok(result)) => result?,
            Ok(Err(_)) | Err(_) => return Err(Error::ConnectionError),
        }
        Ok(transport)
    }

    /// Subscribes to `topic` (and its sub-topics), returning when the broker has confirmed it:
    /// messages published after this call returns are delivered.
    #[allow(clippy::significant_drop_tightening)]
    pub(crate) async fn listen(&self, topic: &str) -> Result<(), Error> {
        let filter = to_mqtt_filter(topic)?;
        debug!("Subscribing to topic {topic} ({filter})");
        let (ack_sender, ack) = oneshot::channel();
        {
            let mut subscriptions = lock(&self.subscriptions);
            if !subscriptions.filters.insert(filter.clone()) {
                return Ok(());
            }
            // the waiter and the request must be queued in the same order
            subscriptions.waiting.push_back(Some(ack_sender));
            if self.client.try_subscribe(filter.clone(), self.qos).is_err() {
                subscriptions.waiting.pop_back();
                subscriptions.filters.remove(&filter);
                return Err(Error::SubscribeError(topic.to_string()));
            }
        }
        match timeout(SUBSCRIBE_TIMEOUT, ack).await {
            Ok(Ok(true)) => Ok(()),
            Ok(Err(_)) => {
                // connection lost before the SUBACK: the subscription is repeated after reconnecting
                warn!("Subscription to {topic} not confirmed yet: it will be completed after reconnecting");
                Ok(())
            },
            Ok(Ok(false)) | Err(_) => {
                lock(&self.subscriptions).filters.remove(&filter);
                Err(Error::SubscribeError(topic.to_string()))
            }
        }
    }

    /// Returns the next message and its topic (dot separated). Presence messages
    /// (`module/<name>/status`) are plain text on the wire and are returned as `Text` messages.
    pub(crate) async fn receive(&self) -> Result<(String, Message), Error> {
        let (mqtt_topic, payload) = self.incoming.lock().await.recv().await.ok_or(Error::GetMessageError)?;
        let topic = from_mqtt_topic(&mqtt_topic);
        if let Some(name) = parse_status_topic(&mqtt_topic) {
            let message = Message {
                message_type: MessageType::Text,
                params: BTreeMap::from([(String::from("module"), name.to_string())]),
                payload,
                ..Message::default()
            };
            return Ok((topic, message));
        }
        let message = Message::decode(&payload)?;
        debug!("{topic}: {message:?}");
        Ok((topic, message))
    }

    /// Publishes `message` with a new unique id.
    pub(crate) async fn send(&self, topic: &str, message: &Message) -> Result<(), Error> {
        let mqtt_topic = to_mqtt_topic(topic)?;
        let payload = message.encode_with_id(&Uuid::new_v4().to_string())?;
        debug!("Publishing message to topic {topic}: {message:?}");
        let properties = PublishProperties { content_type: Some(CONTENT_TYPE.to_string()), payload_format_indicator: Some(0), ..PublishProperties::default() };
        self.client.publish_with_properties(mqtt_topic, self.qos, false, payload, properties).await
            .map_err(|_| Error::PublishError(topic.to_string()))
    }

    /// Publishes the offline status and closes the connection cleanly (the last will is not
    /// sent in this case, so the status is published explicitly).
    pub(crate) async fn disconnect(&self) -> Result<(), Error> {
        if let Some(topic) = &self.status_topic {
            self.client.publish_with_properties(topic.clone(), QoS::AtLeastOnce, true, STATUS_OFFLINE, status_properties()).await
                .map_err(|_| Error::PublishError(topic.clone()))?;
        }
        self.client.disconnect().await.map_err(|err| Error::MqttError(err.to_string()))?;
        let task = self.task.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(task) = task {
            if timeout(DISCONNECT_TIMEOUT, task).await.is_err() {
                warn!("Timeout waiting for the disconnection");
            }
        }
        Ok(())
    }
}

impl Drop for MqttTransport {
    /// Closes the socket without a DISCONNECT packet: the broker publishes the last will.
    fn drop(&mut self) {
        let task = self.task.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(task) = task {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests;
