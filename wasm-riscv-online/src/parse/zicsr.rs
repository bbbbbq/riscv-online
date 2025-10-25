use crate::asm::*;
use crate::riscv::imm::{Uimm, Xlen};
use super::common::{parse_int, parse_register};

pub(crate) fn try_parse(mnem: &str, ops: &[String], _xlen: Xlen) -> Option<Result<Instruction, String>> {
    match mnem {
        "csrrw" | "csrrs" | "csrrc" => {
            if ops.len() != 3 { return Some(Err("用法: csrr{w|s|c} rd, csr, rs1".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let csr_val = match parse_int(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if csr_val < 0 || csr_val > 0xFFF { return Some(Err("csr 编号应为 0..0xFFF".into())); }
            let rs1 = match parse_register(&ops[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let c = CsrRType { rd, rs1, funct3: 0, csr: csr_val as u16 };
            let inst = match mnem {
                "csrrw" => RVZicsr::Csrrw(c).into(),
                "csrrs" => RVZicsr::Csrrs(c).into(),
                "csrrc" => RVZicsr::Csrrc(c).into(),
                _ => unreachable!(),
            };
            Some(Ok(inst))
        }
        "csrrwi" | "csrrsi" | "csrrci" => {
            if ops.len() != 3 { return Some(Err("用法: csrr{x}i rd, csr, uimm".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let csr_val = match parse_int(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if csr_val < 0 || csr_val > 0xFFF { return Some(Err("csr 编号应为 0..0xFFF".into())); }
            let uimm_val = match parse_int(&ops[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if uimm_val < 0 || uimm_val > 31 { return Some(Err("uimm 取值 0..31".into())); }
            let c = CsrIType { rd, uimm: Uimm::new(uimm_val as u32, 5), funct3: 0, csr: csr_val as u16 };
            let inst = match mnem {
                "csrrwi" => RVZicsr::Csrrwi(c).into(),
                "csrrsi" => RVZicsr::Csrrsi(c).into(),
                "csrrci" => RVZicsr::Csrrci(c).into(),
                _ => unreachable!(),
            };
            Some(Ok(inst))
        }
        _ => None,
    }
}
