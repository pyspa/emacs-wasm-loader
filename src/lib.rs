use emacs::{defun, Env, Result, Value};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;

emacs::plugin_is_GPL_compatible!();

pub static WASM: Lazy<Mutex<WASMEngine>> = Lazy::new(|| Mutex::new(Default::default()));

#[emacs::module(name = "wasm-loader", separator = "/")]
pub fn init(env: &Env) -> Result<Value<'_>> {
    env.message("init wasm loader")
}

#[defun]
fn load(env: &Env, dir: String) -> Result<()> {
    if let Ok(mut engine) = WASM.lock() {
        engine.load(env, &dir)?;
    } else {
        eprint!("wasm engine is not initialized");
    }
    Ok(())
}

#[defun]
fn call(env: &Env, name: String, command: String, command_arg: String) -> Result<Option<String>> {
    if let Ok(engine) = WASM.lock() {
        match engine.call(env, name.as_str(), &[command, command_arg]) {
            Ok(out) => Ok(Some(out)),
            Err(err) => {
                // show wasm module error
                eprint!("{:?}", err);
                // TODO return error message ???
                Ok(None)
            }
        }
    } else {
        eprint!("wasm engine is not initialized");
        Ok(None)
    }
}

pub struct WASMEngine {
    engine: Engine,
    modules: HashMap<String, Module>,
}

impl Default for WASMEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for WASMEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("WASMEngine").finish()
    }
}

impl WASMEngine {
    pub fn new() -> Self {
        let engine = Engine::default();
        let modules: HashMap<String, Module> = HashMap::new();
        WASMEngine { engine, modules }
    }

    pub fn load(&mut self, env: &Env, wasm_dir: &str) -> Result<()> {
        // search wasm file
        if let Ok(entries) = fs::read_dir(wasm_dir) {
            let entries: Vec<fs::DirEntry> = entries
                .flatten()
                .filter(|x| x.path().extension().unwrap_or_default() == "wasm") // filer .wasm
                .collect();

            for entry in entries {
                if let Ok(path) = entry.path().canonicalize() {
                    if let Some(file) = path.file_stem() {
                        if let Ok(module) = Module::from_file(&self.engine, &path) {
                            // register wasm module
                            env.message(format!("register wasm module {:?} {:?}", &file, &path))?;
                            let name = file.to_string_lossy().to_string();
                            self.modules.entry(name).or_insert(module);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn call(&self, _env: &Env, name: &str, args: &[String]) -> anyhow::Result<String> {
        if let Some(module) = self.modules.get(name) {
            let stdout = {
                // new linker
                let mut linker = Linker::new(&self.engine);
                wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

                // set stdin
                let stdin = ReadPipe::from(args[1].to_string());
                let stdout = WritePipe::new_in_memory();

                // build wasi ctx
                let ctx = WasiCtxBuilder::new()
                    //.inherit_stdio()
                    .stdin(Box::new(stdin))
                    .stdout(Box::new(stdout.clone()))
                    .args(args)?
                    .build();

                let mut store = Store::new(&self.engine, ctx);
                linker.module(&mut store, "", module)?;

                // debug
                // env.message(format!("call wasm: {}.wasm", name))?;

                // call main
                linker
                    .get_default(&mut store, "")?
                    .typed::<(), (), _>(&store)?
                    .call(&mut store, ())?;
                // drop ctx
                stdout
            };

            // get captured output
            let output: Vec<u8> = stdout
                .try_into_inner()
                .expect("sole remaining reference to WritePipe")
                .into_inner();
            let out: String = String::from_utf8(output)?;

            Ok(out)
        } else {
            anyhow::bail!(format!("unknown wasm module: {}", name))
        }
    }
}
