use crate::asm::*;
use crate::riscv::imm::{Imm, Uimm, Xlen};
use super::common::{parse_int, parse_mem_operand, parse_register, imm_signed_bits};

pub(crate) fn try_parse(mnem: &str, ops: &[String], xlen: Xlen) -> Option<Result<Instruction, String>> {
    match mnem {
        // CIW: c.addi4spn rd, uimm
        "c.addi4spn" => {
            if ops.len() != 2 { return Some(Err("用法: c.addi4spn rd, uimm".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let uimm_val = match parse_int(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if uimm_val <= 0 { return Some(Err("c.addi4spn 的 uimm 必须为正且非零".into())); }
            let u = uimm_val as u32;
            if (u & 0x3) != 0 { return Some(Err("c.addi4spn 的 uimm 必须按 4 字节对齐".into())); }
            if u >= (1 << 10) { return Some(Err("c.addi4spn 的 uimm 超出 10 位范围".into())); }
            let ciw = CIWType { rd, funct3: 0, uimm: Uimm::new(u, 10) };
            Some(Ok(RVC::Caddi4spn(ciw).into()))
        }
        // CI-like: c.addi rd, imm
        "c.addi" => {
            if ops.len() != 2 { return Some(Err("用法: c.addi rd, imm".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm = match parse_int(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm_bits = match imm_signed_bits(imm, 6) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let ci = CIType { rdrs1: rd, funct3: 0, imm: Imm::new(imm_bits, 6) };
            Some(Ok(RVC::Caddi(ci).into()))
        }
        // c.li rd, imm
        "c.li" => {
            if ops.len() != 2 { return Some(Err("用法: c.li rd, imm".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm = match parse_int(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm_bits = match imm_signed_bits(imm, 6) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let ci = CIType { rdrs1: rd, funct3: 0, imm: Imm::new(imm_bits, 6) };
            Some(Ok(RVC::Cli(ci).into()))
        }
        // c.nop
        "c.nop" => {
            let ci = CIType { rdrs1: 0, funct3: 0, imm: Imm::new(0, 6) };
            Some(Ok(RVC::Cnop(ci).into()))
        }
        // CL: c.lw rd, imm(rs1)
        "c.lw" => {
            if ops.len() != 2 { return Some(Err("用法: c.lw rd, imm(rs1)".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, rs1) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let cl = CLType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 7) };
            Some(Ok(RVC::Clw(cl).into()))
        }
        // CS: c.sw rs2, imm(rs1)
        "c.sw" => {
            if ops.len() != 2 { return Some(Err("用法: c.sw rs2, imm(rs1)".into())); }
            let rs2 = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, rs1) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let cs = CSType { rs1, rs2, funct3: 0, imm: Imm::new(imm_bits, 7) };
            Some(Ok(RVC::Csw(cs).into()))
        }
        // C.LWSP: c.lwsp rd, imm(sp)
        "c.lwsp" => {
            if ops.len() != 2 { return Some(Err("用法: c.lwsp rd, imm(sp)".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, rs) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if rs != 2 { return Some(Err("c.lwsp 基址必须是 sp".into())); }
            let ci = CIType { rdrs1: rd, funct3: 0, imm: Imm::new(imm_bits, 8) };
            Some(Ok(RVC::Clwsp(ci).into()))
        }
        // C.SWSP: c.swsp rs2, imm(sp)
        "c.swsp" => {
            if ops.len() != 2 { return Some(Err("用法: c.swsp rs2, imm(sp)".into())); }
            let rs2 = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, rs) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if rs != 2 { return Some(Err("c.swsp 基址必须是 sp".into())); }
            let css = CSSType { rs2, funct3: 0, imm: Imm::new(imm_bits, 8) };
            Some(Ok(RVC::Cswsp(css).into()))
        }
        // RV64+: c.ld/c.sd/c.ldsp/c.sdsp
        "c.ld" => {
            if ops.len() != 2 { return Some(Err("用法: c.ld rd, imm(rs1)".into())); }
            match xlen { Xlen::X64 | Xlen::X128 => {}, _ => return Some(Err("c.ld 仅在 RV64/128 可用".into())) }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, rs1) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let cl = CLType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 8) };
            Some(Ok(RVC::Cld(cl).into()))
        }
        "c.sd" => {
            if ops.len() != 2 { return Some(Err("用法: c.sd rs2, imm(rs1)".into())); }
            match xlen { Xlen::X64 | Xlen::X128 => {}, _ => return Some(Err("c.sd 仅在 RV64/128 可用".into())) }
            let rs2 = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, rs1) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let cs = CSType { rs1, rs2, funct3: 0, imm: Imm::new(imm_bits, 8) };
            Some(Ok(RVC::Csd(cs).into()))
        }
        "c.ldsp" => {
            if ops.len() != 2 { return Some(Err("用法: c.ldsp rd, imm(sp)".into())); }
            match xlen { Xlen::X64 | Xlen::X128 => {}, _ => return Some(Err("c.ldsp 仅在 RV64/128 可用".into())) }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, rs) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if rs != 2 { return Some(Err("c.ldsp 基址必须是 sp".into())); }
            let ci = CIType { rdrs1: rd, funct3: 0, imm: Imm::new(imm_bits, 9) };
            Some(Ok(RVC::Cldsp(ci).into()))
        }
        "c.sdsp" => {
            if ops.len() != 2 { return Some(Err("用法: c.sdsp rs2, imm(sp)".into())); }
            match xlen { Xlen::X64 | Xlen::X128 => {}, _ => return Some(Err("c.sdsp 仅在 RV64/128 可用".into())) }
            let rs2 = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, _rs) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let css = CSSType { rs2, funct3: 0, imm: Imm::new(imm_bits, 9) };
            Some(Ok(RVC::Csdsp(css).into()))
        }
        _ => None,
    }
}
