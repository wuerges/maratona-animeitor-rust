Regenerating WASM:

```wasm-pack build --target web --out-name wasm --out-dir ./static```

Running the development server:

```python -mhttp.server```

Note: the flow above is legacy. The current client is built with trunk via
`make rebuild-client-for-release` (see the repository root `Makefile`).

Client assets are served from memory: at startup the server loads the client
build directory (the `-v <dir>:` volume mounted at the root) and
`user-styles.css` into RAM and serves them precompressed, with no filesystem
reads afterwards. Changes to the client or styles require a server restart.
