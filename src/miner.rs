use std::rc::Rc;
use std::cell::RefCell;

use crate::{transaction::Transaction, block::Block, rocks};
use crate::cli::{CLICommandExec, Command, Instruction};
use crate::crypto;
use crate::database::{BlockHashKeys, Database};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MinerError {
    MiningError,
    DatabaseError(#[from] rocks::DatabaseError)
}

type Result<T> = std::result::Result<T, MinerError>;

impl std::fmt::Display for MinerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "database error")
    }
}

pub struct Miner {
    address: String,
    pub_key_hash: [u8; 20],
    database: Rc<Database>,
    tx_pool: RefCell<Vec<Transaction>>,
    current_difficulty: u8,
    current_reward: f32,
}

impl Miner {
    pub fn new(address: String, database: Rc<Database>, difficulty: u8) -> Miner {
        if let Ok(pub_key_hash) = crypto::address_to_public_key_hash(&address) {
            return Miner{
                address,
                pub_key_hash,
                database,
                tx_pool: RefCell::new(Vec::new()),
                current_difficulty: difficulty,
                current_reward: 50.0
            }
        }
        panic!("Error while creating Miner: could not convert address to public key hash")
    }


    // --- Public

    // TODO: application should call this function once networking is implemented
    pub fn start() {

    }

    pub fn mine(&mut self) -> Result<()> {
        if let Some(last_block) = self.database.get_last_block()? {
            let mut block = Block::new();
            for tx in self.tx_pool.borrow().iter() {
                block.add_transaction((*tx).clone());
            }

            block.set_index(last_block.get_index() + 1);
            block.set_prev_hash_from_block(&last_block);

            if let Ok(_) = block.mine(self.current_difficulty, self.current_reward, self.pub_key_hash) {
                self.database.put_block(&block)?;
                self.clear_tx_pool();
                return Ok(());
            }
        }
        Err(MinerError::MiningError)
    }

    pub fn add_tx_to_tx_pool(&mut self, tx: Transaction) -> bool {
        if self.verify_tx(&tx) {
            self.tx_pool.borrow_mut().push(tx);
            return true
        }

        false
    }


    // --- Private

    fn verify_tx(&self, tx: &Transaction) -> bool {
        true
    }

    fn clear_tx_pool(&self) {
        // if let Some(block) = self.blockchain.borrow().get_last_block() {
        //     self.tx_pool.borrow_mut().retain(|tx| !block.get_transactions().contains(tx));
        // }
    }
}


// --- Instruction execution
impl CLICommandExec for Miner {
    fn execute(&mut self, instruction: Instruction) {
        match instruction.command {
            Command::START  => self.cli_start(),
            Command::STOP   => self.cli_stop(),

            _ => (),
        };
    }
}

impl Miner {
    fn cli_start(&self) {
        println!("Mining started")
    }

    fn cli_stop(&self) {
        println!("Mining stopped")
    }
}