//! RV64A atomic instruction tests for RISC-V online disassembler

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_riscv_online::disassemble;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_rv64a_load_reserved_store_conditional() {
    // LR.D x1, (x2) -> 0x100130AF
    let result = disassemble("100130af");
    assert!(!result.starts_with("Error"), "Failed to decode lr.d: {}", result);
    assert!(result.contains("lr.d"), "Expected 'lr.d' in '{}'", result);

    // SC.D x1, x3, (x2) -> 0x183130AF
    let result = disassemble("183130af");
    assert!(!result.starts_with("Error"), "Failed to decode sc.d: {}", result);
    assert!(result.contains("sc.d"), "Expected 'sc.d' in '{}'", result);
}

#[wasm_bindgen_test]
fn test_rv64a_atomic_swap() {
    // AMOSWAP.D x1, x3, (x2) -> 0x083130AF
    let result = disassemble("083130af");
    assert!(!result.starts_with("Error"), "Failed to decode amoswap.d: {}", result);
    assert!(result.contains("amoswap.d"), "Expected 'amoswap.d' in '{}'", result);
}

#[wasm_bindgen_test]
fn test_rv64a_atomic_arithmetic() {
    let test_cases = vec![
        ("003130af", "amoadd.d"),   // AMOADD.D x1, x3, (x2)
        ("203130af", "amoxor.d"),   // AMOXOR.D x1, x3, (x2)
        ("603130af", "amoand.d"),   // AMOAND.D x1, x3, (x2)
        ("403130af", "amoor.d"),    // AMOOR.D x1, x3, (x2)
        ("803130af", "amomin.d"),   // AMOMIN.D x1, x3, (x2)
        ("a03130af", "amomax.d"),   // AMOMAX.D x1, x3, (x2)
        ("c03130af", "amominu.d"),  // AMOMINU.D x1, x3, (x2)
        ("e03130af", "amomaxu.d"),  // AMOMAXU.D x1, x3, (x2)
    ];

    for (encoding, mnemonic) in test_cases {
        let result = disassemble(encoding);
        assert!(!result.starts_with("Error"), "Failed to decode {}: {}", mnemonic, result);
        assert!(result.contains(mnemonic), "Expected '{}' in '{}'", mnemonic, result);
    }
}

#[wasm_bindgen_test]
fn test_rv64a_with_acquire_release() {
    // LR.D.AQ x1, (x2) -> 0x140130AF
    let result = disassemble("140130af");
    assert!(!result.starts_with("Error"), "Failed to decode lr.d with acquire: {}", result);
    assert!(result.contains("lr.d"), "Expected 'lr.d' in '{}'", result);

    // SC.D.RL x1, x3, (x2) -> 0x1A3130AF
    let result = disassemble("1a3130af");
    assert!(!result.starts_with("Error"), "Failed to decode sc.d with release: {}", result);
    assert!(result.contains("sc.d"), "Expected 'sc.d' in '{}'", result);

    // AMOSWAP.D.AQRL x1, x3, (x2) -> 0x0E3130AF
    let result = disassemble("0e3130af");
    assert!(!result.starts_with("Error"), "Failed to decode amoswap.d with aq+rl: {}", result);
    assert!(result.contains("amoswap.d"), "Expected 'amoswap.d' in '{}'", result);
}

#[wasm_bindgen_test]
fn test_rv64a_different_registers() {
    let test_cases = vec![
        ("0050baaf", "amoadd.d"),   // AMOADD.D x21, x5, (x1) - different registers
        ("100f30af", "lr.d"),       // LR.D x1, (x30) - different base register
    ];

    for (encoding, mnemonic) in test_cases {
        let result = disassemble(encoding);
        assert!(!result.starts_with("Error"), "Failed to decode {}: {}", mnemonic, result);
        assert!(result.contains(mnemonic), "Expected '{}' in '{}'", mnemonic, result);
    }
}

#[wasm_bindgen_test]
fn test_rv64a_comprehensive_coverage() {
    let test_cases = vec![
        ("100130af", "lr.d"),       // LR.D x1, (x2)
        ("183130af", "sc.d"),       // SC.D x1, x3, (x2)
        ("083130af", "amoswap.d"),  // AMOSWAP.D x1, x3, (x2)
        ("003130af", "amoadd.d"),   // AMOADD.D x1, x3, (x2)
        ("203130af", "amoxor.d"),   // AMOXOR.D x1, x3, (x2)
        ("603130af", "amoand.d"),   // AMOAND.D x1, x3, (x2)
        ("403130af", "amoor.d"),    // AMOOR.D x1, x3, (x2)
        ("803130af", "amomin.d"),   // AMOMIN.D x1, x3, (x2)
        ("a03130af", "amomax.d"),   // AMOMAX.D x1, x3, (x2)
        ("c03130af", "amominu.d"),  // AMOMINU.D x1, x3, (x2)
        ("e03130af", "amomaxu.d"),  // AMOMAXU.D x1, x3, (x2)
    ];

    for (hex_input, expected_mnemonic) in test_cases {
        let result = disassemble(hex_input);
        assert!(!result.starts_with("Error"), 
                "Failed to decode instruction: {}", hex_input);
        assert!(result.contains(expected_mnemonic), 
                "Expected '{}' in disassembly '{}' for instruction {}", 
                expected_mnemonic, result, hex_input);
    }
}

#[wasm_bindgen_test]
fn test_rv64a_error_cases() {
    let result = disassemble("f83130af"); // Invalid funct5=11111
    assert!(result.starts_with("Error"), "Expected error for invalid funct5");

    let result = disassemble("003140af"); // Invalid funct3=100
    assert!(result.starts_with("Error"), "Expected error for invalid funct3");

    let result = disassemble("invalid_hex");
    assert!(result.starts_with("Error"), "Expected error for invalid hex input");
}