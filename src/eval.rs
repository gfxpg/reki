use std::ops::RangeInclusive;

pub struct ExecutionState {
    pub sgprs: Vec<&'static str>,
    pub vgprs: Vec<&'static str>,
    pub instrs: Vec<String>
}

#[derive(Debug)]
enum Operand {
    ScalarReg(u8),
    VectorReg(u8),
    ScalarRegRange(RangeInclusive<u8>),
    VectorRegRange(RangeInclusive<u8>),
    Lit(u32),
    VCC,
    Keyseq(String)
}

fn parse_operand(operand: &str) -> Operand {
    let op_start_char = operand.chars().nth(0).unwrap();

    if operand == "vcc" {
        Operand::VCC
    }
    else if operand.len() > 2 && &operand[0..2] == "0x" {
        Operand::Lit(u32::from_str_radix(&operand[2..], 16).unwrap())
    }
    else if op_start_char.is_digit(10) {
        Operand::Lit(u32::from_str_radix(operand, 10).unwrap())
    }
    else if operand.starts_with("s") || operand.starts_with("v") {
        match operand[1..].parse::<u8>() {
            Ok(i) =>
                if operand.starts_with("s") {
                    Operand::ScalarReg(i)
                }
                else {
                    Operand::VectorReg(i)
                }
            _ => {
                if &operand[1..2] != "[" {
                    panic!("unable to parse \"{}\" as an instruction operand", operand)
                }

                let sides: Vec<&str> = operand[2..operand.len() - 1].split(':').collect();
                let left = sides[0].parse::<u8>().unwrap();
                let right = sides[1].parse::<u8>().unwrap();

                if operand.starts_with("s") {
                    Operand::ScalarRegRange(left..=right)
                }
                else {
                    Operand::VectorRegRange(left..=right)
                }
            }
        }
    }
    else {
        Operand::Keyseq(operand.to_string())
    }
}

fn parse_instr(instr: &str) -> (&str, Vec<Operand>) {
    let instr_ops: Vec<&str> = instr.splitn(2, ' ').collect();
    if instr_ops.len() == 1 {
        (instr_ops[0], Vec::new())
    }
    else {
        (instr_ops[0], instr_ops[1].split(", ").map(parse_operand).collect())
    }
}

pub fn eval_loads(st: ExecutionState) {
    for i in st.instrs.iter() {
        let (instr, ops) = parse_instr(i);
        println!("{} {:?}", instr, ops);
    }
}

