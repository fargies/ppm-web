
# PPM Web UI

[![CI](https://github.com/fargies/ppm-web/actions/workflows/main.yml/badge.svg)](https://github.com/fargies/ppm-web/actions/workflows/main.yml)
[![Coverage](https://coveralls.io/repos/github/fargies/ppm-web/badge.svg?branch=master)](https://coveralls.io/github/fargies/ppm-web?branch=master)
[![License](https://img.shields.io/badge/license-Zlib-green)](LICENSE)

[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-GitHub-ea4aaa?logo=githubsponsors)](https://github.com/sponsors/fargies)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-FFDD00?logo=buymeacoffee&logoColor=black)](https://www.buymeacoffee.com/fargies)

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
