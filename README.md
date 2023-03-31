Rust/Webassembly Game of Life simulation, based on the tutorial at <https://rustwasm.github.io/>.

Some features I've added:
- parsing of plaintext and RLE "cells" files with `nom`
- only draw changed cells on each frame
- smooth changing of simulation speed
- gate wasm stuff behind a feature so it compiles as a regular rust lib
- strip out node/npm and in favor of vanilla ES modules

## Building

```
wasm-pack build --target web -d www/pkg --features wasm
```

Static site can be served from `www/`
