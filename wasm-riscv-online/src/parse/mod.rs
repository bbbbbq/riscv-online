use crate::asm::*;
use crate::asm; // for from_register()
use crate::riscv::imm::{Imm, Uimm, Xlen};

fn trim_comment(s: &str) -> &str {
    let s = s.split("//").next().unwrap_or(s);
    let s = s.split('#').next().unwrap_or(s);
    s
}

fn split_operands(s: &str) -> Vec<String> {
    s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
}

fn parse_register(s: &str) -> Result<u8, String> {
    if let Some(r) = asm::from_register(s) { Ok(r) } else { Err(format!("未知寄存器: {}", s)) }
}

fn parse_int(s: &str) -> Result<i64, String> {
    let t = s.trim();
    if t.is_empty() { return Err("缺少立即数".into()); }
    let (sign, body) = if let Some(rest) = t.strip_prefix('-') { (-1i64, rest) } else if let Some(rest) = t.strip_prefix('+') { (1i64, rest) } else { (1i64, t) };
    if body.starts_with("0x") || body.starts_with("0X") {
        match i64::from_str_radix(&body[2..], 16) {
            Ok(v) => Ok(sign * v),
            Err(e) => Err(format!("立即数解析失败: {}", e)),
        }
    } else {
        match body.parse::<i64>() {
            Ok(v) => Ok(sign * v),
            Err(e) => {
                // 如果看起来像是寄存器名，给出更清晰的提示
                if asm::from_register(t).is_some() {
                    Err(format!("期望立即数，但提供了寄存器: {}。对于 addi 这类指令，第 3 个参数应为立即数。", t))
                } else {
                    Err(format!("立即数解析失败: {}", e))
                }
            }
        }
    }
}

fn imm_signed_bits(value: i64, bits: u8) -> Result<u32, String> {
    let min = -(1i64 << (bits - 1));
    let max = (1i64 << (bits - 1)) - 1;
    if value < min || value > max { return Err(format!("立即数超出范围: {} ({} 位)", value, bits)); }
    if value >= 0 { Ok(value as u32) } else { Ok(((1i64 << bits) + value) as u32) }
}

fn parse_mem_operand(s: &str) -> Result<(u32, u8), String> {
    // format: imm(rs)
    let open = s.find('(').ok_or_else(|| format!("内存操作数格式错误: {}", s))?;
    let close = s.find(')').ok_or_else(|| format!("内存操作数格式错误: {}", s))?;
    let imm_str = s[..open].trim();
    let reg_str = &s[open + 1..close];
    let imm = parse_int(imm_str)?;
    let rs = parse_register(reg_str)?;
    Ok((imm_signed_bits(imm, 12)?, rs))
}

pub fn parse_line(line: &str, xlen: Xlen) -> Result<Instruction, String> {
    let raw = trim_comment(line).trim();
    if raw.is_empty() { return Err("空行".into()); }
    // split mnemonic and operands
    let mut parts = raw.split_whitespace();
    let mnem = parts.next().ok_or_else(|| "缺少助记符".to_string())?.to_lowercase();
    let rest = parts.collect::<Vec<_>>().join(" ");
    let ops = if rest.is_empty() { vec![] } else { split_operands(&rest) };

    match mnem.as_str() {
        // U-type
        "lui" => {
            if ops.len() != 2 { return Err("用法: lui rd, imm20".into()); }
            let rd = parse_register(&ops[0])?;
            let imm = parse_int(&ops[1])?;
            // imm20 placed at [31:12]
            let imm_bits = ((imm as i64) as u32) << 12;
            let u = UType { rd, imm: Imm::new(imm_bits, 32) };
            Ok(RV32I::Lui(u).into())
        }
        "auipc" => {
            if ops.len() != 2 { return Err("用法: auipc rd, imm20".into()); }
            let rd = parse_register(&ops[0])?;
            let imm = parse_int(&ops[1])?;
            let imm_bits = ((imm as i64) as u32) << 12;
            let u = UType { rd, imm: Imm::new(imm_bits, 32) };
            Ok(RV32I::Auipc(u).into())
        }

        // J-type
        "jal" => {
            if ops.len() != 2 { return Err("用法: jal rd, imm".into()); }
            let rd = parse_register(&ops[0])?;
            let imm = parse_int(&ops[1])?; // bytes offset
            let imm_bits = imm_signed_bits(imm, 21)?;
            // decoder currently uses 12-bit valid width for J-type Imm; keep consistent
            let j = JType { rd, imm: Imm::new(imm_bits, 12) };
            Ok(RV32I::Jal(j).into())
        }
        // JALR: 支持两种写法: jalr rd, imm(rs1) 或 jalr rd, rs1, imm
        "jalr" => {
            if ops.len() == 2 {
                let rd = parse_register(&ops[0])?;
                let (imm_bits, rs1) = parse_mem_operand(&ops[1])?;
                let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
                Ok(RV32I::Jalr(i).into())
            } else if ops.len() == 3 {
                let rd = parse_register(&ops[0])?;
                let rs1 = parse_register(&ops[1])?;
                let imm = parse_int(&ops[2])?;
                let imm_bits = imm_signed_bits(imm, 12)?;
                let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
                Ok(RV32I::Jalr(i).into())
            } else { Err("用法: jalr rd, imm(rs1) 或 jalr rd, rs1, imm".into()) }
        }

        // B-type
        "beq" | "bne" | "blt" | "bge" | "bltu" | "bgeu" => {
            if ops.len() != 3 { return Err("用法: beq rs1, rs2, imm".into()); }
            let rs1 = parse_register(&ops[0])?;
            let rs2 = parse_register(&ops[1])?;
            let imm = parse_int(&ops[2])?; // bytes offset (must be 2-byte aligned)
            if (imm & 1) != 0 { return Err("分支偏移必须是2字节对齐".into()); }
            let imm_bits = imm_signed_bits(imm, 13)?;
            // decoder uses 12-bit valid width for B-type Imm
            let b = BType { rs1, rs2, funct3: 0, imm: Imm::new(imm_bits, 12) };
            let inst = match mnem.as_str() {
                "beq" => RV32I::Beq(b).into(),
                "bne" => RV32I::Bne(b).into(),
                "blt" => RV32I::Blt(b).into(),
                "bge" => RV32I::Bge(b).into(),
                "bltu" => RV32I::Bltu(b).into(),
                "bgeu" => RV32I::Bgeu(b).into(),
                _ => unreachable!(),
            };
            Ok(inst)
        }

        // Loads
        "lb" | "lh" | "lw" | "lbu" | "lhu" | "lwu" | "ld" => {
            if ops.len() != 2 { return Err("用法: lw rd, imm(rs1)".into()); }
            let rd = parse_register(&ops[0])?;
            let (imm_bits, rs1) = parse_mem_operand(&ops[1])?;
            let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
            let inst = match mnem.as_str() {
                "lb" => RV32I::Lb(i).into(),
                "lh" => RV32I::Lh(i).into(),
                "lw" => RV32I::Lw(i).into(),
                "lbu" => RV32I::Lbu(i).into(),
                "lhu" => RV32I::Lhu(i).into(),
                "lwu" => match xlen { Xlen::X64 | Xlen::X128 => RV64I::Lwu(i).into(), _ => return Err("lwu 仅在 RV64/128 可用".into()) },
                "ld"  => match xlen { Xlen::X64 | Xlen::X128 => RV64I::Ld(i).into(), _ => return Err("ld 仅在 RV64/128 可用".into()) },
                _ => unreachable!(),
            };
            Ok(inst)
        }

        // Stores
        "sb" | "sh" | "sw" | "sd" => {
            if ops.len() != 2 { return Err("用法: sw rs2, imm(rs1)".into()); }
            let rs2 = parse_register(&ops[0])?;
            let (imm_bits, rs1) = parse_mem_operand(&ops[1])?;
            let s = SType { rs1, rs2, funct3: 0, imm: Imm::new(imm_bits, 12) };
            let inst = match mnem.as_str() {
                "sb" => RV32I::Sb(s).into(),
                "sh" => RV32I::Sh(s).into(),
                "sw" => RV32I::Sw(s).into(),
                "sd" => match xlen { Xlen::X64 | Xlen::X128 => RV64I::Sd(s).into(), _ => return Err("sd 仅在 RV64/128 可用".into()) },
                _ => unreachable!(),
            };
            Ok(inst)
        }

        // OP-IMM
        "addi" | "slti" | "sltiu" | "xori" | "ori" | "andi" => {
            if ops.len() != 3 { return Err("用法: addi rd, rs1, imm".into()); }
            let rd = parse_register(&ops[0])?;
            let rs1 = parse_register(&ops[1])?;
            let imm = parse_int(&ops[2])?;
            let imm_bits = imm_signed_bits(imm, 12)?;
            let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
            let inst = match mnem.as_str() {
                "addi" => RV32I::Addi(i).into(),
                "slti" => RV32I::Slti(i).into(),
                "sltiu" => RV32I::Sltiu(i).into(),
                "xori" => RV32I::Xori(i).into(),
                "ori" => RV32I::Ori(i).into(),
                "andi" => RV32I::Andi(i).into(),
                _ => unreachable!(),
            };
            Ok(inst)
        }
        // shifts immediate (xlen-sensitive)
        "slli" | "srli" | "srai" => {
            if ops.len() != 3 { return Err("用法: slli rd, rs1, shamt".into()); }
            let rd = parse_register(&ops[0])?;
            let rs1 = parse_register(&ops[1])?;
            let shamt = parse_int(&ops[2])?;
            let bits = match xlen { Xlen::X32 => 5, Xlen::X64 | Xlen::X128 => 6 };
            let shamt_bits = imm_signed_bits(shamt, bits)?; // positive expected
            let i = IType { rd, rs1, funct3: 0, imm: Imm::new(shamt_bits, bits) };
            let inst = match (mnem.as_str(), xlen) {
                ("slli", Xlen::X32) => RV32I::Slli(i).into(),
                ("srli", Xlen::X32) => RV32I::Srli(i).into(),
                ("srai", Xlen::X32) => RV32I::Srai(i).into(),
                ("slli", _) => RV64I::Slli(i).into(),
                ("srli", _) => RV64I::Srli(i).into(),
                ("srai", _) => RV64I::Srai(i).into(),
                _ => unreachable!(),
            };
            Ok(inst)
        }

        // OP (R-type)
        "add" | "sub" | "sll" | "slt" | "sltu" | "xor" | "srl" | "sra" | "or" | "and" => {
            if ops.len() != 3 { return Err("用法: add rd, rs1, rs2".into()); }
            let rd = parse_register(&ops[0])?;
            let rs1 = parse_register(&ops[1])?;
            let rs2 = parse_register(&ops[2])?;
            let r = RType { rd, rs1, rs2, funct3: 0, funct7: 0 };
            let inst = match mnem.as_str() {
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
            Ok(inst)
        }

        // RV64I W variants
        "addiw" | "slliw" | "srliw" | "sraiw" | "addw" | "subw" | "sllw" | "srlw" | "sraw" => {
            match xlen { Xlen::X64 | Xlen::X128 => {}, _ => return Err("该指令仅在 RV64/128 可用".into()) }
            match mnem.as_str() {
                "addiw" | "slliw" | "srliw" | "sraiw" => {
                    if ops.len() != 3 { return Err("用法: addiw rd, rs1, imm".into()); }
                    let rd = parse_register(&ops[0])?;
                    let rs1 = parse_register(&ops[1])?;
                    let imm = parse_int(&ops[2])?;
                    let imm_bits = imm_signed_bits(imm, 12)?;
                    let i = IType { rd, rs1, funct3: 0, imm: Imm::new(imm_bits, 12) };
                    let inst = match mnem.as_str() {
                        "addiw" => RV64I::Addiw(i).into(),
                        "slliw" => RV64I::Slliw(i).into(),
                        "srliw" => RV64I::Srliw(i).into(),
                        "sraiw" => RV64I::Sraiw(i).into(),
                        _ => unreachable!(),
                    };
                    Ok(inst)
                }
                _ => {
                    if ops.len() != 3 { return Err("用法: addw rd, rs1, rs2".into()); }
                    let rd = parse_register(&ops[0])?;
                    let rs1 = parse_register(&ops[1])?;
                    let rs2 = parse_register(&ops[2])?;
                    let r = RType { rd, rs1, rs2, funct3: 0, funct7: 0 };
                    let inst = match mnem.as_str() {
                        "addw" => RV64I::Addw(r).into(),
                        "subw" => RV64I::Subw(r).into(),
                        "sllw" => RV64I::Sllw(r).into(),
                        "srlw" => RV64I::Srlw(r).into(),
                        "sraw" => RV64I::Sraw(r).into(),
                        _ => unreachable!(),
                    };
                    Ok(inst)
                }
            }
        }

        // System
        "ecall" => Ok(RV32I::Ecall(()).into()),
        "ebreak" => Ok(RV32I::Ebreak(()).into()),
        "fence" => Ok(RV32I::Fence(()).into()),
        "fence.i" | "fencei" => Ok(RV32I::FenceI(()).into()),

        // CSR (Zicsr)
        "csrrw" | "csrrs" | "csrrc" => {
            if ops.len() != 3 { return Err("用法: csrr{w|s|c} rd, csr, rs1".into()); }
            let rd = parse_register(&ops[0])?;
            let csr_val = parse_int(&ops[1])?;
            if csr_val < 0 || csr_val > 0xFFF { return Err("csr 编号应为 0..0xFFF".into()); }
            let rs1 = parse_register(&ops[2])?;
            let c = CsrRType { rd, rs1, funct3: 0, csr: csr_val as u16 };
            let inst = match mnem.as_str() {
                "csrrw" => RVZicsr::Csrrw(c).into(),
                "csrrs" => RVZicsr::Csrrs(c).into(),
                "csrrc" => RVZicsr::Csrrc(c).into(),
                _ => unreachable!(),
            };
            Ok(inst)
        }
        "csrrwi" | "csrrsi" | "csrrci" => {
            if ops.len() != 3 { return Err("用法: csrr{x}i rd, csr, uimm".into()); }
            let rd = parse_register(&ops[0])?;
            let csr_val = parse_int(&ops[1])?;
            if csr_val < 0 || csr_val > 0xFFF { return Err("csr 编号应为 0..0xFFF".into()); }
            let uimm_val = parse_int(&ops[2])?;
            if uimm_val < 0 || uimm_val > 31 { return Err("uimm 取值 0..31".into()); }
            let c = CsrIType { rd, uimm: Uimm::new(uimm_val as u32, 5), funct3: 0, csr: csr_val as u16 };
            let inst = match mnem.as_str() {
                "csrrwi" => RVZicsr::Csrrwi(c).into(),
                "csrrsi" => RVZicsr::Csrrsi(c).into(),
                "csrrci" => RVZicsr::Csrrci(c).into(),
                _ => unreachable!(),
            };
            Ok(inst)
        }

        _ => Err(format!("未支持的指令: {}", mnem)),
    }
}
