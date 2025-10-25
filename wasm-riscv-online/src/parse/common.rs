use crate::asm;

pub(crate) fn trim_comment(s: &str) -> &str {
    let s = s.split("//").next().unwrap_or(s);
    let s = s.split('#').next().unwrap_or(s);
    s
}

pub(crate) fn split_operands(s: &str) -> Vec<String> {
    s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
}

pub(crate) fn parse_register(s: &str) -> Result<u8, String> {
    if let Some(r) = asm::from_register(s) { Ok(r) } else { Err(format!("未知寄存器: {}", s)) }
}

pub(crate) fn parse_int(s: &str) -> Result<i64, String> {
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
                if asm::from_register(t).is_some() {
                    Err(format!("期望立即数，但提供了寄存器: {}。对于 addi 这类指令，第 3 个参数应为立即数。", t))
                } else {
                    Err(format!("立即数解析失败: {}", e))
                }
            }
        }
    }
}

pub(crate) fn imm_signed_bits(value: i64, bits: u8) -> Result<u32, String> {
    let min = -(1i64 << (bits - 1));
    let max = (1i64 << (bits - 1)) - 1;
    if value < min || value > max { return Err(format!("立即数超出范围: {} ({} 位)", value, bits)); }
    if value >= 0 { Ok(value as u32) } else { Ok(((1i64 << bits) + value) as u32) }
}

pub(crate) fn parse_mem_operand(s: &str) -> Result<(u32, u8), String> {
    // format: imm(rs)
    let open = s.find('(').ok_or_else(|| format!("内存操作数格式错误: {}", s))?;
    let close = s.find(')').ok_or_else(|| format!("内存操作数格式错误: {}", s))?;
    let imm_str = s[..open].trim();
    let reg_str = &s[open + 1..close];
    let imm = parse_int(imm_str)?;
    let rs = parse_register(reg_str)?;
    Ok((imm_signed_bits(imm, 12)?, rs))
}
