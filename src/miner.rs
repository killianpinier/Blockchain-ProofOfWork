use std::rc::Rc;
use std::cell::{Ref, RefCell};

use crate::{transaction::Transaction, blockchain::Blockchain, block::Block};
use crate::cli::{CLICommandExec, Command, Instruction};
use crate::utils;

pub struct Miner {
    address: String,
    pub_key_hash: [u8; 20],
    blockchain: Rc<RefCell<Blockchain>>,
    tx_pool: RefCell<Vec<Transaction>>, // To make it easier to manage in clear_tx_pool
}

impl Miner {
    pub fn new(address: String, blockchain: Rc<RefCell<Blockchain>>) -> Miner {
        if let Ok(pub_key_hash) = utils::crypto::address_to_public_key_hash(&address) {
            return Miner{
                address,
                pub_key_hash,
                blockchain,
                tx_pool: RefCell::new(Vec::new())
            }
        }
        panic!("Error while creating Miner: could not convert address to public key hash")
    }


    // --- Public

    // TODO: application should call this function once networking is implemented
    pub fn start() {
        
    }

    pub fn mine(&mut self) {
        let mut block = Block::new();
        for tx in self.tx_pool.borrow().iter() {
            block.add_transaction((*tx).clone());
        }

        self.blockchain.borrow_mut().mine_and_add_block(block, self.pub_key_hash);
        self.clear_tx_pool();
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
        if let Some(block) = self.blockchain.borrow().get_last_block() {
            self.tx_pool.borrow_mut().retain(|tx| !block.get_transactions().contains(tx));
        }
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