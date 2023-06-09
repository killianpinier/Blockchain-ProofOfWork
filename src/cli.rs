use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::str::SplitWhitespace;

pub trait CLICommandExec {
    fn execute(&mut self, instruction: Instruction);
}

#[derive(Debug)]
pub enum Program {
    WALLET,
    MINER,

    NONE,
}

#[derive(Debug)]
pub enum Command {
    // Wallet
    NEWPRIVATEKEY,
    GETADDRESS,
    SEND,
    SHOWUTXO,

    // Miner
    START,
    STOP,
    SHOWTXPOOL,

    NONE,
}

#[derive(Debug)]
pub struct Instruction {
    pub program: Program,
    pub command: Command,
    pub args: Vec<String>,
    pub options: HashSet<char>,
}

impl Instruction {
    pub fn new() -> Instruction {
        Instruction{
            program: Program::NONE,
            command: Command::NONE,
            args: vec![],
            options: HashSet::new(),
        }
    }
}

pub struct CLI {
    cli_name: String
}

impl CLI {
    pub fn new(cli_name: String) -> CLI {
        CLI{ cli_name }
    }

    pub fn get_instruction(&self) -> Option<Instruction> {
        print!("{}> ", self.cli_name);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => self.parse_instruction(input),
            Err(e) => {
                println!("Error: {e}");
                return None;
            }
        }
    }

    fn parse_instruction(&self, input: String) -> Option<Instruction> {
        let mut words = input.split_whitespace();
        let mut instruction: Instruction = Instruction::new();
        match words.next() {
            Some(word) => self.parse_command(&mut instruction, word),
            None => return None
        }
        self.parse_args_and_options(&mut instruction, &mut words);

        Some(instruction)
    }

    fn parse_command(&self, instruction: &mut Instruction, word: &str) {
        let command = self.assign_word_to_command(word);
        instruction.program = command.0;
        instruction.command = command.1;
    }

    fn parse_args_and_options(&self, instruction: &mut Instruction, words: &mut SplitWhitespace) {
        let mut dashes_passed = false;
        for w in words {
            if w.contains("--") && !dashes_passed {
                dashes_passed = true;
            }

            if dashes_passed {
                for c in w.chars() {
                    if c != '-' {
                        instruction.options.insert(c);
                    }
                }
            } else {
                instruction.args.push(w.to_string());
            }
        }
    }

    fn assign_word_to_command(&self, word: &str) -> (Program, Command) {
        match word {
            // Wallet
            "newprivatekey" => (Program::WALLET, Command::NEWPRIVATEKEY),
            "getaddress"    => (Program::WALLET, Command::GETADDRESS),
            "showutxo"      => (Program::WALLET, Command::SHOWUTXO),
            "send"          => (Program::WALLET, Command::SEND),

            // Miner
            "start"         => (Program::MINER, Command::START),
            "stop"          => (Program::MINER, Command::STOP),
            "showtxpool"    => (Program::MINER, Command::SHOWTXPOOL),

            _ => (Program::NONE, Command::NONE)
        }
    }

}