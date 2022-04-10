mod chat;

use crate::chat::{ChatMessage, User};
use core::{decode, encode, Message};
use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use std::io;
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    select,
    sync::broadcast::{self, Receiver},
};
use tokio_tungstenite::{accept_async, tungstenite as ws};

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

    let (sender, mut receiver) = broadcast::channel(16);

    loop {
        select! {
            result = receiver.recv() => {
                match result {
                    Ok(ChatMessage { from, text }) => println!("{from}: {text}"),
                    Err(_) => break Ok(()),
                }
            }
            result = listener.accept() => {
                let (stream, addr) = result?;
                println!("New request at {addr}");

                let sender = sender.clone();
                let receiver = sender.subscribe();
                tokio::spawn(async move {
                    if let Ok(stream) = accept_async(stream).await {
                        let (inp, out) = stream.split();

                        select! {
                            result = process(out, User::new(sender)) => {
                                match result {
                                    Ok(()) => println!("Connection at {addr} lost"),
                                    Err(err) => eprintln!("Error at {addr}: {:?}", err),
                                }
                            }
                            _ = resend(inp, receiver) => {}
                        }
                    }
                });
            }
        }
    }
}

async fn resend<S>(stream: S, mut receiver: Receiver<ChatMessage>)
where
    S: Sink<ws::Message, Error = ws::Error>,
{
    futures::pin_mut!(stream);
    while let Ok(ChatMessage { from, text }) = receiver.recv().await {
        let mut buf = Vec::new();
        encode(&Message::Text { from, text }, &mut buf).unwrap();
        stream.send(ws::Message::Binary(buf)).await.unwrap();
    }
}

async fn process<S>(stream: S, mut user: User) -> ws::Result<()>
where
    S: Stream<Item = ws::Result<ws::Message>>,
{
    futures::pin_mut!(stream);
    while let Some(message) = stream.try_next().await? {
        let message = match message {
            ws::Message::Text(text) => Message::Text {
                text,
                from: String::new(),
            },
            ws::Message::Binary(buf) => match decode(&buf) {
                Ok(message) => message,
                Err(_) => break,
            },
            ws::Message::Ping(_) => {
                println!("ping");
                continue;
            }
            ws::Message::Pong(_) => {
                println!("pong");
                continue;
            }
            ws::Message::Close(_) => {
                println!("close");
                break;
            }
            ws::Message::Frame(_) => unreachable!(),
        };

        user.got(message);
    }

    Ok(())
}
