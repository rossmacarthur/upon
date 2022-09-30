#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let mut engine = upon::Engine::new();
    let _ = engine.add_template("fuzz", data);
});
