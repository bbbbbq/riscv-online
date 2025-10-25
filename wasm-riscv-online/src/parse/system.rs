use crate::asm::*;
use crate::riscv::imm::Xlen;

pub(crate) fn try_parse(mnem: &str, _ops: &[String], _xlen: Xlen) -> Option<Result<Instruction, String>> {
    match mnem {
        "ecall" => Some(Ok(RV32I::Ecall(()).into())),
        "ebreak" => Some(Ok(RV32I::Ebreak(()).into())),
        "fence" => Some(Ok(RV32I::Fence(()).into())),
        "fence.i" | "fencei" => Some(Ok(RV32I::FenceI(()).into())),
        _ => None,
    }
}
