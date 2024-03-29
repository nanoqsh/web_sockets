mod socket;

use crate::socket::SOCK;
use core::Message;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlInputElement};
use yew::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    let app = document()
        .get_element_by_id("app")
        .expect("document should have a `app` container");

    yew::start_app_in_element::<App>(app);
}

fn document() -> Document {
    web_sys::window()
        .expect("no global `window` exists")
        .document()
        .expect("should have a document on window")
}

#[derive(Clone, PartialEq)]
struct User {
    name: Option<Rc<str>>,
}

#[function_component(App)]
fn app() -> Html {
    let user = use_state(|| User { name: None });
    let onauth = Callback::from({
        let user = user.clone();
        move |name| user.set(User { name: Some(name) })
    });

    html! {{
        match &*user {
            User { name: Some(name) } => html! {
                <Chat name={ name.clone() } />
            },
            _ => html! {
                <Auth { onauth } />
            }
        }
    }}
}

#[derive(Clone, PartialEq, Properties)]
struct ChatProps {
    name: Rc<str>,
}

#[function_component(Chat)]
fn chat(_: &ChatProps) -> Html {
    let update = use_state(|| ());
    let messages = use_mut_ref(Vec::new);

    SOCK.with(|sock| {
        let messages = messages.clone();
        sock.on_message(Box::new(move |message| {
            if let Message::Text { from, text } = message {
                messages.borrow_mut().push((from, text));
                update.set(());
            }
        }));
    });

    let text_node = NodeRef::default();
    let onclick = Callback::from({
        let text_node = text_node.clone();
        move |_| {
            if let Some(input) = text_node.cast::<HtmlInputElement>() {
                let text = input.value();
                input.set_value("");
                let text = text.trim();

                if text.is_empty() {
                    return;
                }

                let message = Message::Text {
                    from: String::new(),
                    text: text.into(),
                };
                SOCK.with(|sock| sock.send(&message));
            }
        }
    });

    html! {
        <div>
            {
                for messages.borrow().iter().enumerate().map(|(key, (from, text))| html! {
                    <ChatMessage
                        { key }
                        from={ from.to_string() }
                        text={ text.to_string() }
                    />
                })
            }
            <div id="send_form">
                <input ref={ text_node } type="text" name="text" />
                <button { onclick }>{ "Send" }</button>
            </div>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
struct MessageProps {
    from: String,
    text: String,
}

#[function_component(ChatMessage)]
fn chat_message(MessageProps { from, text }: &MessageProps) -> Html {
    html! {
        <div class="chat_message">
            <strong>{ from }</strong>
            { ":" }
            <p>{ text }</p>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
struct AuthProps {
    onauth: Callback<Rc<str>>,
}

#[function_component(Auth)]
fn auth(props: &AuthProps) -> Html {
    // Initialize the socket
    SOCK.with(|_| {});

    let text_node = NodeRef::default();
    let onauth = props.onauth.clone();
    let onclick = Callback::from({
        let text_node = text_node.clone();
        move |_| {
            if let Some(input) = text_node.cast::<HtmlInputElement>() {
                let name = input.value();
                let name = name.trim();

                if name.is_empty() {
                    return;
                }

                let message = Message::Auth { name: name.into() };
                SOCK.with(|sock| sock.send(&message));
                onauth.emit(name.into());
            }
        }
    });

    html! {
        <div id="auth_form">
            <input ref={ text_node } type="text" name="name" />
            <button { onclick }>{ "Auth" }</button>
        </div>
    }
}
