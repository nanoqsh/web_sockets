use core::{decode, Message};
use futures::{Stream, StreamExt, TryStreamExt};
use std::io;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_tungstenite::{accept_async, tungstenite as ws};

#[derive(Debug)]
struct User {
    name: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    run("127.0.0.1:6789").await
}

async fn run<A>(addr: A) -> io::Result<()>
where
    A: ToSocketAddrs,
{
    let listener = TcpListener::bind(addr).await?;
    println!(
        "Bound at: {}. Waiting for clients..",
        listener.local_addr().unwrap()
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New request at {addr}");

        let mut ws_stream = match accept_async(stream).await {
            Ok(ws_stream) => ws_stream,
            Err(err) => {
                println!("WebSocket handshake failed at {addr} with error: {err}");
                continue;
            }
        };

        tokio::spawn(async move {
            match process(&mut ws_stream).await {
                Ok(()) => println!("Connection at {addr} lost"),
                Err(err) => eprintln!("Error at {addr}: {:?}", err),
            }
        });
    }
}

async fn process<S>(stream: &mut S) -> ws::Result<()>
where
    S: Stream<Item = ws::Result<ws::Message>> + Unpin,
{
    let stream = stream.try_filter_map(|message| async {
        match message {
            ws::Message::Text(text) => Ok(Some(Message::Text(text))),
            ws::Message::Binary(buf) => Ok(Some(decode(&buf).unwrap())),
            ws::Message::Ping(_) => {
                println!("ping");
                Ok(None)
            }
            ws::Message::Pong(_) => {
                println!("pong");
                Ok(None)
            }
            ws::Message::Close(_) => {
                print!("Closed. ");
                Err(ws::Error::AlreadyClosed)
            }
            _ => unreachable!(),
        }
    });

    futures::pin_mut!(stream);
    process_messages(stream).await
}

async fn process_messages<S>(mut stream: S) -> ws::Result<()>
where
    S: Stream<Item = ws::Result<Message>> + Unpin,
{
    while let Some(item) = stream.next().await {
        match item? {
            Message::Auth { name } => println!("Auth as {name}"),
            Message::Text(text) => println!("USER: {text}"),
        }
    }

    Ok(())
}
