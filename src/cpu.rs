use crate::debug::REGS_NAMES;
use crate::memory;
use crate::opcode::*;
use crate::registers;

#[derive(Debug, Clone)]
pub struct CPU {
    // integer registers
    pub xregs: registers::XREGS,
    pub pc: u32,

    pub bus: memory::BUS,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu: CPU = CPU {
            xregs: registers::XREGS::new(),
            pc: memory::MEM_BASE,
            bus: memory::BUS::new(),
        };
        cpu.xregs.regs[2] = memory::MEM_BASE + memory::MEM_SIZE; // Set stack pointer
        cpu.pc = memory::MEM_BASE;
        return cpu;
    }

    pub fn fetch(&self) -> u32 {
        let instr: u32 = self.bus.load(self.pc, 32);
        return instr;
    }

    pub fn execute(&mut self, instr: u32) {
        let opcode = instr & 0x7f;
        let funct3 = (instr >> 12) & 0x7;
        let funct7 = (instr >> 25) & 0x7f;
        self.xregs.regs[0] = 0; // x0 hardwired to 0 at each cycle

        match opcode {
            LUI => exec_lui(self, instr),
            AUIPC => exec_auipc(self, instr),
            JAL => exec_jal(self, instr),
            JALR => exec_jalr(self, instr),
            B_TYPE => match funct3 {
                BEQ => exec_beq(self, instr),
                BNE => exec_bne(self, instr),
                BLT => exec_blt(self, instr),
                BGE => exec_bge(self, instr),
                BLTU => exec_bltu(self, instr),
                BGEU => exec_bgeu(self, instr),
                _ => panic!(),
            },
            LOAD => match funct3 {
                LB => exec_lb(self, instr),
                LH => exec_lh(self, instr),
                LW => exec_lw(self, instr),
                LBU => exec_lbu(self, instr),
                LHU => exec_lhu(self, instr),
                LWU => exec_lwu(self, instr),
                _ => panic!(),
            },
            S_TYPE => match funct3 {
                SB => exec_sb(self, instr),
                SH => exec_sh(self, instr),
                SW => exec_sw(self, instr),
                _ => panic!(),
            },
            I_TYPE => match funct3 {
                ADDI => exec_addi(self, instr),
                SLLI => exec_slli(self, instr),
                SLTI => exec_slti(self, instr),
                SLTIU => exec_sltiu(self, instr),
                XORI => exec_xori(self, instr),
                SRI => match funct7 {
                    SRLI => exec_srli(self, instr),
                    SRAI => exec_srai(self, instr),
                    _ => panic!(),
                },
                ORI => exec_ori(self, instr),
                ANDI => exec_andi(self, instr),
                _ => {
                    panic!("malformed I type instruction");
                }
            },
            R_TYPE => match funct3 {
                ADDSUB => match funct7 {
                    ADD => exec_add(self, instr),
                    SUB => exec_sub(self, instr),
                    _ => (),
                },
                SLL => exec_sll(self, instr),
                SLT => exec_slt(self, instr),
                SLTU => exec_sltu(self, instr),
                XOR => exec_xor(self, instr),
                SR => match funct7 {
                    SRL => exec_srl(self, instr),
                    SRA => exec_sra(self, instr),
                    _ => (),
                },
                OR => exec_or(self, instr),
                AND => exec_and(self, instr),
                _ => {
                    panic!("malformed I type instruction");
                }
            },
            FENCE => exec_fence(self, instr),
            CSR => match (funct3) {
                ECALL | EBREAK => match imm_i(instr) {
                    0x0 => exec_ecall(self, instr),
                    0x1 => exec_ebreak(self, instr),
                    _ => (),
                },
                CSRRW => exec_csrrw(self, instr),
                CSRRS => exec_csrrs(self, instr),
                CSRRC => exec_csrrc(self, instr),
                CSRRWI => exec_csrrwi(self, instr),
                CSRRSI => exec_csrrsi(self, instr),
                CSRRCI => exec_csrrci(self, instr),
                _ => {
                    panic!("malformed CSR instruction");
                }
            },
            _ => panic!("invalid instr {}, opcode: {:b}", instr, opcode),
        }
    }
}

// RV32I
// see page 64 at https://riscv.org/wp-content/uploads/2016/06/riscv-spec-v2.1.pdf
pub fn exec_lui(cpu: &mut CPU, instr: u32) {
    let imm = (imm_u(instr) as i32) as u32;
    dump_format_instr_u(cpu, instr);
    cpu.xregs.regs[rd(instr) as usize] = imm;
}
pub fn exec_auipc(cpu: &mut CPU, instr: u32) {
    let imm = imm_u(instr) as i32;
    dump_format_instr_u(cpu, instr);
    cpu.xregs.regs[rd(instr) as usize] = (cpu.pc as i32).wrapping_add(imm) as u32;
}
pub fn exec_jal(cpu: &mut CPU, instr: u32) {
    let imm = imm_j(instr) as i32;
    cpu.xregs.regs[rd(instr) as usize] = cpu.pc.wrapping_add(4);
    cpu.pc = ((cpu.pc as i32).wrapping_add(imm)).wrapping_sub(4) as u32;
}
pub fn exec_jalr(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr) as i32;
    cpu.xregs.regs[rd(instr) as usize] = cpu.pc + 4;
    // ignore the last 1 bit with 0xfffffffe
    cpu.pc = (cpu.xregs.regs[rs1(instr) as usize] as i32).wrapping_add(imm) as u32 & 0xfffffffe;
}
pub fn exec_beq(cpu: &mut CPU, instr: u32) {
    let imm = imm_b(instr) as i32;
    if cpu.xregs.regs[rs1(instr) as usize] == cpu.xregs.regs[rs2(instr) as usize] {
        cpu.pc = (cpu.pc as i32).wrapping_add(imm).wrapping_sub(4) as u32;
    }
}
pub fn exec_bne(cpu: &mut CPU, instr: u32) {
    let imm = imm_b(instr) as i32;
    dump_format_instr_b(cpu, instr);
    if cpu.xregs.regs[rs1(instr) as usize] != cpu.xregs.regs[rs2(instr) as usize] {
        cpu.pc = (cpu.pc as i32).wrapping_add(imm).wrapping_sub(4) as u32;
    }
}
pub fn exec_blt(cpu: &mut CPU, instr: u32) {
    let imm = imm_b(instr) as i32;
    dump_format_instr_b(cpu, instr);
    if (cpu.xregs.regs[rs1(instr) as usize] as i32) < (cpu.xregs.regs[rs2(instr) as usize] as i32) {
        cpu.pc = (cpu.pc as i32).wrapping_add(imm).wrapping_sub(4) as u32;
    }
}
pub fn exec_bge(cpu: &mut CPU, instr: u32) {
    let imm = imm_b(instr) as i32;
    if (cpu.xregs.regs[rs1(instr) as usize] as i32) >= (cpu.xregs.regs[rs2(instr) as usize] as i32)
    {
        cpu.pc = (cpu.pc as i32).wrapping_add(imm).wrapping_sub(4) as u32;
    }
}
pub fn exec_bltu(cpu: &mut CPU, instr: u32) {
    let imm = imm_b(instr) as i32;
    if cpu.xregs.regs[rs1(instr) as usize] < cpu.xregs.regs[rs2(instr) as usize] {
        cpu.pc = (cpu.pc as i32).wrapping_add(imm).wrapping_sub(4) as u32;
    }
}
pub fn exec_bgeu(cpu: &mut CPU, instr: u32) {
    let imm = imm_b(instr) as i32;
    if cpu.xregs.regs[rs1(instr) as usize] >= cpu.xregs.regs[rs2(instr) as usize] {
        cpu.pc = (cpu.pc as i32).wrapping_add(imm).wrapping_sub(4) as u32;
    }
}
pub fn exec_lb(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr) as i32;
    let load_i8 = cpu.bus.load(
        (cpu.xregs.regs[rs1(instr) as usize] as i32).wrapping_add(imm) as u32,
        8,
    ) as i32;
    cpu.xregs.regs[rd(instr) as usize] = ((load_i8 << 26) >> 26) as u32;
}
pub fn exec_lh(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr) as i32;
    let load_i16 = cpu.bus.load(
        (cpu.xregs.regs[rs1(instr) as usize] as i32).wrapping_add(imm) as u32,
        16,
    ) as i32;
    cpu.xregs.regs[rd(instr) as usize] = ((load_i16 << 16) >> 16) as u32;
}
pub fn exec_lw(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr) as i32;
    cpu.xregs.regs[rd(instr) as usize] = cpu.bus.load(
        (cpu.xregs.regs[rs1(instr) as usize] as i32).wrapping_add(imm) as u32,
        32,
    );
}
pub fn exec_lbu(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr) as u32;
    cpu.xregs.regs[rd(instr) as usize] = cpu
        .bus
        .load(cpu.xregs.regs[rs1(instr) as usize].wrapping_add(imm), 8);
}
pub fn exec_lhu(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr) as u32;
    cpu.xregs.regs[rd(instr) as usize] = cpu
        .bus
        .load(cpu.xregs.regs[rs1(instr) as usize].wrapping_add(imm), 16);
}
pub fn exec_lwu(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr) as u32;
    cpu.xregs.regs[rd(instr) as usize] = cpu
        .bus
        .load(cpu.xregs.regs[rs1(instr) as usize].wrapping_add(imm), 32);
}
pub fn exec_sb(cpu: &mut CPU, instr: u32) {
    let imm = imm_s(instr) as i32;
    let addr = (cpu.xregs.regs[rs1(instr) as usize] as i32).wrapping_add(imm) as u32;
    let val = cpu.xregs.regs[rs2(instr) as usize] & std::u8::MAX as u32;
    cpu.bus.store(addr, 8, val);
}
pub fn exec_sh(cpu: &mut CPU, instr: u32) {
    let imm = imm_s(instr) as i32;
    let addr = (cpu.xregs.regs[rs1(instr) as usize] as i32).wrapping_add(imm) as u32;
    let val = cpu.xregs.regs[rs2(instr) as usize] & std::u16::MAX as u32;
    cpu.bus.store(addr, 16, val);
}
pub fn exec_sw(cpu: &mut CPU, instr: u32) {
    let imm = imm_s(instr) as i32;
    let addr = (cpu.xregs.regs[rs1(instr) as usize] as i32).wrapping_add(imm) as u32;
    let val = cpu.xregs.regs[rs2(instr) as usize] & std::u32::MAX as u32;
    cpu.bus.store(addr, 32, val);
}
pub fn exec_addi(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr);
    dump_format_instr_i(cpu, instr);
    cpu.xregs.regs[rd(instr) as usize] =
        (cpu.xregs.regs[rs1(instr) as usize] as i32).wrapping_add(imm) as u32;
}
pub fn exec_slti(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr);
    cpu.xregs.regs[rd(instr) as usize] =
        ((cpu.xregs.regs[rs1(instr) as usize] as i32) < (imm as i32)) as u32;
}
pub fn exec_sltiu(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr);
    cpu.xregs.regs[rd(instr) as usize] = (cpu.xregs.regs[rs1(instr) as usize] < imm as u32) as u32;
}
pub fn exec_xori(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr);
    cpu.xregs.regs[rd(instr) as usize] = cpu.xregs.regs[rs1(instr) as usize] ^ imm as u32;
}
pub fn exec_ori(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr);
    cpu.xregs.regs[rd(instr) as usize] = cpu.xregs.regs[rs1(instr) as usize] | imm as u32;
}
pub fn exec_andi(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr);
    cpu.xregs.regs[rd(instr) as usize] = cpu.xregs.regs[rs1(instr) as usize] & imm as u32;
}
pub fn exec_slli(cpu: &mut CPU, instr: u32) {
    let shamt = shamt(instr);
    dump_format_instr_i(cpu, instr);
    // shift-by-immediate takes only the lower 5 bits
    cpu.xregs.regs[rd(instr) as usize] = cpu.xregs.regs[rs1(instr) as usize] << shamt;
}
pub fn exec_srli(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr);
    // shift-by-immediate takes only the lower 5 bits
    cpu.xregs.regs[rd(instr) as usize] = cpu.xregs.regs[rs1(instr) as usize] >> (imm & 0x1f) as u32;
}
pub fn exec_srai(cpu: &mut CPU, instr: u32) {
    let imm = imm_i(instr);
    cpu.xregs.regs[rd(instr) as usize] =
        (cpu.xregs.regs[rs1(instr) as usize] as i32 >> (imm & 0x1f)) as u32;
}
pub fn exec_add(cpu: &mut CPU, instr: u32) {
    cpu.xregs.regs[rd(instr) as usize] = (cpu.xregs.regs[rs1(instr) as usize] as i32
        + cpu.xregs.regs[rs2(instr) as usize] as i32)
        as u32;
}
pub fn exec_sub(cpu: &mut CPU, instr: u32) {
    dump_format_instr_r(cpu, instr);
    cpu.xregs.regs[rd(instr) as usize] = (cpu.xregs.regs[rs1(instr) as usize] as i32)
        .wrapping_sub(cpu.xregs.regs[rs2(instr) as usize] as i32)
        as u32;
}
pub fn exec_sll(cpu: &mut CPU, instr: u32) {
    cpu.xregs.regs[rd(instr) as usize] = ((cpu.xregs.regs[rs1(instr) as usize] as i32)
        << cpu.xregs.regs[rs2(instr) as usize] as i32)
        as u32;
}
pub fn exec_slt(cpu: &mut CPU, instr: u32) {
    cpu.xregs.regs[rd(instr) as usize] = ((cpu.xregs.regs[rs1(instr) as usize] as i32)
        < cpu.xregs.regs[rs2(instr) as usize] as i32)
        as u32;
}
pub fn exec_sltu(cpu: &mut CPU, instr: u32) {
    cpu.xregs.regs[rd(instr) as usize] =
        (cpu.xregs.regs[rs1(instr) as usize] < cpu.xregs.regs[rs2(instr) as usize]) as u32;
}
pub fn exec_xor(cpu: &mut CPU, instr: u32) {
    cpu.xregs.regs[rd(instr) as usize] =
        cpu.xregs.regs[rs1(instr) as usize] ^ cpu.xregs.regs[rs2(instr) as usize];
}
pub fn exec_srl(cpu: &mut CPU, instr: u32) {
    cpu.xregs.regs[rd(instr) as usize] =
        cpu.xregs.regs[rs1(instr) as usize] >> cpu.xregs.regs[rs2(instr) as usize];
}
pub fn exec_sra(cpu: &mut CPU, instr: u32) {
    cpu.xregs.regs[rd(instr) as usize] = ((cpu.xregs.regs[rs1(instr) as usize] as i32)
        >> cpu.xregs.regs[rs2(instr) as usize] as i32)
        as u32;
}
pub fn exec_or(cpu: &mut CPU, instr: u32) {
    cpu.xregs.regs[rd(instr) as usize] =
        cpu.xregs.regs[rs1(instr) as usize] | cpu.xregs.regs[rs2(instr) as usize];
}
pub fn exec_and(cpu: &mut CPU, instr: u32) {
    cpu.xregs.regs[rd(instr) as usize] =
        cpu.xregs.regs[rs1(instr) as usize] & cpu.xregs.regs[rs2(instr) as usize];
}
pub fn exec_fence(cpu: &mut CPU, instr: u32) {}
pub fn exec_fence_i(cpu: &mut CPU, instr: u32) {}
pub fn exec_ecall(cpu: &mut CPU, instr: u32) {}
pub fn exec_ebreak(cpu: &mut CPU, instr: u32) {}
pub fn exec_csrrw(cpu: &mut CPU, instr: u32) {}
pub fn exec_csrrs(cpu: &mut CPU, instr: u32) {}
pub fn exec_csrrc(cpu: &mut CPU, instr: u32) {}
pub fn exec_csrrwi(cpu: &mut CPU, instr: u32) {}
pub fn exec_csrrsi(cpu: &mut CPU, instr: u32) {}
pub fn exec_csrrci(cpu: &mut CPU, instr: u32) {}

fn dump_format_instr_r(cpu: &CPU, instr: u32) {
    println!(
        "{}<- {}: {:#x}, {}: {:#x}",
        REGS_NAMES[rd(instr) as usize],
        REGS_NAMES[rs1(instr) as usize],
        cpu.xregs.regs[rs1(instr) as usize],
        REGS_NAMES[rs2(instr) as usize],
        cpu.xregs.regs[rs2(instr) as usize],
    );
}
fn dump_format_instr_i(cpu: &CPU, instr: u32) {
    println!(
        "{}<- {}: {:#x}, imm: {:#x}",
        REGS_NAMES[rd(instr) as usize],
        REGS_NAMES[rs1(instr) as usize],
        cpu.xregs.regs[rs1(instr) as usize],
        imm_i(instr) as i32,
    );
}
fn dump_format_instr_s(cpu: &CPU, instr: u32) {
    println!(
        "{}: {:#x}, {}: {:#x}, imm: {:#x}",
        REGS_NAMES[rs1(instr) as usize],
        cpu.xregs.regs[rs1(instr) as usize],
        REGS_NAMES[rs2(instr) as usize],
        cpu.xregs.regs[rs2(instr) as usize],
        imm_s(instr) as i32,
    );
}
fn dump_format_instr_load(cpu: &CPU, instr: u32) {
    println!(
        "{}<- {}: {:#x}, imm: {:#x}",
        REGS_NAMES[rd(instr) as usize],
        REGS_NAMES[rs1(instr) as usize],
        cpu.xregs.regs[rs1(instr) as usize],
        imm_i(instr) as i32,
    );
}
fn dump_format_instr_b(cpu: &CPU, instr: u32) {
    println!(
        "{}: {:#x}, {}: {:#x}, imm: {:#x}",
        REGS_NAMES[rs1(instr) as usize],
        cpu.xregs.regs[rs1(instr) as usize],
        REGS_NAMES[rs2(instr) as usize],
        cpu.xregs.regs[rs2(instr) as usize],
        imm_b(instr),
    );
}
fn dump_format_instr_j(cpu: &CPU, instr: u32) {
    println!(
        "{}<- {:#x}, imm: {:#x}",
        REGS_NAMES[rd(instr) as usize],
        cpu.xregs.regs[rs1(instr) as usize],
        imm_j(instr) as u32,
    );
}
fn dump_format_instr_u(cpu: &CPU, instr: u32) {
    println!(
        "{}<- {:#x}, imm: {:#x}",
        REGS_NAMES[rd(instr) as usize],
        cpu.xregs.regs[rs1(instr) as usize],
        imm_u(instr) as u32,
    );
}
