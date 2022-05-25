pub mod pb {
    tonic::include_proto!("echo");
}

use std::time::Duration;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

use pb::{echo_client::EchoClient, EchoRequest};

async fn server_streaming_echo(client: &mut EchoClient<Channel>, msg: &str, num: usize) {
    let stream = client
        .server_streaming_echo(EchoRequest {
            message: msg.to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    // stream is infinite - take just 5 elements and then disconnect
    let mut stream = stream.take(num);
    while let Some(item) = stream.next().await {
        println!("\treceived: {}", item.unwrap().message);
    }
    // stream is droped here and the disconnect info is send to server
}

async fn worker1_run(mut client: EchoClient<Channel>) {
    println!("worker1: running");

    let stream = client
        .server_streaming_echo(EchoRequest {
            message: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    tokio::time::sleep(Duration::from_secs(5)).await;
    let mut stream = stream.take(10);
    while let Some(item) = stream.next().await {
        println!("\tworker1 received: {}", item.unwrap().message);
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
    // stream is droped here and the disconnect info is send to server
    tokio::time::sleep(Duration::from_secs(1)).await; //do not mess server println functions
    println!("worker1: bye");
}


async fn worker2_run(mut client: EchoClient<Channel>) {
    println!("worker2: running");
    let stream = client
        .server_streaming_echo(EchoRequest {
            message: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    let mut stream = stream.take(10);
    while let Some(item) = stream.next().await {
        println!("\tworker2 received: {}", item.unwrap().message);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    // stream is droped here and the disconnect info is send to server
    tokio::time::sleep(Duration::from_secs(1)).await; //do not mess server println functions
    println!("worker2: bye");
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://[::1]:50051")
        .initial_stream_window_size(16384).initial_connection_window_size(16384).connect().await.unwrap();
    let client = EchoClient::new(channel);

    let join1 = tokio::spawn(worker1_run(client.clone()));
    let join2 = tokio::spawn(worker2_run(client));

    join1.await.unwrap();
    join2.await.unwrap();

    Ok(())
}
