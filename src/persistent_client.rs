use crate::socket::try_get_connection;
use mpd_client::client::{CommandError, ConnectionEvent};
use mpd_client::responses::{SongInQueue, Status};
use mpd_client::{commands, Client};
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::spawn;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::sleep;
use tracing::error;

#[derive(Debug, Clone)]
enum State {
    Disconnected,
    Connected(Arc<Client>),
}

type ConnectionChannel = (
    UnboundedSender<Arc<Client>>,
    Arc<AsyncMutex<UnboundedReceiver<Arc<Client>>>>,
);

/// MPD client which automatically attempts to reconnect
/// if the connection cannot be established or is lost.
///
/// Commands sent to a disconnected client are queued.
#[derive(Debug)]
pub struct PersistentClient<'a> {
    host: &'a str,
    retry_interval: Duration,
    state: Arc<Mutex<State>>,
    channel: (
        UnboundedSender<ConnectionEvent>,
        UnboundedReceiver<ConnectionEvent>,
    ),
    connection_channel: ConnectionChannel,
}

impl<'a> PersistentClient<'a> {
    pub fn new(host: &'a str, retry_interval: Duration) -> Self {
        let channel = mpsc::unbounded_channel();
        let connection_channel = mpsc::unbounded_channel();

        Self {
            host,
            retry_interval,
            state: Arc::new(Mutex::new(State::Disconnected)),
            channel,
            connection_channel: (
                connection_channel.0,
                Arc::new(AsyncMutex::new(connection_channel.1)),
            ),
        }
    }

    /// Attempts to connect to the MPD host
    /// and begins listening to server events.
    pub fn init(&self) {
        let host = self.host.to_string();
        let retry_interval = self.retry_interval;
        let state = self.state.clone();
        let tx = self.channel.0.clone();
        let conn_tx = self.connection_channel.0.clone();

        spawn(async move {
            loop {
                let connection = try_get_connection(&host).await;

                match connection {
                    Ok(connection) => {
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

                            tx.send(event).expect("Failed to send event");
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
        self.host
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

        let rx = self.connection_channel.1.clone();
        let mut rx = rx.lock().await;

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
    pub async fn recv(&mut self) -> Option<ConnectionEvent> {
        let rx = &mut self.channel.1;
        rx.recv().await
    }

    /// Runs the `status` command on the MPD server.
    pub async fn status(&self) -> Result<Status, CommandError> {
        self.with_client(|client| async move { client.command(commands::Status).await })
            .await
    }

    /// Runs the `currentsong` command on the MPD server.
    pub async fn current_song(&self) -> Result<Option<SongInQueue>, CommandError> {
        self.with_client(|client| async move { client.command(commands::CurrentSong).await })
            .await
    }
}

/// Creates a new client on the default localhost TCP address
/// with a connection retry of 5 seconds.
impl<'a> Default for PersistentClient<'a> {
    fn default() -> Self {
        PersistentClient::new("localhost:6600", Duration::from_secs(5))
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
