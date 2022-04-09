use core::{decode, encode, Message};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{BinaryType, Document, ErrorEvent, HtmlInputElement, MessageEvent, WebSocket};
use yew::{function_component, html, use_state, Callback, Properties};

type OnMessage = Box<dyn FnMut(Message)>;

struct Socket {
    ws: WebSocket,
    on_message: RefCell<Option<OnMessage>>,
}

impl Socket {
    fn new(url: &str) -> Result<Self, JsValue> {
        let ws = WebSocket::new(url)?;
        ws.set_binary_type(BinaryType::Arraybuffer);

        let onopen_callback = Closure::wrap(Box::new(move |_| {
            log("socket opened");
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let onerror_callback = Closure::wrap(Box::new(move |ev: ErrorEvent| {
            log("an error occurred");
            log_value(&ev);
            log_value(&ev.error());
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let onmessage_callback = Closure::wrap(Box::new(move |ev: MessageEvent| {
            log("a message received");

            let message = match ev.data().dyn_into::<js_sys::Uint8Array>() {
                Ok(array) => {
                    log("Uint8Array");
                    let vec = array.to_vec();
                    decode(&vec).unwrap()
                }
                _ => return,
            };

            SOCK.with(|sock| {
                let mut on_message = sock.on_message.borrow_mut();
                if let Some(callback) = on_message.as_mut() {
                    callback(message)
                }
            })
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        Ok(Self {
            ws,
            on_message: RefCell::new(None),
        })
    }

    fn send(&self, message: Message) {
        if self.ws.ready_state() != 1 {
            return;
        }

        let mut buf = Vec::new();
        encode(&message, &mut buf).unwrap();

        match self.ws.send_with_u8_array(&buf) {
            Ok(_) => log("message successfully sent"),
            Err(e) => {
                log("error sending message");
                log_value(&e);
            }
        }
    }

    fn on_message<F>(&self, f: F)
    where
        F: Into<OnMessage>,
    {
        let mut on_message = self.on_message.borrow_mut();
        *on_message = Some(f.into());
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        if let Err(err) = self.ws.close() {
            log_value(&err);
        }
    }
}

thread_local! {
    static SOCK: Socket = {
        match Socket::new("ws://127.0.0.1:6789") {
            Ok(sock) => sock,
            Err(err) => {
                log_value(&err);
                panic!("received an error while starting a socket");
            }
        }
    };
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_value(s: &wasm_bindgen::JsValue);
}

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
fn chat(ChatProps { name }: &ChatProps) -> Html {
    html! {
        <div>
            { "Hi, " }
            <strong>{ name }</strong>
            { "!" }
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

    let onauth = props.onauth.clone();
    let onclick = Callback::from(move |_| {
        let form = document().get_element_by_id("auth_form").unwrap();
        let input = match form.query_selector("input[name='name']") {
            Ok(input) => input.unwrap(),
            Err(err) => {
                log_value(&err);
                return;
            }
        };

        if let Some(input) = input.dyn_ref::<HtmlInputElement>() {
            let name = input.value();
            let name = name.trim();

            if name.is_empty() {
                return;
            }

            SOCK.with(|sock| sock.send(Message::Auth { name: name.into() }));
            onauth.emit(name.into())
        }
    });

    html! {
        <div id="auth_form">
            <input type="text" name="name" />
            <button { onclick }>{ "Auth" }</button>
        </div>
    }
}
