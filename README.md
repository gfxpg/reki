# Reki

_Reki_ (礫) is a study project to explore the low-level foundations
of general-purpose GPU computing.

## Prerequisites

* npm
* Rust
* [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)

## Development

UI:

```
npm run dev
```

Core lib:

```
wasm-pack build src/ --dev --target no-modules --out-dir ../dist/wasm
```
