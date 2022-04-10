use core::Message;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct ChatMessage {
    pub from: String,
    pub text: String,
}

pub struct User {
    name: Option<String>,
    sender: Sender<ChatMessage>,
}

impl User {
    pub fn new(sender: Sender<ChatMessage>) -> Self {
        Self { name: None, sender }
    }

    pub fn got(&mut self, message: Message) {
        match message {
            Message::Auth { name } => match &self.name {
                Some(name) => println!("My name is {name} already."),
                None => self.name = Some(name),
            },
            Message::Text { text, .. } => match &self.name {
                Some(name) => {
                    self.sender
                        .send(ChatMessage {
                            from: name.clone(),
                            text,
                        })
                        .ok()
                        .unwrap();
                }
                None => (),
            },
        }
    }
}
