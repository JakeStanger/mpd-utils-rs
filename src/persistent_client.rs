use crate::socket::try_get_connection;
use mpd_client::client::{CommandError, ConnectionEvent};
use mpd_client::commands::Command;
use mpd_client::responses::{SongInQueue, Status};
use mpd_client::{commands, Client};
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::spawn;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::sleep;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
enum State {
    Disconnected,
    Connected(Arc<Client>),
}

type Channel<T> = (broadcast::Sender<T>, broadcast::Receiver<T>);

/// MPD client which automatically attempts to reconnect
/// if the connection cannot be established or is lost.
///
/// Commands sent to a disconnected client are queued.
#[derive(Debug)]
pub struct PersistentClient {
    host: String,
    retry_interval: Duration,
    state: Arc<Mutex<State>>,
    channel: Channel<Arc<ConnectionEvent>>,
    connection_channel: Channel<Arc<Client>>,
}

impl PersistentClient {
    pub fn new(host: String, retry_interval: Duration) -> Self {
        let channel = broadcast::channel(32);
        let connection_channel = broadcast::channel(8);

        Self {
            host,
            retry_interval,
            state: Arc::new(Mutex::new(State::Disconnected)),
            channel,
            connection_channel,
        }
    }

    /// Attempts to connect to the MPD host
    /// and begins listening to server events.
    pub fn init(&self) {
        let host = self.host.clone();
        let retry_interval = self.retry_interval;
        let state = self.state.clone();
        let tx = self.channel.0.clone();
        let conn_tx = self.connection_channel.0.clone();

        spawn(async move {
            loop {
                let connection = try_get_connection(&host).await;

                match connection {
                    Ok(connection) => {
                        info!("Connected to '{host}'");

                        let client = Arc::new(connection.0);

                        {
                            *state.lock().expect("Failed to get lock on state") =
                                State::Connected(client.clone());
                            conn_tx.send(client).expect("Failed to send event");
                        }

                        let mut events = connection.1;

                        while let Some(event) = events.next().await {
                            if let ConnectionEvent::ConnectionClosed(err) = event {
                                error!("Lost connection to '{host}': {err:?}");
                                *state.lock().expect("Failed to get lock on state") =
                                    State::Disconnected;

                                break;
                            }

                            debug!("Sending event: {event:?}");

                            // Wrap in `Arc` because `ConnectionEvent` isn't `Clone`.
                            tx.send(Arc::new(event)).expect("Failed to send event");
                        }
                    }
                    Err(err) => {
                        error!("Failed to connect to '{host}': {err:?}");
                        *state.lock().expect("Failed to get lock on state") = State::Disconnected;
                    }
                }

                sleep(retry_interval).await;
            }
        });
    }

    /// Gets the client host address or path
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Gets whether there is a valid connection to the server
    pub fn is_connected(&self) -> bool {
        matches!(
            *self.state.lock().expect("Failed to get lock on state"),
            State::Connected(_)
        )
    }

    /// Waits for a valid connection to the server to be established.
    /// If already connected, resolves immediately.
    pub async fn wait_for_client(&self) -> Arc<Client> {
        {
            let state = self.state.lock().expect("Failed to get lock on state");

            if let State::Connected(client) = &*state {
                return client.clone();
            }
        }

        let mut rx = self.connection_channel.0.subscribe();
        rx.recv().await.unwrap()
    }

    /// Runs the provided callback as soon as the connected client is available.
    pub async fn with_client<F, Fut, T>(&self, f: F) -> T
    where
        F: FnOnce(Arc<Client>) -> Fut,
        Fut: Future<Output = T>,
    {
        let client = self.wait_for_client().await;
        f(client).await
    }

    /// Receives an event from the MPD server.
    pub async fn recv(&self) -> Result<Arc<ConnectionEvent>, RecvError> {
        let mut rx = self.channel.0.subscribe();
        rx.recv().await
    }

    /// Creates a new receiver to be able to receive events
    /// outside of the context of `&self`.
    ///
    /// When you have access to the client instance, prefer` recv()` instead.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<ConnectionEvent>> {
        self.channel.0.subscribe()
    }

    /// Runs the provided command on the MPD server.
    ///
    /// Waits for a valid connection and response before the future is completed.
    pub async fn command<C: Command>(&self, cmd: C) -> Result<C::Response, CommandError> {
        self.with_client(|client| async move { client.command(cmd).await })
            .await
    }

    /// Runs the `status` command on the MPD server.
    ///
    /// Waits for a valid connection and response before the future is completed.
    pub async fn status(&self) -> Result<Status, CommandError> {
        self.command(commands::Status).await
    }

    /// Runs the `currentsong` command on the MPD server.
    ///
    /// Waits for a valid connection and response before the future is completed.
    pub async fn current_song(&self) -> Result<Option<SongInQueue>, CommandError> {
        self.command(commands::CurrentSong).await
    }
}

/// Creates a new client on the default localhost TCP address
/// with a connection retry of 5 seconds.
impl Default for PersistentClient {
    fn default() -> Self {
        PersistentClient::new("localhost:6600".to_string(), Duration::from_secs(5))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use mpd_client::commands;

    #[tokio::test]
    async fn test() {
        let client = PersistentClient::default();
        client.init();

        let status = client
            .with_client(|client| async move { client.command(commands::Status).await })
            .await
            .unwrap();

        println!("{:?}", status);
    }
}
