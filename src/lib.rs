use emacs::{defun, Env, Result, Value};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
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
    }
    Ok(())
}

#[defun]
fn call(env: &Env, name: String) -> Result<()> {
    if let Ok(engine) = WASM.lock() {
        // TODO pass args to stdin
        engine.call(env, name.as_str(), &[]);
    }
    Ok(())
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
        if let Ok(entries) = fs::read_dir(wasm_dir) {
            let entries: Vec<fs::DirEntry> = entries.flatten().collect();
            for entry in entries {
                if let Ok(path) = entry.path().canonicalize() {
                    if let Some(file) = path.file_stem() {
                        if let Ok(module) = Module::from_file(&self.engine, &path) {
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

    pub fn call(&self, env: &Env, name: &str, args: &[String]) -> anyhow::Result<()> {
        if let Some(module) = self.modules.get(name) {
            // new linker
            let mut linker = Linker::new(&self.engine);
            wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
            let wasi = WasiCtxBuilder::new().inherit_stdio().args(args)?.build();
            let mut store = Store::new(&self.engine, wasi);
            linker.module(&mut store, "", module)?;
            linker
                .get_default(&mut store, "")?
                .typed::<(), (), _>(&store)?
                .call(&mut store, ())?;
        } else {
            env.message(format!("unknown wasm: {}", name))?;
        }
        Ok(())
    }
}
