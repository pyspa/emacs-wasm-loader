# This gives Emacs awesome power !!

This provides the ability to load and call `wasm` modules into Emacs.

This is still in the experimental stage.

## Build

```
$ cargo build
```

## Usage

```lisp
;; load wasm loader
(module-load "path/to/target/debug/libwasm_loader.so")
;; register wasm modules
(wasm-loader/load "path/to/wasm-dir/")
;; call hello.wasm
(wasm-loader/call "hello")

```
