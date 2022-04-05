Run the server:
```
cargo run -p server
```

Build the web client and run locally:
```
wasm-pack build --target web ./web
python3 -m http.server --directory ./web
```
