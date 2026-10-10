use bedrock::network::tokio::Client;
use bedrock::protocol::V2225;
use std::net::SocketAddr;
use std::time::Duration;

type Protocol = V2225;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:19132".to_string())
        .parse()
        .expect("address must look like 127.0.0.1:19132");
    let client = Client::offline("bedrock-rs").timeout(Duration::from_secs(30));

    let mut session = match client.connect::<Protocol>(addr).await {
        Ok(session) => session,
        Err(error) => {
            println!("Login failed: {error}");
            return;
        }
    };
    println!(
        "Logged in to {addr} with {} packets already queued",
        session.pending().len()
    );

    match session.connection_mut().recv().await {
        Ok(packets) => println!("{packets:#?}"),
        Err(error) => println!("Connection ended: {error}"),
    }
    session.connection().close().await;
}
