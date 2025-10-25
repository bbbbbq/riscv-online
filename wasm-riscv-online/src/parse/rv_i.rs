use crate::asm::*;
use crate::riscv::imm::{Imm, Xlen};
use super::common::{parse_register, parse_int, imm_signed_bits, parse_mem_operand};

pub(crate) fn try_parse(mnem: &str, ops: &[String], xlen: Xlen) -> Option<Result<Instruction, String>> {
    match mnem {
        // U-type
        "lui" => {
            if ops.len() != 2 { return Some(Err("用法: lui rd, imm20".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm = match parse_int(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm_bits = ((imm as i64) as u32) << 12;
            let u = UType { rd, imm: Imm::new(imm_bits, 32) };
            Some(Ok(RV32I::Lui(u).into()))
        }
        "auipc" => {
            if ops.len() != 2 { return Some(Err("用法: auipc rd, imm20".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm = match parse_int(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm_bits = ((imm as i64) as u32) << 12;
            let u = UType { rd, imm: Imm::new(imm_bits, 32) };
            Some(Ok(RV32I::Auipc(u).into()))
        }

        // J-type
        "jal" => {
            if ops.len() != 2 { return Some(Err("用法: jal rd, imm".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm = match parse_int(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm_bits = match imm_signed_bits(imm, 21) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let j = JType { rd, imm: Imm::new(imm_bits, 12) }; // keep 12 valid bits consistent with decoder
            Some(Ok(RV32I::Jal(j).into()))
        }
        // JALR
        "jalr" => {
            if ops.len() == 2 {
                let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                let (imm_bits, rs1) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
                Some(Ok(RV32I::Jalr(i).into()))
            } else if ops.len() == 3 {
                let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                let rs1 = match parse_register(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                let imm = match parse_int(&ops[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                let imm_bits = match imm_signed_bits(imm, 12) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
                Some(Ok(RV32I::Jalr(i).into()))
            } else { Some(Err("用法: jalr rd, imm(rs1) 或 jalr rd, rs1, imm".into())) }
        }

        // B-type
        "beq" | "bne" | "blt" | "bge" | "bltu" | "bgeu" => {
            if ops.len() != 3 { return Some(Err("用法: beq rs1, rs2, imm".into())); }
            let rs1 = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let rs2 = match parse_register(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm = match parse_int(&ops[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if (imm & 1) != 0 { return Some(Err("分支偏移必须是2字节对齐".into())); }
            let imm_bits = match imm_signed_bits(imm, 13) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let b = BType { rs1, rs2, funct3: 0, imm: Imm::new(imm_bits, 12) };
            let inst = match mnem {
                "beq" => RV32I::Beq(b).into(),
                "bne" => RV32I::Bne(b).into(),
                "blt" => RV32I::Blt(b).into(),
                "bge" => RV32I::Bge(b).into(),
                "bltu" => RV32I::Bltu(b).into(),
                "bgeu" => RV32I::Bgeu(b).into(),
                _ => unreachable!(),
            };
            Some(Ok(inst))
        }

        // Loads
        "lb" | "lh" | "lw" | "lbu" | "lhu" | "lwu" | "ld" => {
            if ops.len() != 2 { return Some(Err("用法: lw rd, imm(rs1)".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, rs1) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
            let inst = match mnem {
                "lb" => RV32I::Lb(i).into(),
                "lh" => RV32I::Lh(i).into(),
                "lw" => RV32I::Lw(i).into(),
                "lbu" => RV32I::Lbu(i).into(),
                "lhu" => RV32I::Lhu(i).into(),
                "lwu" => match xlen { Xlen::X64 | Xlen::X128 => RV64I::Lwu(i).into(), _ => return Some(Err("lwu 仅在 RV64/128 可用".into())) },
                "ld"  => match xlen { Xlen::X64 | Xlen::X128 => RV64I::Ld(i).into(), _ => return Some(Err("ld 仅在 RV64/128 可用".into())) },
                _ => unreachable!(),
            };
            Some(Ok(inst))
        }

        // Stores
        "sb" | "sh" | "sw" | "sd" => {
            if ops.len() != 2 { return Some(Err("用法: sw rs2, imm(rs1)".into())); }
            let rs2 = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let (imm_bits, rs1) = match parse_mem_operand(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let s = SType { rs1, rs2, funct3: 0, imm: Imm::new(imm_bits, 12) };
            let inst = match mnem {
                "sb" => RV32I::Sb(s).into(),
                "sh" => RV32I::Sh(s).into(),
                "sw" => RV32I::Sw(s).into(),
                "sd" => match xlen { Xlen::X64 | Xlen::X128 => RV64I::Sd(s).into(), _ => return Some(Err("sd 仅在 RV64/128 可用".into())) },
                _ => unreachable!(),
            };
            Some(Ok(inst))
        }

        // OP-IMM
        "addi" | "slti" | "sltiu" | "xori" | "ori" | "andi" => {
            if ops.len() != 3 { return Some(Err("用法: addi rd, rs1, imm".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let rs1 = match parse_register(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm = match parse_int(&ops[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let imm_bits = match imm_signed_bits(imm, 12) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
            let inst = match mnem {
                "addi" => RV32I::Addi(i).into(),
                "slti" => RV32I::Slti(i).into(),
                "sltiu" => RV32I::Sltiu(i).into(),
                "xori" => RV32I::Xori(i).into(),
                "ori" => RV32I::Ori(i).into(),
                "andi" => RV32I::Andi(i).into(),
                _ => unreachable!(),
            };
            Some(Ok(inst))
        }
        // shifts immediate (xlen-sensitive)
        "slli" | "srli" | "srai" => {
            if ops.len() != 3 { return Some(Err("用法: slli rd, rs1, shamt".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let rs1 = match parse_register(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let shamt = match parse_int(&ops[2]) { Ok(v) => v as u32, Err(e) => return Some(Err(e)) };
            let bits = match xlen { Xlen::X32 => 5, Xlen::X64 | Xlen::X128 => 6 };
            // range check
            if shamt >= (1u32 << bits) { return Some(Err(format!("shamt {} 超出范围 ({} 位)", shamt, bits))); }
            let i = IType { rd, rs1, funct3: 0, imm: Imm::new(shamt, bits) };
            let inst = match (mnem, xlen) {
                ("slli", Xlen::X32) => RV32I::Slli(i).into(),
                ("srli", Xlen::X32) => RV32I::Srli(i).into(),
                ("srai", Xlen::X32) => RV32I::Srai(i).into(),
                ("slli", _) => RV64I::Slli(i).into(),
                ("srli", _) => RV64I::Srli(i).into(),
                ("srai", _) => RV64I::Srai(i).into(),
                _ => unreachable!(),
            };
            Some(Ok(inst))
        }

        // OP (R-type)
        "add" | "sub" | "sll" | "slt" | "sltu" | "xor" | "srl" | "sra" | "or" | "and" => {
            if ops.len() != 3 { return Some(Err("用法: add rd, rs1, rs2".into())); }
            let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let rs1 = match parse_register(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let rs2 = match parse_register(&ops[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let r = RType { rd, rs1, rs2, funct3: 0, funct7: 0 };
            let inst = match mnem {
                "add" => RV32I::Add(r).into(),
                "sub" => RV32I::Sub(r).into(),
                "sll" => RV32I::Sll(r).into(),
                "slt" => RV32I::Slt(r).into(),
                "sltu" => RV32I::Sltu(r).into(),
                "xor" => RV32I::Xor(r).into(),
                "srl" => RV32I::Srl(r).into(),
                "sra" => RV32I::Sra(r).into(),
                "or"  => RV32I::Or(r).into(),
                "and" => RV32I::And(r).into(),
                _ => unreachable!(),
            };
            Some(Ok(inst))
        }

        // RV64I W variants
        "addiw" | "slliw" | "srliw" | "sraiw" | "addw" | "subw" | "sllw" | "srlw" | "sraw" => {
            match xlen { Xlen::X64 | Xlen::X128 => {}, _ => return Some(Err("该指令仅在 RV64/128 可用".into())) }
            match mnem {
                "addiw" | "slliw" | "srliw" | "sraiw" => {
                    if ops.len() != 3 { return Some(Err("用法: addiw rd, rs1, imm".into())); }
                    let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let rs1 = match parse_register(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let imm = match parse_int(&ops[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let imm_bits = match imm_signed_bits(imm, 12) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
                    let inst = match mnem {
                        "addiw" => RV64I::Addiw(i).into(),
                        "slliw" => RV64I::Slliw(i).into(),
                        "srliw" => RV64I::Srliw(i).into(),
                        "sraiw" => RV64I::Sraiw(i).into(),
                        _ => unreachable!(),
                    };
                    Some(Ok(inst))
                }
                _ => {
                    if ops.len() != 3 { return Some(Err("用法: addw rd, rs1, rs2".into())); }
                    let rd = match parse_register(&ops[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let rs1 = match parse_register(&ops[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let rs2 = match parse_register(&ops[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                    let r = RType { rd, rs1, rs2, funct3: 0, funct7: 0 };
                    let inst = match mnem {
                        "addw" => RV64I::Addw(r).into(),
                        "subw" => RV64I::Subw(r).into(),
                        "sllw" => RV64I::Sllw(r).into(),
                        "srlw" => RV64I::Srlw(r).into(),
                        "sraw" => RV64I::Sraw(r).into(),
                        _ => unreachable!(),
                    };
                    Some(Ok(inst))
                }
            }
        }
        _ => None,
    }
}
