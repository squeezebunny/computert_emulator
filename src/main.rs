use std::thread::sleep;
use std::time::{Duration};

iota::iota! {
    const NOP: u8 = iota;
    , ADD, SUB, MUL, DIV, NEG
    , AND, OR, XOR, NOT
    , BEQ, BNEQ, BLT, BLE, BGT, BGE, B
    , BCS, BCC, BMI, BPL, BVS, BVC, BHI, BLS
    , LSL, LSR, ASR
    , LDR, STR
}

fn main() {

    let mut ram = [0u16; 8];

    ram[6] = 8;
    ram[7] = 16;

    run(&mut ram, 8, vec![
        (LDR, 6, 0, 0),
        (LDR, 7, 0, 1),
        (ADD, 0, 1, 2),
        (STR, 0, 0, 0),
    ]);

    assert_eq!(ram[0], 24);

}

fn run(ram: &mut [u16; 8], steps: usize, prog: Vec<(u8, u8, u8, u8)>) {
    let mut pc = 0usize;
    let mut registers = [0u8; 8];

    let mut f_zero = false;
    let mut f_carry = false;
    let mut f_neg = false;
    let mut f_overflow = false;

    for _ in 0..steps {
        let ins_l = (ram[pc] >> 8) as u8;
        let ins_r = ram[pc] as u8;

        let opcode = ins_l >> 3;
        let dest = (ins_l & 0b00000111) as usize;
        let source1 = (ins_r >> 5) as usize;
        let source2 = ins_r & 0b00011111;
        let is_const = 1 == ins_r >> 4 & 0b1;

        let x = registers[source1];

        let y = if is_const {
            (source2 as i8) as u8
        } else {
            registers[source2 as usize]
        };

        use instruction_types::*;
        enum instruction_types {
            INO, // no operation
            ISR(u8), // set register
            IASR(u8), // arithmetic set register
            ISM(u8), // set memory/ram
            IBR(bool), // branch
        }

        match match opcode {
            NOP => { INO }

            ADD => { IASR(x + y) }
            SUB => { IASR(x - y) }
            MUL => { IASR(x * y) }
            DIV => { IASR(x / y) }
            NEG => { IASR(-(y as i8) as u8) }

            AND => { ISR(x & y) }
            OR  => { ISR(x | y) }
            XOR => { ISR(x ^ y) }
            NOT => { ISR(!y) }

            LSL => { ISR(x << y) }
            LSR => { ISR(x >> y) }
            ASR => { ISR(((x as i8) >> y) as u8) }

            BCS => { IBR(f_carry) }
            BCC => { IBR(!f_carry) }
            BMI => { IBR(f_neg) }
            BPL => { IBR(!f_neg) }
            BVS => { IBR(f_overflow) }
            BVC => { IBR(!f_overflow) }
            BHI => { IBR(f_carry && !f_zero) }
            BLS => { IBR(!(f_carry && !f_zero)) }
            B => { IBR(true) }

            _ => { panic!("undefined opcode used: {}", opcode)}
        } {
            INO => { }
            IBR(b) => { if b { pc += (y as usize); } }
            ISM(x) => { ram[y as usize] = x as u16 }
            ISR(o) => {
                registers[dest] = o;
                f_zero = (o == 0);
            }
            IASR(o) => {
                registers[dest] = o;
                f_zero = (o == 0);
                f_neg = (o >> 7) == 1;
                f_carry = (((x + y) as u16) > 255 || ((x + y) as i16) < 0);
                f_overflow = (f_neg || f_zero) && !(f_neg == f_zero);
            }
        }

        println!(r#"PC: {pc}
        Registers: {registers:?}
        Memory: {ram:?}"#);

        pc += 1;

        sleep(Duration::from_secs(1) / 2);
    }
}


