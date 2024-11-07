#![no_main]

use cranelift_assembler::{fuzz, Inst};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|inst: Inst| {
    fuzz::roundtrip(&inst);
});
