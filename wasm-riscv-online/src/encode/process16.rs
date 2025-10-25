use crate::{asm::Instruction, riscv::imm::Xlen};

// Placeholder: compressed instruction encoder will be implemented in Phase 2
#[allow(dead_code)]
pub fn encode_u16(_inst: &Instruction, _xlen: Xlen) -> Result<u16, String> {
    Err("RVC (compressed) encoding is not implemented yet".into())
}
