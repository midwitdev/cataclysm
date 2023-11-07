use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::{self, write},
    hash::{self, Hash, Hasher},
    string,
};

struct Label {
    label: String,
}

struct Global {
    value: String,
}

impl Label {
    fn plain(label: &str) -> Self {
        Label {
            label: label.to_string(),
        }
    }

    fn hashed(label: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        label.hash(&mut hasher);
        let hash = hasher.finish();

        Label {
            label: format!("L_{:x}", hash),
        }
    }
}

impl Global {
    fn new(value: &str) -> Self {
        Global {
            value: value.to_string(),
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:", self.label)
    }
}

impl fmt::Display for Global {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "global {}", self.value)
    }
}

struct Amd64Instruction {
    mnemonic: String,
    operands: Vec<Operand>,
}

enum ImmediateValue {
    Label(Label),
    U64(u64),
    USize(usize),
    I64(i64),
    Bytes(&'static [u8]),
}

impl fmt::Display for ImmediateValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ImmediateValue::U64(n) => write!(f, "{}", n),
            ImmediateValue::I64(n) => write!(f, "{}", n),
            ImmediateValue::USize(n) => write!(f, "{}", n),
            ImmediateValue::Label(s) => {
                write!(f, "{}", s.label)
            }
            ImmediateValue::Bytes(b) => {
                
                for (i, &byte) in b.iter().enumerate() {
                    let formatted_byte = format!("0x{:02X}", byte);
                
                    if i == b.len() - 1 {
                        write!(f, "{}", formatted_byte).unwrap();
                    } else {
                        write!(f, "{}, ", formatted_byte).unwrap();
                    }
                }

                Ok(())
            }
        }
    }
}

struct LabelOffset {
    label: Label,
    rel: Option<Amd64Register>,
}

enum Operand {
    Register(Amd64Register),
    Immediate(ImmediateValue),
    DataRef(LabelOffset),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::Immediate(imm) => write!(f, "{}", imm),
            Operand::DataRef(r) => {
                match &r.rel {
                    None => write!(f, "[rel {}]", r.label.label),
                    Some(v) => write!(f, "[{} + {}]", v, r.label.label),
                }
                
            }
        }
    }
}

enum Amd64SpecialRegister {
    RAX,
    RBX,
    RCX,
    RDX,
    RDI,
    RSI,
    RIP,
}

impl fmt::Display for Amd64SpecialRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Amd64SpecialRegister::RAX => write!(f, "{}", "rax"),
            Amd64SpecialRegister::RBX => write!(f, "{}", "rbx"),
            Amd64SpecialRegister::RCX => write!(f, "{}", "rcx"),
            Amd64SpecialRegister::RDX => write!(f, "{}", "rdx"),
            Amd64SpecialRegister::RDI => write!(f, "{}", "rdi"),
            Amd64SpecialRegister::RSI => write!(f, "{}", "rsi"),
            Amd64SpecialRegister::RIP => write!(f, "{}", "rip"),
        }
    }
}

enum Amd64Register {
    GeneralPurpose(u32),
    Special(Amd64SpecialRegister), // Add more register types as needed (e.g., SIMD, FP, etc.)
}

impl fmt::Display for Amd64Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Amd64Register::GeneralPurpose(reg_num) => write!(f, "x{}", reg_num),
            Amd64Register::Special(reg) => write!(f, "{}", reg),
            // Add more cases for other register types (e.g., SIMD, FP) as needed
        }
    }
}

struct Amd64MemoryAccess {
    base_register: Amd64Register,
    displacement: i64,
    index_register: Option<Amd64Register>,
    scale: u32,
}

struct Amd64LabelOffset {
    label: ImmediateValue,
    offset: i64,
    dest_register: Amd64Register,
}

impl fmt::Display for Amd64LabelOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //8(L_8b785f225f7f0d83)(%rip), %rsi
        write!(
            f,
            "{}({})(%rip), {}",
            self.offset, self.label, self.dest_register
        )
    }
}

impl fmt::Display for Amd64MemoryAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}", self.base_register)?;

        if self.displacement != 0 {
            write!(f, "{}", self.displacement)?;
        }

        if let Some(index_reg) = &self.index_register {
            write!(f, ",{}", index_reg)?;
            if self.scale > 1 {
                write!(f, ",{}", self.scale)?;
            }
        }

        write!(f, "]")
    }
}

impl Amd64Instruction {
    fn new(mnemonic: &str, operands: Vec<Operand>) -> Self {
        Amd64Instruction {
            mnemonic: mnemonic.to_string(),
            operands,
        }
    }
}

impl fmt::Display for Amd64Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.mnemonic)?;

        if !self.operands.is_empty() {
            write!(f, "\t")?;
            for (index, operand) in self.operands.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", operand)?;
            }
        }

        Ok(())
    }
}

enum Data {
    Int(i64),
    UInt(u64),
    USize(usize),
    Float(f64),
    Bytes(Vec<u8>),
}

enum AsmExpr {
    Data(Data),
    Instruction(Amd64Instruction),
    Block(Vec<AsmExpr>),
    Label(Label),
    Raw(String),
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Data::Float(v) => write!(f, "dq {}", v),
            Data::Int(v) => write!(f, "dq {}", v),
            Data::UInt(v) => write!(f, "dq {}", v),
            Data::USize(v) => write!(f, "dq {}", v),
            Data::Bytes(v) => {
                let formatted_bytes = v
                    .iter()
                    .map(|&byte| format!("0x{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "db {}", formatted_bytes)
            }
        }
    }
}

impl fmt::Display for AsmExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AsmExpr::Data(data) => write!(f, "\t\t{}", data),
            AsmExpr::Instruction(inst) => write!(f, "\t\t{}", inst),
            AsmExpr::Label(lbl) => write!(f, "\t{}", lbl),
            AsmExpr::Raw(str) => write!(f, "{}", str),
            AsmExpr::Block(lines) => {
                for line in lines {
                    write!(f, "{}\n", line)?;
                }
                Ok(())
            }
        }
    }
}
struct Section {
    name: String,
    body: Vec<AsmExpr>,
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "section .{}", self.name)?;

        if !self.body.is_empty() {
            for (_, line) in self.body.iter().enumerate() {
                write!(f, "{}\n", line)?;
            }
        }

        Ok(())
    }
}

impl Section {
    fn new(name: &str, body: Vec<AsmExpr>) -> Self {
        Section {
            name: name.to_string(),
            body: body,
        }
    }
}

macro_rules! datastring {
    ($label:expr, $data:expr) => {
        vec![
            AsmExpr::Label(Label::hashed($label)),
            AsmExpr::Data(Data::Bytes($data.as_bytes().to_vec())),
            AsmExpr::Raw(format!("\t{} equ $ - {}", Label::hashed(&format!("S_{}", $label)).label, Label::hashed($label).label)),
        ]
    };
}

// Example usage:
fn main() {
    let globals = vec![Global::new("_start")];

    let section_data = Section::new(
        "data",
        vec![
            AsmExpr::Block(datastring!("helloWorldStr", "This is a test of my macroassembler"))
        ],
    );

    let section_text = Section::new(
        "text",
        vec![
            AsmExpr::Label(Label::plain("_start")),
            AsmExpr::Instruction(Amd64Instruction::new(
                "mov",
                vec![
                    Operand::Register(Amd64Register::Special(Amd64SpecialRegister::RAX)),
                    Operand::Immediate(ImmediateValue::I64(1)),
                ],
            )),
            AsmExpr::Instruction(Amd64Instruction::new(
                "mov",
                vec![
                    Operand::Register(Amd64Register::Special(Amd64SpecialRegister::RDI)),
                    Operand::Immediate(ImmediateValue::I64(1)),
                ],
            )),
            AsmExpr::Instruction(Amd64Instruction::new(
                "lea",
                vec![
                    Operand::Register(Amd64Register::Special(Amd64SpecialRegister::RSI)),
                    Operand::DataRef(LabelOffset {
                        label: Label::hashed("helloWorldStr"),
                        rel: None
                    })
                ],
            )),
            AsmExpr::Instruction(Amd64Instruction::new(
                "mov",
                vec![
                    Operand::Register(Amd64Register::Special(Amd64SpecialRegister::RDX)),
                    Operand::Immediate(ImmediateValue::Label(Label::hashed("S_helloWorldStr"))),
                ],
            )),
            AsmExpr::Instruction(Amd64Instruction::new("syscall", vec![])),
            AsmExpr::Instruction(Amd64Instruction::new(
                "mov",
                vec![
                    Operand::Register(Amd64Register::Special(Amd64SpecialRegister::RAX)),
                    Operand::Immediate(ImmediateValue::I64(60)),
                ],
            )),
            AsmExpr::Instruction(Amd64Instruction::new(
                "xor",
                vec![
                    Operand::Register(Amd64Register::Special(Amd64SpecialRegister::RDI)),
                    Operand::Register(Amd64Register::Special(Amd64SpecialRegister::RDI)),
                ],
            )),
            AsmExpr::Instruction(Amd64Instruction::new("syscall", vec![])),
        ],
    );

    for global in globals {
        println!("{}", global);
    }
    println!("{}", section_text);
    println!("{}", section_data);
}
