//! wasm.rs — fuzzes WebAssembly validation and compilation as the WebAssembly API does it, then instantiation and calls under a fuel limit; the portable dispatch loop stands in for tail-call dispatch, which ASan builds cannot compile to jumps.
#![no_main]

use wasmi::{Config, Engine, Func, Linker, Module, Store, Val};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let engine = Engine::default();
    if Module::new(&engine, data).is_err() {
        return;
    }
    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    let Ok(module) = Module::new(&engine, data) else {
        return;
    };
    let mut store = Store::new(&engine, ());
    if store.set_fuel(200_000).is_err() {
        return;
    }
    let linker = Linker::<()>::new(&engine);
    let Ok(instance) = linker.instantiate_and_start(&mut store, &module) else {
        return;
    };
    let funcs: Vec<Func> = instance.exports(&store).filter_map(|e| e.into_func()).take(8).collect();
    for func in funcs {
        let ty = func.ty(&store);
        let params: Vec<Val> = ty.params().iter().map(|&t| Val::default_for_ty(t)).collect();
        let mut results: Vec<Val> = ty.results().iter().map(|&t| Val::default_for_ty(t)).collect();
        let _ = store.set_fuel(200_000);
        std::hint::black_box(func.call(&mut store, &params, &mut results).is_ok());
    }
});
