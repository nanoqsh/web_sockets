use core::{decode, encode, Message};
use gloo::console::log;
use std::cell::RefCell;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{BinaryType, ErrorEvent, MessageEvent, WebSocket};

type OnMessage = Box<dyn FnMut(Message)>;

pub struct Socket {
    ws: WebSocket,
    on_message: RefCell<Option<OnMessage>>,
}

impl Socket {
    fn new(url: &str) -> Result<Self, JsValue> {
        let ws = WebSocket::new(url)?;
        ws.set_binary_type(BinaryType::Arraybuffer);

        let onopen_callback = Closure::wrap(Box::new(move |_| {
            log!("socket opened");
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let onerror_callback = Closure::wrap(Box::new(move |ev: ErrorEvent| {
            log!("an error occurred");
            log!(&ev);
            log!(&ev.error());
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let onmessage_callback = Closure::wrap(Box::new(move |ev: MessageEvent| {
            log!("a message received");

            let message = match ev.data().dyn_into::<js_sys::ArrayBuffer>() {
                Ok(array) => {
                    let array = js_sys::Uint8Array::new(&array);
                    let vec = array.to_vec();
                    decode(&vec).unwrap()
                }
                _ => return,
            };

            SOCK.with(|sock| {
                let mut on_message = sock.on_message.borrow_mut();
                if let Some(callback) = on_message.as_mut() {
                    callback(message);
                }
            });
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        Ok(Self {
            ws,
            on_message: RefCell::new(None),
        })
    }

    pub fn send(&self, message: &Message) {
        const OPEN: u16 = 1;

        if self.ws.ready_state() != OPEN {
            return;
        }

        let mut buf = Vec::new();
        encode(message, &mut buf).unwrap();

        match self.ws.send_with_u8_array(&buf) {
            Ok(_) => log!("message successfully sent"),
            Err(e) => {
                log!("error sending message");
                log!(e);
            }
        }
    }

    pub fn on_message(&self, f: OnMessage) {
        if let Ok(mut on_message) = self.on_message.try_borrow_mut() {
            on_message.replace(f);
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        if let Err(err) = self.ws.close() {
            log!(err);
        }
    }
}

thread_local! {
    pub static SOCK: Socket = {
        match Socket::new("ws://127.0.0.1:6789") {
            Ok(sock) => sock,
            Err(err) => {
                log!(err);
                panic!("received an error while starting a socket");
            }
        }
    };
}
