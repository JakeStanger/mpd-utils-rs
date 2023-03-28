use crate::error::{Error, Result};
use crate::persistent_client::PersistentClient;
use mpd_client::client::{CommandError, ConnectionEvent};
use mpd_client::responses::{PlayState, SongInQueue, Status};
use mpd_client::Client;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

pub struct MultiHostClient<'a> {
    clients: Vec<PersistentClient<'a>>,
}

impl<'a> MultiHostClient<'a> {
    pub fn new(hosts: &'a [&'a str], retry_interval: Duration) -> Self {
        let hosts = hosts
            .iter()
            .map(|&host| PersistentClient::new(host, retry_interval))
            .collect();

        Self { clients: hosts }
    }

    /// Initialises each of the clients.
    pub fn init(&self) {
        for client in &self.clients {
            client.init();
        }
    }

    /// Waits until any of the clients
    /// make a valid connection to their host.
    pub async fn wait_for_any_client(&self) -> Arc<Client> {
        let waits = self
            .clients
            .iter()
            .map(|client| Box::pin(client.wait_for_client()));
        futures::future::select_all(waits).await.0
    }

    /// Waits until all of the clients
    /// make a valid connection to their host.
    pub async fn wait_for_all_clients(&self) -> Vec<Arc<Client>> {
        let waits = self.clients.iter().map(|client| client.wait_for_client());
        futures::future::join_all(waits).await
    }

    /// Attempts to find the current most relevant client.
    /// This checks for, in order:
    ///
    /// - A currently playing client
    /// - A paused client (ie has items in the playlist)
    /// - A connected client
    async fn get_current_client(
        &self,
    ) -> std::result::Result<Option<&PersistentClient>, CommandError> {
        self.wait_for_any_client().await;

        let connected_clients = self
            .clients
            .iter()
            .filter(|client| client.is_connected())
            .collect::<Vec<_>>();

        if connected_clients.is_empty() {
            Ok(None)
        } else {
            let player_states = connected_clients.iter().map(|&client| async move {
                client.status().await.map(|status| (client, status.state))
            });

            let player_states = futures::future::join_all(player_states)
                .await
                .into_iter()
                .collect::<std::result::Result<Vec<_>, _>>();

            player_states.map(|player_states| {
                player_states
                    .iter()
                    .find(|(_, state)| state == &PlayState::Playing)
                    .or_else(|| {
                        player_states
                            .iter()
                            .find(|(_, state)| state == &PlayState::Paused)
                    })
                    .or_else(|| {
                        player_states
                            .iter()
                            .find(|(_, state)| state == &PlayState::Stopped)
                    })
                    .map(|(client, _)| *client)
            })
        }
    }

    /// Runs the provided callback as soon as a connected client is available,
    /// using the most relevant client (see `get_current_client`).
    pub async fn with_client<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(Arc<Client>) -> Fut,
        Fut: Future<Output = T>,
    {
        let client = self.get_current_client().await;

        match client {
            Ok(Some(client)) => Ok(client.with_client(f).await),
            Ok(None) => Err(Error::NoHostConnectedError),
            Err(err) => Err(Error::CommandError(err)),
        }
    }

    pub async fn recv(&mut self) -> Option<ConnectionEvent> {
        let waits = self.clients.iter().map(|client| Box::pin(client.recv()));
        futures::future::select_all(waits).await.0
    }

    /// Runs the `status` command on the MPD server.
    pub async fn status(&self) -> Result<Status> {
        let client = self.get_current_client().await;
        match client {
            Ok(Some(client)) => client.status().await.map_err(Error::CommandError),
            Ok(None) => Err(Error::NoHostConnectedError),
            Err(err) => Err(Error::CommandError(err)),
        }
    }

    /// Runs the `currentsong` command on the MPD server.
    pub async fn current_song(&self) -> Result<Option<SongInQueue>> {
        match self.get_current_client().await {
            Ok(Some(client)) => client.current_song().await.map_err(Error::CommandError),
            Ok(None) => Err(Error::NoHostConnectedError),
            Err(err) => Err(Error::CommandError(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        let client =
            MultiHostClient::new(&["localhost:6600", "chloe:6600"], Duration::from_secs(5));

        client.init();
        client.wait_for_all_clients().await;

        let current_client = client.get_current_client().await;
        println!("{current_client:?}");
    }
}
