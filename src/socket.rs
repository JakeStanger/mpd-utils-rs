use mpd_client::client::Connection;
use mpd_client::protocol::MpdProtocolError;
use mpd_client::Client;
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;
use tokio::net::{TcpStream, UnixStream};

/// Cycles through each MPD host and
/// returns the first one which connects,
/// or none if there are none
pub(crate) async fn try_get_connection(host: &str) -> Result<Connection, MpdProtocolError> {
    if is_unix_socket(host) {
        connect_unix(host).await
    } else {
        connect_tcp(host).await
    }
}

fn is_unix_socket(host: &str) -> bool {
    let path = PathBuf::from(host);
    path.exists()
        && path
            .metadata()
            .map_or(false, |metadata| metadata.file_type().is_socket())
}

async fn connect_unix(host: &str) -> Result<Connection, MpdProtocolError> {
    let connection = UnixStream::connect(host).await?;
    Client::connect(connection).await
}

async fn connect_tcp(host: &str) -> Result<Connection, MpdProtocolError> {
    let connection = TcpStream::connect(host).await?;
    Client::connect(connection).await
}
