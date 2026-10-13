# room1313

A horror/luck narrative card game inspired by Reigns, written in Rust and compiled to WebAssembly.

## Build and run

Needs [rustup](https://rustup.rs) and `python3`.

```sh
make setup   # once per machine: wasm target + wasm-pack
make run     # build, then serve at http://localhost:8765
```

`make help` lists everything else (`dev`, `test`, `lint`, `check`, `clean`…).
Use another port with `make serve PORT=9000`.
