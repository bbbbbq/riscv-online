#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);
use wasm_riscv_online::{assemble_auto, assemble_with_xlen};

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
    assert_eq!(out64.trim(), "0x00013083"); // ld rd=1 rs1=2 imm=0 -> opcode LOAD, funct3=011
}

#[wasm_bindgen_test]
fn zicsr_and_system() {
    // csrrw x1, 0x305, x2 -> csr=0x305, rs1=2, rd=1
    let out = assemble_with_xlen("csrrw x1, 0x305, x2", 32);
    assert_eq!(out.trim(), "0x305110f3");

    // csrrwi x0, 0xc00, 3 -> csr=0xc00, uimm=3, rd=0
    let out = assemble_with_xlen("csrrwi x0, 0xc00, 3", 32);
    assert_eq!(out.trim(), "0xc001d073");

    // fence/fence.i/ecall/ebreak
    assert_eq!(assemble_with_xlen("fence", 32).trim(), "0x0000000f");
    assert_eq!(assemble_with_xlen("fence.i", 32).trim(), "0x0000100f");
    assert_eq!(assemble_with_xlen("ecall", 32).trim(), "0x00000073");
    assert_eq!(assemble_with_xlen("ebreak", 32).trim(), "0x00100073");
}
