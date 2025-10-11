#![allow(dead_code)]
use super::{to_register, RType};

#[derive(Debug, Clone, Copy)]
pub enum RV32A {
    // Load-Reserved/Store-Conditional - Word (32-bit)
    Lrw(RType),
    Scw(RType),
    
    // Atomic Memory Operations - Word (32-bit)
    Amoswapw(RType),
    Amoaddw(RType),
    Amoxorw(RType),
    Amoandw(RType),
    Amoorw(RType),
    Amominw(RType),
    Amomaxw(RType),
    Amominuw(RType),
    Amomaxuw(RType),
}

impl RV32A {
    pub fn to_string(&self) -> String {
        match self {
            Self::Lrw(r) => format!("lr.w {}, {}", to_register(r.rd), to_register(r.rs1)),
            Self::Scw(r) => format!("sc.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
            Self::Amoswapw(r) => format!("amoswap.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
            Self::Amoaddw(r) => format!("amoadd.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
            Self::Amoxorw(r) => format!("amoxor.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
            Self::Amoandw(r) => format!("amoand.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
            Self::Amoorw(r) => format!("amoor.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
            Self::Amominw(r) => format!("amomin.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
            Self::Amomaxw(r) => format!("amomax.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
            Self::Amominuw(r) => format!("amominu.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
            Self::Amomaxuw(r) => format!("amomaxu.w {}, {}", to_register(r.rs2), to_register(r.rs1)),
        }
    }
}
            