
# PPM Web UI

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
