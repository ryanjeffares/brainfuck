use std::error::Error;
use std::{env, io::{stdin, stdout, Write}};

use console::Term;

const DATA_SIZE: usize          = 30000;

const INCREMENT_DP: char        = '>';
const DECREMENT_DP: char        = '<';
const INCREMENT_DP_VALUE: char  = '+';
const DECREMENT_DP_VALUE: char  = '-';
const OUTPUT_DP: char           = '.';
const INPUT_DP: char            = ',';
const JUMP_FORWARD: char        = '[';
const JUMP_BACK: char           = ']';

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(),
        2 => {
            if !args[1].ends_with(".bf") {
                eprintln!("Error: file {} was not a `.bf` file.", args[1]);
            } else {
                let result = run_file(&args[1]);
                if result.is_err() {
                    eprintln!("Error reading file: {}", result.err().unwrap());
                }
            }
        },
        _ => usage(),
    }
}

fn repl() {
    println!("Welcome to brainfuck!");
    let mut interpreter = Interpreter::<DATA_SIZE>::new(); 
    loop {
        // println!();
        print!("> ");
        stdout().flush().unwrap();

        let mut buffer = String::new();
        match stdin().read_line(&mut buffer) {
            Ok(_) => {
                interpreter.compile(buffer)
            }
            Err(error) => println!("Error: {error}"),
        }
    }
}

fn run_file(file_path: &String) -> Result<(), Box<dyn Error>> {
    let text = std::fs::read_to_string(file_path)?.parse()?;
    let mut interpreter = Interpreter::<DATA_SIZE>::new();
    interpreter.compile(text);
    Ok(())
}

fn usage() {
    println!("Brainfuck\n\
        \n\
        Usage:\n\
        \n\
        brainfuck [file]\n\
        "
        );
}

#[derive(PartialEq)]
enum Op {
    IncrementDp,
    DecrementDp,
    IncrementDpValue,
    DecrementDpValue,
    OutputDp,
    InputDp,
    JumpForward,
    JumpBackward,
}

struct JumpPosition {
    start: usize,
    end: usize,
}

struct Interpreter<const N: usize> {
    data: [i8; N],
    data_pointer: usize,
    inst_pointer: usize,
    op_list: Vec<Op>,
    jump_positions: Vec<JumpPosition>, 
}

impl<const N: usize> Interpreter<N> {
    fn new() -> Self {
        Interpreter {
            data: [0; N],
            data_pointer: 0,
            inst_pointer: 0,
            op_list: Vec::<Op>::new(),
            jump_positions: Vec::<JumpPosition>::new(),
        }
    }

    fn compile(&mut self, code: String) {
        self.op_list.clear();

        let chars = code.as_bytes();
        for c in chars {
            match *c as char {
                INCREMENT_DP =>         self.op_list.push(Op::IncrementDp),
                DECREMENT_DP =>         self.op_list.push(Op::DecrementDp),
                INCREMENT_DP_VALUE =>   self.op_list.push(Op::IncrementDpValue),
                DECREMENT_DP_VALUE =>   self.op_list.push(Op::DecrementDpValue),
                OUTPUT_DP =>            self.op_list.push(Op::OutputDp),
                INPUT_DP =>             self.op_list.push(Op::InputDp),
                JUMP_FORWARD =>         self.op_list.push(Op::JumpForward),
                JUMP_BACK =>            self.op_list.push(Op::JumpBackward),
                _ => (),
            }
        }

        if !self.validate_jumps() {
            eprintln!("Execution stopped due to mismatched jump instructions.");
        } else {
            let res = self.run();
            if !res {
                eprintln!("Error occured during execution.");
            }
        }
    }

    fn validate_jumps(&mut self) -> bool {
        self.jump_positions.clear();
        let mut stack = Vec::<(Op, usize)>::new();
        
        for (index, op) in self.op_list.iter().enumerate() {
            match *op {
                Op::JumpForward => stack.push((Op::JumpForward, index)),
                Op::JumpBackward => {
                    // there should be a JumpForward on the top of the stack
                    if stack.is_empty() || stack.last().unwrap().0 != Op::JumpForward {
                        eprintln!("Found mismatched jump instruction at Op {index}.");
                        return false;
                    }

                    let start = stack.pop().unwrap();
                    self.jump_positions.push(JumpPosition {
                        start: start.1,
                        end: index,
                    });
                }
                _ => (),
            }
        } 

        stack.is_empty()
    }

    fn run(&mut self) -> bool {
        self.inst_pointer = 0;

        while self.inst_pointer < self.op_list.len() {
            match self.op_list[self.inst_pointer] {
                Op::IncrementDp => {
                    self.increment_dp();
                    self.inst_pointer += 1;
                }
                Op::DecrementDp => {
                    self.decrement_dp();
                    self.inst_pointer += 1;
                }
                Op::IncrementDpValue => {
                    self.increment_dp_value();
                    self.inst_pointer += 1;
                }
                Op::DecrementDpValue => {
                    self.decrement_dp_value();
                    self.inst_pointer += 1;
                }
                Op::OutputDp => {
                    self.output_dp();
                    self.inst_pointer += 1;
                }
                Op::InputDp => {
                    self.input_dp();
                    self.inst_pointer += 1;
                }
                Op::JumpForward => {
                    if !self.jump_forward() {
                        return false;
                    }
                } 
                Op::JumpBackward => {
                    if !self.jump_backward() {
                        return false;
                    }
                }
            }
        }

        true
    }
    

    #[inline]
    fn increment_dp(&mut self) {
        if self.data_pointer == DATA_SIZE - 1 {
            panic!("Cannot increment data pointer above data size {DATA_SIZE}.");
        }
        self.data_pointer += 1;
    }

    #[inline]
    fn decrement_dp(&mut self) {
        if self.data_pointer == 0 {
            panic!("Cannot decrement data pointer below 0.");
        }
        self.data_pointer -= 1;
    }

    #[inline]
    fn increment_dp_value(&mut self) {
        self.data[self.data_pointer] = match self.data[self.data_pointer] {
            i8::MAX => i8::MIN,
            _ => self.data[self.data_pointer] + 1,  
        };
    }

    #[inline]
    fn decrement_dp_value(&mut self) {
        self.data[self.data_pointer] = match self.data[self.data_pointer] {
            i8::MIN => i8::MAX,
            _ => self.data[self.data_pointer] - 1,  
        };
    }

    fn input_dp(&mut self) {
        let term = Term::stdout();
        match term.read_char() {
            Ok(c) => self.data[self.data_pointer] = c as i8,
            Err(e) => panic!("Invalid character input: {e}"),
        } 
    }

    #[inline]
    fn output_dp(&self) {
        println!("{}", self.data[self.data_pointer]);
    }

    fn jump_forward(&mut self) -> bool {
        if self.data[self.data_pointer] == 0 {
            let jp = self.jump_positions.iter().find(|&j| j.start == self.inst_pointer);
            if let Some(jump) = jp {
                self.inst_pointer = jump.end + 1;
            } else {
                eprintln!("No jump position found for jump at Op index {}, there was a problem during compilation.", self.inst_pointer);
                return false;
            }
        } else {
            self.inst_pointer += 1;
        }

        true
    }

    fn jump_backward(&mut self) -> bool {
        if self.data[self.data_pointer] != 0 {
            let jp = self.jump_positions.iter().find(|&j| j.end == self.inst_pointer);
            if let Some(jump) = jp {
                self.inst_pointer = jump.start + 1;
            } else {
                eprintln!("No jump position found for jump at Op index {}, there was a problem during compilation.", self.inst_pointer);
                return false;
            }
        } else {
            self.inst_pointer += 1;
        }

        true
    }
}
