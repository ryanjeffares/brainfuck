use std::error::Error;
use std::{env, io::{stdin, stdout, Write}, process};

use console::Term;  // read_char()

/// The size of the array of memory cells used by brainfuck.
/// This can be changed and recompiled to suit different needs.
/// Having this as a `const` allows us to use it as a const generic
/// and stack allocate the `Interpreter` struct.
const DATA_SIZE: usize          = 30000;

/// The 6 characters that will be interpreted as brainfuck code.

/// `>` increments the position of the data pointer by 1.
/// Incrementing the data pointer above `DATA_SIZE` results in a panic.
const INCREMENT_DP: char        = '>';
/// `<` decrements the position of the data pointer by 1.
/// Decrementing the data pointer below 0 results in a panic.
const DECREMENT_DP: char        = '<';
/// `+` increments the value of the byte at the data pointer by 1.
/// Incrementing a byte over `i8::MAX`, or 127, results in the value wrapping around to `i8::MIN`,
/// or -128.
const INCREMENT_DP_VALUE: char  = '+';
/// `-` decrements the value of the byte at the data pointer by 1.
/// Decrementing a byte below 0 results in the value wrapping around to `i8::MAX`,
/// or 127.
const DECREMENT_DP_VALUE: char  = '-';
/// `.` outputs the byte at the data pointer to the console.
const OUTPUT_DP: char           = '.';
/// `,` prompts the user to input a single character, which is written to the byte at the data
/// pointer.
const INPUT_DP: char            = ',';
/// `[` moves the instruction pointer forwards to the command after the matching `]` if the byte at the data
/// pointer is 0, or else the instruction pointer is incremented by 1.
const JUMP_FORWARD: char        = '[';
/// `]` moves the instruction pointer backwards to the command after the matching `[` if the byte
/// at the data pointer is non-zero, or else the instruction pointer is incremented by 1.
const JUMP_BACK: char           = ']';

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(),
        2 | 3 => {
            if !args[1].ends_with(".bf") {
                eprintln!("Error: file {} was not a `.bf` file.", args[1]);
                return;
            }

            if args.len() == 3 && (args[2] != "-v" && args[2] != "--verbose") {
                usage();
                return;
            }

            let verbose = args.len() == 3 && (args[2] == "-v" || args[2] == "--verbose");
            let result = run_file(&args[1], verbose);
            if result.is_err() {
                eprintln!("Error reading file: {}", result.err().unwrap());
            }
        }
        _ => usage(),
    }
}

/// Run the REPL.
/// Creates an instance of the Interpreter struct, and continually prompts the user to input a
/// line which is compiled and ran. 'exit' can be entered to exit the REPL.
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
                match buffer.trim() {
                    "exit" => process::exit(0),
                    _ => interpreter.compile(buffer, false),
                }
            }
            Err(error) => println!("Error: {error}"),
        }
    }
}

/// Read the given file, create and instance of the Interpreter struct and run the file.
/// Path given to this function has already been checked to be a `.bf` file, and any errors
/// encountered while reading the file are reported.
fn run_file(file_path: &String, verbose: bool) -> Result<(), Box<dyn Error>> {
    let text = std::fs::read_to_string(file_path)?.parse()?;
    let mut interpreter = Interpreter::<DATA_SIZE>::new();
    interpreter.compile(text, verbose);
    Ok(())
}

fn usage() {
    println!("Brainfuck\n\
        \n\
        Usage:\n\
        \n\
        brainfuck [file [-v/--verbose]]\n\
        "
        );
}

/// An enum to represent the 6 operations within brainfuck.
/// Any brainfuck program is compiled into a list of Ops, as a lightweight way to run through the
/// operations of the program.
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

/// A small struct to hold the start and end positions (within the list of operations) for matching
/// `[` and `]` pairs.
struct JumpPosition {
    start: usize,
    end: usize,
}

/// The Interpreter struct holds the array of memory cells, the data and instruction pointers, and
/// Vecs of Ops and JumpInstructions that are filled during compilation.
/// Using a const generic allows us to stack allocate the Interpreter while experimenting with
/// different sizes of the data array.
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

    /// Compile and run brainfuck code.
    fn compile(&mut self, code: String, verbose: bool) {
        let start = std::time::Instant::now();

        // Clearing the Op list is only necessary in the REPL,
        // so that the same Interpreter instance can be reused
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
                // any other character is ignored, so that brainfuck programs can contain whitespace and comments.                
                _ => (),
            }
        }

        if !self.validate_jumps() {
            eprintln!("Execution stopped due to mismatched jump instructions.");
        } else {
            if verbose {
                println!("Compilation succeeded in {:?}", start.elapsed());
            }
            let res = self.run();
            if !res {
                eprintln!("Error occured during execution.");
            }
        }
    }

    /// Validates jumps (`[` and `]`) by ensuring each jump forward instruction has exactly one
    /// corresponding jump backward instruction, and vice versa.
    fn validate_jumps(&mut self) -> bool {
        // In REPL mode, this needs to be cleared since the same Interpreter instance is reused.
        self.jump_positions.clear();

        // Use a Vec like a stack to validate the jumps    
        let mut stack = Vec::<(Op, usize)>::new();
        
        for (index, op) in self.op_list.iter().enumerate() {
            match *op {
                // Push a jump forward instruction and its index in the Op list to the top of the
                // stack                
                Op::JumpForward => stack.push((Op::JumpForward, index)),
                // When we come across a jump back instruction, there must be its corresponding
                // jump forward instruction at the top of the stack.  
                Op::JumpBackward => {
                    // checking if the stack is empty first means the calls to `unwrap()` are safe
                    if stack.is_empty() || stack.last().unwrap().0 != Op::JumpForward {
                        eprintln!("Found mismatched jump instruction at Op {index}.");
                        return false;
                    }

                    // now we know where the jump starts and ends
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

    /// Reset the instruction pointer to 0 and run the compiled list of instructions.
    fn run(&mut self) -> bool {
        self.inst_pointer = 0;

        // Jump instructions will move the instruction pointer around the program
        // and any other operation will increment it by 1.
        // So just run until the list of operations in exhausted.
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
        // Called when we encounter a jump forward instruction.
        // If the byte at the data pointer is 0, we need to jump just beyond the corresponding jump
        // back instruction. We can find the JumpInstruction struct from the Vec that has the same
        // `start` value as the current instruction pointer, so we move 1 beyond its `end`.
        if self.data[self.data_pointer] == 0 {
            let jp = self.jump_positions.iter().find(|&j| j.start == self.inst_pointer);
            if let Some(jump) = jp {
                self.inst_pointer = jump.end + 1;
            } else {
                eprintln!("No jump position found for jump at Op index {}, there was a problem during compilation.", self.inst_pointer);
                return false;
            }
        } else {
            // or else just increment by 1.
            self.inst_pointer += 1;
        }

        true
    }

    fn jump_backward(&mut self) -> bool {
        // Called when we encounter a jump backward instruction.
        // If the byte at the data pointer is non 0, we go back just beyond the corresponding jump
        // forward instruction.
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
