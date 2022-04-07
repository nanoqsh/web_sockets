use core::{encode, Message};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{BinaryType, ErrorEvent, MessageEvent, WebSocket};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_value(s: &wasm_bindgen::JsValue);
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let ws = WebSocket::new("ws://127.0.0.1:6789")?;
    ws.set_binary_type(BinaryType::Arraybuffer);

    let onopen_callback = Closure::wrap(Box::new({
        let ws = ws.clone();
        move |_| {
            log("socket opened");
            send(
                &ws,
                Message::Auth {
                    name: "nano".into(),
                },
            );
            send(&ws, Message::Text("hello".into()));
            send(&ws, Message::Text("sup!".into()));
        }
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
        log("an error occurred");
        log_value(&e);
        log_value(&e.error());
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let onmessage_callback = Closure::wrap(Box::new(move |_: MessageEvent| {
        log("a message received");
    }) as Box<dyn FnMut(MessageEvent)>);
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    // The socket needs to be closed after use.
    // ws.close()?;

    {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("document should have a body");

        let val = document.create_element("p")?;
        val.set_inner_html("Hello from Rust!");

        body.append_child(&val)?;
    }

    Ok(())
}

fn send(ws: &WebSocket, message: Message) {
    let mut buf = Vec::new();
    encode(&message, &mut buf).unwrap();

    match ws.send_with_u8_array(&buf) {
        Ok(_) => log("message successfully sent"),
        Err(e) => {
            log("error sending message");
            log_value(&e);
        }
    }
}
