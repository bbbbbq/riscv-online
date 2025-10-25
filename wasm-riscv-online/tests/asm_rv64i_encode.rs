#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_riscv_online::{assemble_with_xlen, assemble_auto};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn rv64i_w_variants_and_loads() {
    // addiw a0, a0, 1 -> 0x0015051b
    let out = assemble_with_xlen("addiw a0, a0, 1", 64);
    assert_eq!(out.trim(), "0x0015051b");

    // addw a0, a0, a1 via auto -> 0x00b5053b (auto should pick RV64)
    let out = assemble_auto("addw a0, a0, a1");
    assert_eq!(out.trim(), "0x00b5053b");

    // ld x1, 0(x2) only valid on RV64+
    let out32 = assemble_with_xlen("ld x1, 0(x2)", 32);
    assert!(out32.trim().starts_with("Error:"));
    let out64 = assemble_with_xlen("ld x1, 0(x2)", 64);
    assert_eq!(out64.trim(), "0x00013083");
}
