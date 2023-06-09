use std::time::{SystemTime, UNIX_EPOCH, SystemTimeError};
use std::fmt;
use serde::Serialize;

use crate::{transaction::{Transaction, TxOut}, utils::{self, crypto::leading_zeros_count}};

const PUB_KEY_HASH_SIZE: usize = 20;

#[derive(Serialize)]
pub struct Block {
    index: u32,
    hash: [u8; 32],
    prev_hash: [u8; 32],
    timestamp: u128, // Time when mining starts
    merkle_root: [u8; 32],
    transactions: Vec<Transaction>,
    nonce: u32,
}


impl Block {

    pub fn new() -> Block{
        Block { 
            index: 0, 
            hash: [0; 32], 
            prev_hash: [0; 32],
            nonce: 0,  
            merkle_root: [0; 32],
            timestamp: 0, 
            transactions: Vec::new() 
        }
    }

    // --- Private
    fn concatenate(&self) -> String {
        let mut data = String::new();

        data.push_str(&self.index.to_string());                    // Index
        data.push_str(&hex::encode(&self.prev_hash));            // Previous hash
        data.push_str(&self.timestamp.to_string());             // Timestamp
        data.push_str(&hex::encode(&self.merkle_root));       // Merkle root
        data.push_str(&self.nonce.to_string());              // Nonce

        data
    }

    pub fn calculate_hash(&mut self) {
        let data = self.concatenate();
        utils::crypto::calculate_sha256_hash(data.as_bytes(), &mut self.hash );
    }

    fn mine_until_done(&mut self, difficulty: u8) {
        self.calculate_hash();

        while leading_zeros_count(&hex::encode(&self.hash)) < difficulty {
            self.nonce += 1;
            self.calculate_hash();
        }
    }


    // --- Public
    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
    }

    pub fn mine(&mut self, difficulty: u8, reward: f32, pub_key_hash: [u8; 20]) -> Result<(), &'static str> {
        if let Ok(time) = SystemTime::now().duration_since(UNIX_EPOCH) {
            self.timestamp = time.as_millis();
        } else {
            return Err("Error while mining block: could not get current time");
        }

        self.add_transaction(Transaction::new(Vec::new(), vec![TxOut::new(reward, pub_key_hash)]));
        self.mine_until_done(difficulty);
        Ok(())
    }

}


// --- Getters/Setters
impl Block {
    pub fn get_hash(&self) -> &[u8; 32] { &self.hash }
    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn set_index(&mut self, index: u32) {
        self.index = index;
    }

    pub fn set_prev_hash_from_block(&mut self, prev_block: Option<&Block>) -> bool {
        if let Some(prev_block) = prev_block {
            self.prev_hash = prev_block.hash;
        } else {
            return false;
        }

        true
    }
}


impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tab = "        ";

        writeln!(f, "Block {{")?;
        writeln!(f, "{}    index: {},", tab, self.index)?;
        writeln!(f, "{}    hash: {},", tab, hex::encode(self.hash))?;
        writeln!(f, "{}    prev_hash: {},", tab, hex::encode(self.prev_hash))?;
        writeln!(f, "{}    timestamp: {}", tab, self.timestamp)?;
        writeln!(f, "{}    transactions: [", tab)?;
        self.transactions.iter().for_each(|tx| {writeln!(f, "{}        {}", tab, tx); });
        writeln!(f, "{}    ],", tab)?;
        writeln!(f, "{}}}", tab)
    }
}