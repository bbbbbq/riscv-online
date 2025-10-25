use crate::{asm::Instruction,asm::*, riscv::imm::{Imm, Uimm, Xlen}};
use crate::isa::*;

pub fn resolve_u32(ins: u32, xlen: Xlen) -> core::result::Result<Instruction, ()> {
    use crate::asm::{RVZicsr::*, RV32I::*, RV64I::*, RVF::*, RV32A::*,RV64A::*, RV128A::*};
    let opcode = ins & 0b111_1111;
    let rd = ((ins >> 7) & 0b1_1111) as u8;
    let rs1 = ((ins >> 15) & 0b1_1111) as u8;
    let rs2 = ((ins >> 20) & 0b1_1111) as u8;
    let funct3 = ((ins >> 12) & 0b111) as u8;
    let funct5 = ((ins >> 27) & 0b11111) as u8;
    let funct7 = ((ins >> 25) & 0b111_1111) as u8;
    let funct12 = (ins >> 20) & 0b1111_1111_1111;
    let rs3 = ((ins >> 27) & 0b1_1111) as u8;
    let funct2 = ((ins >> 25) & 0b11) as u8;
    let imm_i = {
        let val = (ins >> 20) & 0b1111_1111_1111;
        Imm::new(val, 12)
    };
    let imm_s = {
        let val = ((ins >> 7) & 0b11111) | (((ins >> 25) & 0b1111111) << 5);
        Imm::new(val, 12)
    };
    let imm_b = {
        let val = (((ins >> 7) & 0b1) << 11)
            | (((ins >> 8) & 0b1111) << 1)
            | (((ins >> 25) & 0b111111) << 5)
            | (((ins >> 31) & 0b1) << 12);
        Imm::new(val, 12)
    };
    let imm_u = Imm::new(ins & 0xFFFFF000, 32);
    let imm_j = {
        let val = (((ins & 0b1000_0000_0000_0000_0000_0000_0000_0000) >> 31) << 20)
            | (((ins & 0b0111_1111_1110_0000_0000_0000_0000_0000) >> 21) << 1)
            | (((ins & 0b0000_0000_0001_0000_0000_0000_0000_0000) >> 20) << 11)
            | (((ins & 0b0000_0000_0000_1111_1111_0000_0000_0000) >> 12) << 12);
        Imm::new(val, 12)
    };
    let uimm_csr = Uimm::new((ins >> 15) & 0b11111, 5);
    let csr = ((ins >> 20) & 0xFFF) as u16;
    let u_type = UType { rd, imm: imm_u };
    let j_type = JType { rd, imm: imm_j };
    let b_type = BType {
        rs1,
        rs2,
        funct3,
        imm: imm_b,
    };
    let i_type = IType {
        rd,
        rs1,
        funct3,
        imm: imm_i,
    };
    let s_type = SType {
        rs1,
        rs2,
        funct3,
        imm: imm_s,
    };
    let r_type = RType {
        rd,
        rs1,
        rs2,
        funct3,
        funct7,
    };
    let csr_r_type = CsrRType {
        rd,
        rs1,
        funct3,
        csr,
    };
    let csr_i_type = CsrIType {
        rd,
        uimm: uimm_csr,
        funct3,
        csr,
    };
    let r4_type = R4Type {
        rd,
        rs1,
        rs2,
        rs3,
        funct3,
        funct2,
    };
    let ans = match opcode {
        OPCODE_LUI => Lui(u_type).into(),
        OPCODE_AUIPC => Auipc(u_type).into(),
        OPCODE_JAL => Jal(j_type).into(),
        OPCODE_JALR => Jalr(i_type).into(),
        OPCODE_BRANCH => match funct3 {
            FUNCT3_BRANCH_BEQ => Beq(b_type).into(),
            FUNCT3_BRANCH_BNE => Bne(b_type).into(),
            FUNCT3_BRANCH_BLT => Blt(b_type).into(),
            FUNCT3_BRANCH_BGE => Bge(b_type).into(),
            FUNCT3_BRANCH_BLTU => Bltu(b_type).into(),
            FUNCT3_BRANCH_BGEU => Bgeu(b_type).into(),
            _ => Err(())?,
        },
        OPCODE_LOAD => match funct3 {
            FUNCT3_LOAD_LB => Lb(i_type).into(),
            FUNCT3_LOAD_LH => Lh(i_type).into(),
            FUNCT3_LOAD_LW => Lw(i_type).into(),
            FUNCT3_LOAD_LD if xlen != Xlen::X32 => Ld(i_type).into(),
            FUNCT3_LOAD_LBU => Lbu(i_type).into(),
            FUNCT3_LOAD_LHU => Lhu(i_type).into(),
            FUNCT3_LOAD_LWU if xlen != Xlen::X32 => Lwu(i_type).into(),
            _ => Err(())?,
        },
        OPCODE_STORE => match funct3 {
            FUNCT3_STORE_SB => Sb(s_type).into(),
            FUNCT3_STORE_SH => Sh(s_type).into(),
            FUNCT3_STORE_SW => Sw(s_type).into(),
            FUNCT3_STORE_SD if xlen != Xlen::X32 => Sd(s_type).into(),
            _ => Err(())?,
        },
        OPCODE_MISC_MEM => match funct3 {
            FUNCT3_MISC_MEM_FENCE => Fence(()).into(),
            FUNCT3_MISC_MEM_FENCE_I => FenceI(()).into(),
            _ => Err(())?,
        },
        OPCODE_SYSTEM => match funct3 {
            FUNCT3_SYSTEM_PRIV => match funct12 {
                FUNCT12_SYSTEM_ECALL if funct3 == FUNCT3_SYSTEM_PRIV && rs1 == 0 && rd == 0 => {
                    Ecall(()).into()
                }
                FUNCT12_SYSTEM_EBREAK if funct3 == FUNCT3_SYSTEM_PRIV && rs1 == 0 && rd == 0 => {
                    Ebreak(()).into()
                }
                _ => Err(())?,
            },
            FUNCT3_SYSTEM_CSRRW => Csrrw(csr_r_type).into(),
            FUNCT3_SYSTEM_CSRRS => Csrrs(csr_r_type).into(),
            FUNCT3_SYSTEM_CSRRC => Csrrc(csr_r_type).into(),
            FUNCT3_SYSTEM_CSRRWI => Csrrwi(csr_i_type).into(),
            FUNCT3_SYSTEM_CSRRSI => Csrrsi(csr_i_type).into(),
            FUNCT3_SYSTEM_CSRRCI => Csrrci(csr_i_type).into(),
            _ => Err(())?,
        },
        OPCODE_OP_IMM => match funct3 {
            FUNCT3_OP_ADD_SUB => Addi(i_type).into(),
            FUNCT3_OP_SLT => Slti(i_type).into(),
            FUNCT3_OP_SLTU => Sltiu(i_type).into(),
            FUNCT3_OP_XOR => Xori(i_type).into(),
            FUNCT3_OP_OR => Ori(i_type).into(),
            FUNCT3_OP_AND => Andi(i_type).into(),
            FUNCT3_OP_SLL if funct7 == 0 && xlen == Xlen::X32 => RV32I::Slli(i_type).into(),
            FUNCT3_OP_SLL if funct7 & 0b1111110 == 0 && xlen == Xlen::X64 => {
                RV64I::Slli(i_type).into()
            }
            FUNCT3_OP_SRL_SRA => match funct7 {
                FUNCT7_OP_SRL if xlen == Xlen::X32 => RV32I::Srli(i_type).into(),
                FUNCT7_OP_SRA if xlen == Xlen::X32 => RV32I::Srai(i_type).into(),
                x if x & 0b1111110 == FUNCT7_OP_SRL && xlen == Xlen::X64 => {
                    RV64I::Srli(i_type).into()
                }
                x if x & 0b1111110 == FUNCT7_OP_SRA && xlen == Xlen::X64 => {
                    RV64I::Srai(i_type).into()
                }
                _ => Err(())?,
            },
            _ => Err(())?,
        },
        OPCODE_OP => match funct3 {
            FUNCT3_OP_ADD_SUB => match funct7 {
                FUNCT7_OP_ADD => Add(r_type).into(),
                FUNCT7_OP_SUB => Sub(r_type).into(),
                0b000_0001 => Mul(r_type).into(),
                _ => Err(())?,
            },
            FUNCT3_OP_SLL => match funct7 {
                0 => RV32I::Sll(r_type).into(),
                0b000_0001 => Mulh(r_type).into(),
                _ => Err(())?,
            },
            FUNCT3_OP_SLT if funct7 == 0 => Slt(r_type).into(),
            FUNCT3_OP_SLTU if funct7 == 0 => Sltu(r_type).into(),
            FUNCT3_OP_XOR => match funct7 {
                0 => Xor(r_type).into(),
                0b000_0001 => Mulhsu(r_type).into(),
                _ => Err(())?,
            },
            FUNCT3_OP_SRL_SRA => match funct7 {
                0 => RV32I::Srl(r_type).into(),
                0b010_0000 => RV32I::Sra(r_type).into(),
                0b000_0001 => Div(r_type).into(),
                _ => Err(())?,
            },
            FUNCT3_OP_OR => match funct7 {
                0 => Or(r_type).into(),
                0b000_0001 => Divu(r_type).into(),
                _ => Err(())?,
            },
            FUNCT3_OP_AND => match funct7 {
                0 => And(r_type).into(),
                0b000_0001 if xlen == Xlen::X32 => Rem(r_type).into(),
                0b000_0001 if xlen == Xlen::X64 => Remu(r_type).into(),
                _ => Err(())?,
            },
            _ => Err(())?,
        },
        OPCODE_OP_IMM32 if xlen == Xlen::X64 => match funct3 {
            FUNCT3_OP_ADD_SUB => Addiw(i_type).into(),
            FUNCT3_OP_SLL if funct7 == 0 => Slliw(i_type).into(),
            FUNCT3_OP_SRL_SRA => match funct7 {
                FUNCT7_OP_SRL => Srliw(i_type).into(),
                FUNCT7_OP_SRA => Sraiw(i_type).into(),
                _ => Err(())?,
            },
            _ => Err(())?,
        },
        OPCODE_OP_32 if xlen == Xlen::X64 => match funct3 {
            FUNCT3_OP_ADD_SUB => match funct7 {
                FUNCT7_OP_ADD => Addw(r_type).into(),
                FUNCT7_OP_SUB => Subw(r_type).into(),
                _ => Err(())?,
            },
            FUNCT3_OP_SLL if funct7 == 0 => Sllw(r_type).into(),
            FUNCT3_OP_SRL_SRA => match funct7 {
                FUNCT7_OP_SRL => Srlw(r_type).into(),
                FUNCT7_OP_SRA => Sraw(r_type).into(),
                _ => Err(())?,
            },
            _ => Err(())?,
        },
        OPCODE_LOAD_FP => match funct3 {
            FUNCT3_WIDTH_W => Flw(i_type).into(),
            _ => Err(())?,
        },
        OPCODE_STORE_FP => match funct3 {
            FUNCT3_WIDTH_W => Fsw(s_type).into(),
            _ => Err(())?,
        },
        OPCODE_FMADD => match funct2 {
            FUNCT2_FMT_S => Fmadds(r4_type).into(),
            _ => Err(())?,
        },
        OPCODE_FMSUB => match funct2 {
            FUNCT2_FMT_S => Fmsubs(r4_type).into(),
            _ => Err(())?,
        },
        OPCODE_FNMSUB => match funct2 {
            FUNCT2_FMT_S => Fnmsubs(r4_type).into(),
            _ => Err(())?,
        },
        OPCODE_FNMADD => match funct2 {
            FUNCT2_FMT_S => Fnmadds(r4_type).into(),
            _ => Err(())?,
        },
        OPCODE_FP => match rs3 {
            FUNCT_RS3_FP_ADD => match funct2 {
                FUNCT2_FMT_S => Fadds(r_type).into(),
                _ => Err(())?,
            },
            FUNCT_RS3_FP_SUB => match funct2 {
                FUNCT2_FMT_S => Fsubs(r_type).into(),
                _ => Err(())?,
            },
            FUNCT_RS3_FP_MUL => match funct2 {
                FUNCT2_FMT_S => Fmuls(r_type).into(),
                _ => Err(())?,
            },
            FUNCT_RS3_FP_DIV => match funct2 {
                FUNCT2_FMT_S => Fdivs(r_type).into(),
                _ => Err(())?,
            },
            FUNCT_RS3_FP_SQRT if rs2 == 0 => match funct2 {
                FUNCT2_FMT_S => Fsqrts(r_type).into(),
                _ => Err(())?,
            },
            FUNCT_RS3_FP_MIN_MAX => match funct3 {
                FUNCT3_FP_MIN => match funct2 {
                    FUNCT2_FMT_S => Fmins(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT3_FP_MAX => match funct2 {
                    FUNCT2_FMT_S => Fmaxs(r_type).into(),
                    _ => Err(())?,
                },
                _ => Err(())?,
            },
            FUNCT_RS3_FP_SGNJ => match funct3 {
                FUNCT3_FP_SGNJ => match funct2 {
                    FUNCT2_FMT_S => Fsgnjs(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT3_FP_SGNJN => match funct2 {
                    FUNCT2_FMT_S => Fsgnjns(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT3_FP_SGNJX => match funct2 {
                    FUNCT2_FMT_S => Fsgnjxs(r_type).into(),
                    _ => Err(())?,
                },
                _ => Err(())?,
            },
            FUNCT_RS3_FP_CMP => match funct3 {
                FUNCT3_FP_EQ => match funct2 {
                    FUNCT2_FMT_S => Feqs(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT3_FP_LT => match funct2 {
                    FUNCT2_FMT_S => Flts(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT3_FP_LE => match funct2 {
                    FUNCT2_FMT_S => Fles(r_type).into(),
                    _ => Err(())?,
                },
                _ => Err(())?,
            },
            // fcvt.{w|l}[u].s, fcvt.int.fmt
            FUNCT_RS3_FP_FCVTX => match rs2 {
                FUNCT_RS2_CVT_W => match funct2 {
                    FUNCT2_FMT_S => Fcvtws(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT_RS2_CVT_WU => match funct2 {
                    FUNCT2_FMT_S => Fcvtwus(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT_RS2_CVT_L if xlen != Xlen::X32 => match funct2 {
                    FUNCT2_FMT_S => Fcvtls(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT_RS2_CVT_LU if xlen != Xlen::X32 => match funct2 {
                    FUNCT2_FMT_S => Fcvtlus(r_type).into(),
                    _ => Err(())?,
                },
                _ => Err(())?,
            },
            // fcvt.s.{w|l}[u], fcvt.fmt.int
            FUNCT_RS3_FP_XCVTF => match rs2 {
                FUNCT_RS2_CVT_W => match funct2 {
                    FUNCT2_FMT_S => Fcvtsw(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT_RS2_CVT_WU => match funct2 {
                    FUNCT2_FMT_S => Fcvtswu(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT_RS2_CVT_L if xlen != Xlen::X32 => match funct2 {
                    FUNCT2_FMT_S => Fcvtsl(r_type).into(),
                    _ => Err(())?,
                },
                FUNCT_RS2_CVT_LU if xlen != Xlen::X32 => match funct2 {
                    FUNCT2_FMT_S => Fcvtslu(r_type).into(),
                    _ => Err(())?,
                },
                _ => Err(())?,
            },
            // fmv.x.w
            FUNCT_RS3_FP_FMVX_CLASS if rs2 == 0 && funct3 == 0 => match funct2 {
                FUNCT2_FMT_S => Fmvxw(r_type).into(),
                _ => Err(())?,
            },
            FUNCT_RS3_FP_FMVX_CLASS if rs2 == 0 && funct3 == 1 => match funct2 {
                FUNCT2_FMT_S => Fclasss(r_type).into(),
                _ => Err(())?,
            },
            // fmv.w.x
            FUNCT_RS3_FP_XMVF if rs2 == 0 && funct3 == 0 => match funct2 {
                FUNCT2_FMT_S => Fmvwx(r_type).into(),
                _ => Err(())?,
            },
            _ => Err(())?,
        }, // opcode_fp

        // atomic instructions
        OPCODE_A => match funct3 {
            FUNCT3_LOAD_LW => match funct5 {
                FUNCT5_A_LR => Lrw(r_type).into(),
                FUNCT5_A_SC => Scw(r_type).into(),
                FUNCT5_A_AMOSWAP => Amoswapw(r_type).into(),
                FUNCT5_A_AMOADD => Amoaddw(r_type).into(),
                FUNCT5_A_AMOXOR => Amoxorw(r_type).into(),
                FUNCT5_A_AMOAND => Amoandw(r_type).into(),
                FUNCT5_A_AMOOR => Amoorw(r_type).into(),
                FUNCT5_A_AMOMIN => Amominw(r_type).into(),
                FUNCT5_A_AMOMAX => Amomaxw(r_type).into(),
                FUNCT5_A_AMOMINU => Amominuw(r_type).into(),
                FUNCT5_A_AMOMAXU => Amomaxuw(r_type).into(),
                _ => Err(())?,
            },
            FUNCT3_LOAD_LD => match funct5 {
                FUNCT5_A_LR => Lrd(r_type).into(),
                FUNCT5_A_SC => Scd(r_type).into(),
                FUNCT5_A_AMOSWAP => Amoswapd(r_type).into(),
                FUNCT5_A_AMOADD => Amoaddd(r_type).into(),
                FUNCT5_A_AMOXOR => Amoxord(r_type).into(),
                FUNCT5_A_AMOAND => Amoandd(r_type).into(),
                FUNCT5_A_AMOOR => Amoord(r_type).into(),
                FUNCT5_A_AMOMIN => Amomind(r_type).into(),
                FUNCT5_A_AMOMAX => Amomaxd(r_type).into(),
                FUNCT5_A_AMOMINU => Amominud(r_type).into(),
                FUNCT5_A_AMOMAXU => Amomaxud(r_type).into(),
                _ => Err(())?,
            },
            // RV128A (.q) width
            x if x == FUNCT3_A_WIDTH_Q && xlen == Xlen::X128 => match funct5 {
                FUNCT5_A_LR => Lrq(r_type).into(),
                FUNCT5_A_SC => Scq(r_type).into(),
                FUNCT5_A_AMOSWAP => Amoswapq(r_type).into(),
                FUNCT5_A_AMOADD => Amoaddq(r_type).into(),
                FUNCT5_A_AMOXOR => Amoxorq(r_type).into(),
                FUNCT5_A_AMOAND => Amoandq(r_type).into(),
                FUNCT5_A_AMOOR => Amoorq(r_type).into(),
                FUNCT5_A_AMOMIN => Amominq(r_type).into(),
                FUNCT5_A_AMOMAX => Amomaxq(r_type).into(),
                FUNCT5_A_AMOMINU => Amominuq(r_type).into(),
                FUNCT5_A_AMOMAXU => Amomaxuq(r_type).into(),
                _ => Err(())?,
            },
            _ => Err(())?,
        },
        _ => Err(())?,
    };
    Ok(ans)
}
