use crate::{asm::*, riscv::imm::Xlen};
use crate::isa::*;

fn ensure_align(imm: u32, align: u32) -> Result<(), String> {
    if (imm & (align - 1)) == 0 { Ok(()) } else { Err(format!("立即数未按 {} 字节对齐", align)) }
}

fn c_reg_index(reg: u8) -> Result<u16, String> {
    if (8..=15).contains(&reg) { Ok((reg - 8) as u16) } else { Err(format!("寄存器 {} 不是压缩寄存器 (x8..x15)", reg)) }
}

#[inline]
fn set_bits(val: u16, pos: u8, width: u8, field: u16) -> u16 {
    let mask = (((1u32 << width) - 1) as u16) << pos;
    (val & !mask) | ((field & (((1u16 << width) - 1))) << pos)
}

fn encode_ci_addi_like(rd: u8, imm6: u32, funct3: u16) -> Result<u16, String> {
    // CI format (addi/li): opcode=01, bits: [15:13]=funct3, [12]=imm[5], [11:7]=rd, [6:2]=imm[4:0]
    let imm = imm6 & 0x3f; // two's complement already in Imm
    let mut ins: u16 = 0;
    ins = set_bits(ins, 13, 3, funct3);
    ins = set_bits(ins, 12, 1, ((imm >> 5) & 0x1) as u16);
    ins = set_bits(ins, 7, 5, rd as u16);
    ins = set_bits(ins, 2, 5, (imm & 0x1f) as u16);
    ins = set_bits(ins, 0, 2, OPCODE_C1 as u16);
    Ok(ins)
}

fn encode_ciw_addi4spn(rd: u8, uimm: u32) -> Result<u16, String> {
    // CIW (c.addi4spn): opcode=00, funct3=000, rd' in bits [4:2],
    // uimm mapping (10 bits, non-zero): [5:4] -> [12:11], [9:6] -> [10:7], [2] -> [6], [3] -> [5]
    if uimm == 0 { return Err("c.addi4spn 的 uimm 不能为 0".into()); }
    if uimm >= (1 << 10) { return Err("c.addi4spn 的 uimm 超出 10 位范围".into()); }
    let rd_c = c_reg_index(rd)?; // rd must be x8..x15
    let mut ins: u16 = 0;
    // funct3 = 000
    ins = set_bits(ins, 13, 3, 0b000);
    // uimm[5:4] -> bits 12:11
    ins = set_bits(ins, 11, 2, ((uimm >> 4) & 0x3) as u16);
    // uimm[9:6] -> bits 10:7
    ins = set_bits(ins, 7, 4, ((uimm >> 6) & 0xF) as u16);
    // uimm[3] -> bit 5
    ins = set_bits(ins, 5, 1, ((uimm >> 3) & 0x1) as u16);
    // uimm[2] -> bit 6
    ins = set_bits(ins, 6, 1, ((uimm >> 2) & 0x1) as u16);
    // rd' -> bits 4:2
    ins = set_bits(ins, 2, 3, rd_c);
    // opcode = 00
    ins = set_bits(ins, 0, 2, OPCODE_C0 as u16);
    Ok(ins)
}

fn encode_clw(rd: u8, rs1: u8, imm: u32) -> Result<u16, String> {
    // C.LW: opcode=00, funct3=010; rd'=x8..x15, rs1'=x8..x15; imm uses bits [6|5:3|2]
    if imm >= 128 { return Err("c.lw 偏移过大 (需 < 128)".into()); }
    ensure_align(imm, 4)?;
    let rd_c = c_reg_index(rd)?;
    let rs1_c = c_reg_index(rs1)?;
    let mut ins: u16 = 0;
    // funct3
    ins = set_bits(ins, 13, 3, 0b010);
    // imm[5:3] -> bits 13:11 already used; according to decode it used (ins>>11)&0b111; that is bits 13..11
    ins = set_bits(ins, 11, 3, ((imm >> 3) & 0x7) as u16);
    // rs1' -> bits 9:7
    ins = set_bits(ins, 7, 3, rs1_c);
    // imm[6] -> bit 5
    ins = set_bits(ins, 5, 1, ((imm >> 6) & 0x1) as u16);
    // imm[2] -> bit 6
    ins = set_bits(ins, 6, 1, ((imm >> 2) & 0x1) as u16);
    // rd' -> bits 4:2
    ins = set_bits(ins, 2, 3, rd_c);
    // opcode 00
    ins = set_bits(ins, 0, 2, OPCODE_C0 as u16);
    Ok(ins)
}

fn encode_csw(rs2: u8, rs1: u8, imm: u32) -> Result<u16, String> {
    // C.SW: opcode=00, funct3=110; rs2'=x8..x15, rs1'=x8..x15; imm uses [6|5:3|2]
    if imm >= 128 { return Err("c.sw 偏移过大 (需 < 128)".into()); }
    ensure_align(imm, 4)?;
    let rs2_c = c_reg_index(rs2)?;
    let rs1_c = c_reg_index(rs1)?;
    let mut ins: u16 = 0;
    ins = set_bits(ins, 13, 3, 0b110);
    // imm[5:3] -> bits 13:11
    ins = set_bits(ins, 11, 3, ((imm >> 3) & 0x7) as u16);
    // rs1' -> bits 9:7
    ins = set_bits(ins, 7, 3, rs1_c);
    // imm[6] -> bit 5
    ins = set_bits(ins, 5, 1, ((imm >> 6) & 0x1) as u16);
    // imm[2] -> bit 6
    ins = set_bits(ins, 6, 1, ((imm >> 2) & 0x1) as u16);
    // rs2' -> bits 4:2
    ins = set_bits(ins, 2, 3, rs2_c);
    // opcode
    ins = set_bits(ins, 0, 2, OPCODE_C0 as u16);
    Ok(ins)
}

fn encode_clwsp(rd: u8, imm: u32) -> Result<u16, String> {
    // C.LWSP: opcode=10, funct3=010; rd!=x0; imm uses [5|4:2|7:6]
    if rd == 0 { return Err("c.lwsp 目的寄存器不能为 x0".into()); }
    if imm >= 256 { return Err("c.lwsp 偏移过大 (需 < 256)".into()); }
    ensure_align(imm, 4)?;
    let mut ins: u16 = 0;
    ins = set_bits(ins, 13, 3, 0b010);
    // imm[5] -> bit 12
    ins = set_bits(ins, 12, 1, ((imm >> 5) & 0x1) as u16);
    // rd -> bits 11:7
    ins = set_bits(ins, 7, 5, rd as u16);
    // imm[4:2] -> bits 6:4
    ins = set_bits(ins, 4, 3, ((imm >> 2) & 0x7) as u16);
    // imm[7:6] -> bits 3:2
    ins = set_bits(ins, 2, 2, ((imm >> 6) & 0x3) as u16);
    // opcode
    ins = set_bits(ins, 0, 2, OPCODE_C2 as u16);
    Ok(ins)
}

fn encode_cswsp(rs2: u8, imm: u32) -> Result<u16, String> {
    // C.SWSP: opcode=10, funct3=110; base=sp
    if imm >= 256 { return Err("c.swsp 偏移过大 (需 < 256)".into()); }
    ensure_align(imm, 4)?;
    let mut ins: u16 = 0;
    ins = set_bits(ins, 13, 3, 0b110);
    // imm[5:2] -> bits 12:9 (4 bits)
    ins = set_bits(ins, 9, 4, ((imm >> 3) & 0xF) as u16);
    // imm[7:6] -> bits 8:7 (2 bits)
    ins = set_bits(ins, 7, 2, ((imm >> 6) & 0x3) as u16);
    // rs2 -> bits 6:2
    ins = set_bits(ins, 2, 5, rs2 as u16);
    // opcode
    ins = set_bits(ins, 0, 2, OPCODE_C2 as u16);
    Ok(ins)
}

fn encode_cld(rd: u8, rs1: u8, imm: u32) -> Result<u16, String> {
    // C.LD: opcode=00, funct3=011; rd', rs1' compressed; imm uses [7:6|5:3]
    if imm >= 256 { return Err("c.ld 偏移过大 (需 < 256)".into()); }
    ensure_align(imm, 8)?;
    let rd_c = c_reg_index(rd)?;
    let rs1_c = c_reg_index(rs1)?;
    let mut ins: u16 = 0;
    ins = set_bits(ins, 13, 3, 0b011);
    // imm[5:3] -> bits 12:10 (use (ins>>10)&0b111)
    ins = set_bits(ins, 10, 3, ((imm >> 3) & 0x7) as u16);
    // rs1' -> bits 9:7
    ins = set_bits(ins, 7, 3, rs1_c);
    // imm[7:6] -> bits 6:5
    ins = set_bits(ins, 5, 2, ((imm >> 6) & 0x3) as u16);
    // rd' -> bits 4:2
    ins = set_bits(ins, 2, 3, rd_c);
    ins = set_bits(ins, 0, 2, OPCODE_C0 as u16);
    Ok(ins)
}

fn encode_csd(rs2: u8, rs1: u8, imm: u32) -> Result<u16, String> {
    // C.SD: opcode=00, funct3=111; rs2', rs1' compressed; imm uses [7:6|5:3]
    if imm >= 256 { return Err("c.sd 偏移过大 (需 < 256)".into()); }
    ensure_align(imm, 8)?;
    let rs2_c = c_reg_index(rs2)?;
    let rs1_c = c_reg_index(rs1)?;
    let mut ins: u16 = 0;
    ins = set_bits(ins, 13, 3, 0b111);
    // imm[5:3] -> bits 12:10
    ins = set_bits(ins, 10, 3, ((imm >> 3) & 0x7) as u16);
    // rs1' -> bits 9:7
    ins = set_bits(ins, 7, 3, rs1_c);
    // imm[7:6] -> bits 6:5
    ins = set_bits(ins, 5, 2, ((imm >> 6) & 0x3) as u16);
    // rs2' -> bits 4:2
    ins = set_bits(ins, 2, 3, rs2_c);
    ins = set_bits(ins, 0, 2, OPCODE_C0 as u16);
    Ok(ins)
}

fn encode_cldsp(rd: u8, imm: u32) -> Result<u16, String> {
    // C.LDSP: opcode=10, funct3=011; rd!=x0; imm uses [5|7:6|4:3]
    if rd == 0 { return Err("c.ldsp 目的寄存器不能为 x0".into()); }
    if imm >= 512 { return Err("c.ldsp 偏移过大".into()); }
    ensure_align(imm, 8)?;
    let mut ins: u16 = 0;
    ins = set_bits(ins, 13, 3, 0b011);
    // imm[5] -> bit 12
    ins = set_bits(ins, 12, 1, ((imm >> 5) & 0x1) as u16);
    // rd -> bits 11:7
    ins = set_bits(ins, 7, 5, rd as u16);
    // imm[7:6] -> bits 6:5
    ins = set_bits(ins, 5, 2, ((imm >> 6) & 0x3) as u16);
    // imm[4:3] -> bits 4:3
    ins = set_bits(ins, 3, 2, ((imm >> 3) & 0x3) as u16);
    ins = set_bits(ins, 0, 2, OPCODE_C2 as u16);
    Ok(ins)
}

fn encode_csdsp(rs2: u8, imm: u32) -> Result<u16, String> {
    // C.SDSP: opcode=10, funct3=111; base=sp; imm uses [5:3|8:6]
    if imm >= 512 { return Err("c.sdsp 偏移过大".into()); }
    ensure_align(imm, 8)?;
    let mut ins: u16 = 0;
    ins = set_bits(ins, 13, 3, 0b111);
    // imm[5:3] -> bits 12:10
    ins = set_bits(ins, 10, 3, ((imm >> 3) & 0x7) as u16);
    // imm[8:6] -> bits 9:7
    ins = set_bits(ins, 7, 3, ((imm >> 6) & 0x7) as u16);
    // rs2 -> bits 6:2
    ins = set_bits(ins, 2, 5, rs2 as u16);
    ins = set_bits(ins, 0, 2, OPCODE_C2 as u16);
    Ok(ins)
}

fn encode_rvc(rvc: &RVC, xlen: Xlen) -> Result<u16, String> {
    use RVC::*;
    match rvc {
        Caddi4spn(ciw) => encode_ciw_addi4spn(ciw.rd, ciw.uimm.low32()),
        // CI-like
        Cnop(ci) => encode_ci_addi_like(ci.rdrs1, ci.imm.low_u32(), 0b000),
        Caddi(ci) => encode_ci_addi_like(ci.rdrs1, ci.imm.low_u32(), 0b000),
        Cli(ci)   => encode_ci_addi_like(ci.rdrs1, ci.imm.low_u32(), 0b010),

        // CL/CS (with c-registers)
        Clw(cl) => encode_clw(cl.rd, cl.rs1, cl.imm.low_u32()),
        Csw(cs) => encode_csw(cs.rs2, cs.rs1, cs.imm.low_u32()),

        // SP-relative
        Clwsp(ci) => encode_clwsp(ci.rdrs1, ci.imm.low_u32()),
        Cswsp(css)=> encode_cswsp(css.rs2, css.imm.low_u32()),

        // RV64-only loads/stores
        Cld(cl) if matches!(xlen, Xlen::X64 | Xlen::X128) => encode_cld(cl.rd, cl.rs1, cl.imm.low_u32()),
        Csd(cs) if matches!(xlen, Xlen::X64 | Xlen::X128) => encode_csd(cs.rs2, cs.rs1, cs.imm.low_u32()),
        Cldsp(ci) if matches!(xlen, Xlen::X64 | Xlen::X128) => encode_cldsp(ci.rdrs1, ci.imm.low_u32()),
        Csdsp(css) if matches!(xlen, Xlen::X64 | Xlen::X128) => encode_csdsp(css.rs2, css.imm.low_u32()),

        // Unimplemented compressed ops (future)
        _ => Err("该 RVC 指令的编码暂未实现".into()),
    }
}

pub fn encode_u16(inst: &Instruction, xlen: Xlen) -> Result<u16, String> {
    match inst {
        Instruction::RVC(rvc) => encode_rvc(rvc, xlen),
        _ => Err("非压缩指令，不能使用 16 位编码".into()),
    }
}
