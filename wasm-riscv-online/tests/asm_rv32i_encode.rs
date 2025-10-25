#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_riscv_online::assemble_with_xlen;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn rv32i_basic_encode() {
    // addi x1, x2, 10 -> 0x00a10093
    let out = assemble_with_xlen("addi x1, x2, 10", 32);
    assert_eq!(out.trim(), "0x00a10093");

    // lw x5, 0(x2) -> 0x00012283
    let out = assemble_with_xlen("lw x5, 0(x2)", 32);
    assert_eq!(out.trim(), "0x00012283");

    // sw x5, 4(x2) -> 0x00512223
    let out = assemble_with_xlen("sw x5, 4(x2)", 32);
    assert_eq!(out.trim(), "0x00512223");

    // beq x1, x2, 8 -> 0x00208463
    let out = assemble_with_xlen("beq x1, x2, 8", 32);
    assert_eq!(out.trim(), "0x00208463");

    // jal x1, 12 -> 0x00c000ef
    let out = assemble_with_xlen("jal x1, 12", 32);
    assert_eq!(out.trim(), "0x00c000ef");

    // jalr x1, 0(x2) -> 0x000100e7
    let out = assemble_with_xlen("jalr x1, 0(x2)", 32);
    assert_eq!(out.trim(), "0x000100e7");

    // slli x1, x1, 5 -> 0x00509093
    let out = assemble_with_xlen("slli x1, x1, 5", 32);
    assert_eq!(out.trim(), "0x00509093");

    // lui x3, 0x12345 -> 0x123451b7
    let out = assemble_with_xlen("lui x3, 0x12345", 32);
    assert_eq!(out.trim(), "0x123451b7");
}
