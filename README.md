
# PPM Web UI

Web User Interface for [PPM](https://github.gremory.org/ppm/).

## Tests

```bash
# tests may be ran directly from cargo
cargo test

# to run `vitest` in watch mode
cd src/app ; npm install && npx vitest --coverage
```

## Development

```bash
# Install cargo-make if not already installed
cargo install cargo-make

# Will run:
# - ppm-webapp to serve websocket to daemon
# - ppm-daemon with sample configuration
# - vite to serve the front-end with sourcemap and hot-reload
cargo make serve:dev
```
