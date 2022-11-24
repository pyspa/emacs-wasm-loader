# WASI gives Emacs awesome power !!

This provides the ability to run `WASI: WebAssembly System Interface` program on Emacs.

This is still in the experimental stage.

This module is developed using `wasmtime`.

## Build

```
$ cargo build
```

## Usage

Set up the following in your emacs.

```lisp
;; load wasm loader
(module-load "path/to/target/debug/libwasm_loader.so")
;; register wasm modules
(wasm-loader/load "path/to/wasm-dir/")
```

Use `wasm-loader/call` to run program on WASI.

```lisp
;; call hello.wasm
(message (wasm-loader/call "hello" "test" "{ \"name\": \"WASM\" }"))
```

## Emacs interface for WASI

Basically, stdio is used to communicate with program.

To run on WASI, use the function `wasm-loader/call`.

This function takes three arguments.

1. wasm module name (this is the file name with the extension removed)
2. command type argument (e.g., get, setting )
3. command argument body (This is passed to the stdio. e.g., JSON for RPC requests)

WASI program reads the 2 and 3.

Here is a simple example of a program.

see `examples/hello/main.rs` .

```rust
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    command: String,
    name: String,
}

// This is a sample program that uses stdio to exchange values using the JSON format.
fn main() {
    // Reads args from Emacs from stdin.
    let mut raw_stdin: Vec<u8> = Vec::new();
    let mut stdin = io::stdin();
    stdin.read_to_end(&mut raw_stdin).ok();

    // convert
    let input: String = String::from_utf8(raw_stdin).unwrap();
    // deserialize
    let req: Request = serde_json::from_str(&input).unwrap();

    // create return response
    let mut res = Response {
        command: "".to_string(),
        name: format!("hello {}", req.name),
    };

    let args: Vec<String> = std::env::args().collect();
    if !args.is_empty() {
        // get command type from args
        let arg = args[0].clone();
        res.command = arg;
    }

    // to json
    let serialized = serde_json::to_string(&res).unwrap();

    println!("{}", serialized);
}

```

build WASI program.

```
$ cd example/hello
$ cargo build --target wasm32-wasi
```
