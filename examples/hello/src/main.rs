// build
// cargo build --target wasm32-wasi
//

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
        let arg = args[1].clone();
        res.command = arg;
    }

    // to json
    let serialized = serde_json::to_string(&res).unwrap();
    // return response json
    println!("{}", serialized);
}
