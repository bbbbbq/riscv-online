#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_riscv_online::assemble_with_xlen;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn rvc_basic_ci_and_cl_cs() {
    // c.addi4spn a0, 16 (rd must be compressed reg a0=x10)
    let out = assemble_with_xlen("c.addi4spn a0, 16", 32);
    assert!(out.trim().starts_with("0x"));

    // c.addi a0, -1
    let out = assemble_with_xlen("c.addi a0, -1", 32);
    assert!(out.trim().starts_with("0x"));

    // c.lw a0, 0(a1) and c.sw a0, 4(a1)
    let out = assemble_with_xlen("c.lw a0, 0(a1)", 32);
    assert!(out.trim().starts_with("0x"));
    let out = assemble_with_xlen("c.sw a0, 4(a1)", 32);
    assert!(out.trim().starts_with("0x"));
}

#[wasm_bindgen_test]
fn rvc_ci_ca_cr_misc() {
    // c.slli a0, 3
    let out = assemble_with_xlen("c.slli a0, 3", 32);
    assert!(out.trim().starts_with("0x"));

    // c.srli/c.srai/c.andi on compressed regs via a0
    let out = assemble_with_xlen("c.srli a0, 1", 32);
    assert!(out.trim().starts_with("0x"));
    let out = assemble_with_xlen("c.srai a0, 1", 32);
    assert!(out.trim().starts_with("0x"));
    let out = assemble_with_xlen("c.andi a0, -1", 32);
    assert!(out.trim().starts_with("0x"));

    // c.and a0, a1
    let out = assemble_with_xlen("c.and a0, a1", 32);
    assert!(out.trim().starts_with("0x"));

    // CR group: c.mv/c.add/c.jr/c.jalr
    let out = assemble_with_xlen("c.mv a0, a1", 32);
    assert!(out.trim().starts_with("0x"));
    let out = assemble_with_xlen("c.add a0, a1", 32);
    assert!(out.trim().starts_with("0x"));
    let out = assemble_with_xlen("c.jr ra", 32);
    assert!(out.trim().starts_with("0x"));
    let out = assemble_with_xlen("c.jalr ra", 32);
    assert!(out.trim().starts_with("0x"));
}

#[wasm_bindgen_test]
fn rvc_cj_cb_branches() {
    // c.j offset (RV32/64), c.jal only on RV32
    let out = assemble_with_xlen("c.j 8", 32);
    assert!(out.trim().starts_with("0x"));
    let out = assemble_with_xlen("c.jal 8", 32);
    assert!(out.trim().starts_with("0x"));

    // c.beqz/c.bnez with compressed rs1 (a0=x10)
    let out = assemble_with_xlen("c.beqz a0, 8", 32);
    assert!(out.trim().starts_with("0x"));
    let out = assemble_with_xlen("c.bnez a0, 8", 32);
    assert!(out.trim().starts_with("0x"));
}
