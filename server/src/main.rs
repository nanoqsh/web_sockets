mod chat;

use crate::chat::{ChatMessage, User};
use core::{decode, Message};
use futures::TryStreamExt;
use std::io;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, ToSocketAddrs},
    sync::broadcast::{self, Sender},
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
        tokio::select! {
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
                tokio::spawn(async move {
                    match process(stream, sender).await {
                        Ok(()) => println!("Connection at {addr} lost"),
                        Err(err) => eprintln!("Error at {addr}: {:?}", err),
                    }
                });
            }
        }
    }
}

async fn process<S>(stream: S, sender: Sender<ChatMessage>) -> ws::Result<()>
where
    S: AsyncRead + AsyncWrite,
{
    futures::pin_mut!(stream);
    let mut stream = accept_async(stream).await?;
    let mut user = User::new(sender);

    while let Some(message) = stream.try_next().await? {
        let message = match message {
            ws::Message::Text(text) => Message::Text(text),
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
            _ => unreachable!(),
        };

        user.got(message);
    }

    Ok(())
}
