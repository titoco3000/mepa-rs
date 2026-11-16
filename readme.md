# MEPA Web

Browser playground for [mepa-rs](https://github.com/titoco3000/mepa-rs), the compiler, optimizer and virtual machine from my undergraduate thesis
("Implementing intermediate-code optimizations on MEPA code").

**Live demo:** https://titoco3000.github.io/mepa-web/

<img width="859" height="617" alt="image" src="https://github.com/user-attachments/assets/37b55770-7bb2-4461-bc54-4f453fa4dc91" />

## What it does

- Write a program in *ipt* (a small integer-and-pointer language) or raw MEPA in the code editor.
- Compile it to MEPA and apply the optimizer passes.
- Step through execution on the MEPA stack machine and watch the memory (`M`), display registers (`D`) and the instruction pointer.

Everything runs client-side: the Rust crate is compiled to WebAssembly (`src/pkg/`) and loaded by a React UI.

## Stack

React, Vite, WebAssembly (Rust via `wasm-pack`).

## Running locally

```bash
npm install
npm run dev
```

To rebuild the WebAssembly package from mepa-rs:

```bash
# in the mepa-rs checkout
wasm-pack build --target web
# copy the generated pkg/ into this repo's src/pkg/
```

## Related

- [mepa-rs](https://github.com/titoco3000/mepa-rs): compiler, optimizer and VM (Rust)
- [cLiteCompiler](https://github.com/titoco3000/cLiteCompiler): earlier C compiler targeting MEPA
