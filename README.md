Run the server:
```
cargo run -p server
```

Build the web client and run locally:
```
wasm-pack build -t web ./web
python -m http.server --directory ./web
```
