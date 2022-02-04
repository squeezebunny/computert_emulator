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

const MEM_SIZE: usize = 16;

fn main() {
    let mut test_prg = ez_program(vec![
        (LDR, 6, 0, 0),
        (LDR, 7, 0, 1),
        (ADD, 0, 1, 2),
        (STR, 0, 0, 5),
    ]);

    test_prg[14] = 8;
    test_prg[15] = 16;

    run(&mut test_prg, 4);

    assert_eq!(test_prg[0], 24);
}

fn ez_program(stuff: Vec<(u8, u8, u8, u8)>) -> [u16; MEM_SIZE] {
    let mut mem = [0u16; MEM_SIZE];
    for (i, e) in stuff.into_iter().enumerate() {
        let num =
              ((e.0 as u16) << 11) // instruction
            | ((e.1 & 0b111) as u16) << 8 // destination register
            | ((e.2 & 0b111) as u16) << 5 // source reg 1
            | ((e.3 & 0b111) as u16); // source reg 2
        println!("{:#016b}", num);
        mem[i] = num;
    }
    mem
}

fn run(ram: &mut [u16; MEM_SIZE], steps: usize) {
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
        let dest = (ins_l & 0b111) as usize;
        let source1 = (ins_r >> 5) as usize;
        let source2 = ins_r & 0b11111;
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
            ISM, // set memory/ram
            ISR(u8), // set register
            IASR(u8), // arithmetic set register
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

            BEQ => { IBR(f_zero) }
            BNEQ => { IBR(!f_zero) }
            BLT => { IBR(f_neg != f_overflow) }
            BLE => { IBR(!f_zero && (f_neg == f_overflow)) }
            BGT => { IBR(f_zero && (f_neg == f_overflow)) }
            BGE => { IBR(f_neg == f_overflow) }
            B => { IBR(true) }

            BCS => { IBR(f_carry) }
            BCC => { IBR(!f_carry) }
            BMI => { IBR(f_neg) }
            BPL => { IBR(!f_neg) }
            BVS => { IBR(f_overflow) }
            BVC => { IBR(!f_overflow) }
            BHI => { IBR(f_carry && !f_zero) }
            BLS => { IBR(!(f_carry && !f_zero)) }

            LDR => { ISR(ram[y as usize] as u8) }
            STR => { ISM }

            _ => { panic!("undefined opcode used: {}", opcode)}
        } {
            INO => { }
            ISM => { ram[dest as usize] = registers[y as usize] as u16 }
            IBR(b) => { if b { pc += (y as usize); } }
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

        //sleep(Duration::from_secs(1) / 2);
    }
}


