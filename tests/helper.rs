use emurv::{
    cpu,
    opcode::{I_TYPE, R_TYPE},
};

pub fn set_i_type_instruction(imm: i16, rs1: u8, funct3: u8, rd: u8) -> u32 {
    // |31-20|19-15|14-12|11-7|6-0|
    return ((imm as u32 & 0xfff) << 20)
        | ((rs1 as u32 & 0x1f) << 15)
        | ((funct3 as u32 & 0x7) << 12)
        | ((rd as u32 & 0x1f) << 7)
        | ((I_TYPE as u32) & 0x7f);
}

pub fn set_u_type_instruction(imm: i32, rd: u8, opcode: u8) -> u32 {
    return (imm as u32 & 0xfffff000) as u32 | ((rd as u32 & 0x1f) << 7) | ((opcode as u32) & 0x7f);
}

pub fn set_j_type_instruction(imm: i32, rd: u8, opcode: u8) -> u32 {
    // |31-12|11-7|6-0|
    // imm[20|10:1|11|19:12] = instr[31|30:21|20|19:12]
    let instr_imm = (((imm as i64) << 11) & 0x80000000)
        | (((imm as i64) << 20) & 0x3ff00000)
        | (((imm as i64) << 9) & 0x80000)
        | ((imm as i64) & 0xff000);
    return (instr_imm) as u32 | ((rd as u32 & 0x1f) << 7) | ((opcode as u32) & 0x7f);
}

pub fn set_r_type_instruction(funct7: u8, rs2: u8, rs1: u8, rd: u8) -> u32 {
    // |31-20|19-15|14-12|11-7|6-0|
    return ((funct7 as u32 & 0x7f) << 25)
        | ((rs2 as u32 & 0x1f) << 20)
        | ((rs1 as u32 & 0x1f) << 15)
        | ((rd as u32 & 0x1f) << 7)
        | ((R_TYPE as u32) & 0x7f);
}

pub fn set_register_val(cpu: &mut cpu::CPU, rd: u8, val: i16) {
    let instr = set_i_type_instruction(val, 0x0, 0x0, rd);
    cpu::exec_addi(cpu, instr);
}
