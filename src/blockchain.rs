// Blockchain is for testing only
// It will later be replaced by a database

use std::collections::LinkedList;
use std::fmt;

use crate::block::Block;
use crate::crypto;

const INITIAL_MINING_REWARD: f32 = 25.0;
const PUB_KEY_HASH_SIZE: usize = 20;

pub struct Blockchain {
    chain: LinkedList<Block>,
    difficulty: u8,
    reward: f32
}


impl Blockchain {
    pub fn new(difficulty: u8) -> Blockchain {
        let mut blockchain = Blockchain{chain: LinkedList::new(), difficulty, reward: INITIAL_MINING_REWARD};
        let mut genesis = Block::new();

        genesis.set_index(0);
        genesis.mine(difficulty, INITIAL_MINING_REWARD, crypto::address_to_public_key_hash(&String::from("128GaUUoKKnEgioDsm5Pa9FxmXtzQMk3F9")).unwrap())
            .expect("Could not add genesis block");
        blockchain
    }

    pub fn mine_and_add_block(&mut self, mut block: Block, miner_addr: [u8; PUB_KEY_HASH_SIZE]) -> bool {
        // block.set_index(self.chain.len() as u32);
        // block.set_prev_hash_from_block(self.chain.back());
        // block.mine(self.difficulty, self.reward, miner_addr);
        //
        // self.chain.push_back(block);
        //
        // true
        true
    }

    pub fn get_last_block(&self) -> Option<&Block> {
        self.chain.back()
    }
}


impl Blockchain {
    pub fn get_difficulty(&self) -> u8 {
        self.difficulty
    }
}


impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Blockchain {{")?;
        writeln!(f, "    difficulty: {},", self.difficulty)?;
        writeln!(f, "    chain: [")?;
        self.chain.iter().for_each(|b| {writeln!(f, "        {}", b); });
        writeln!(f, "    ],")?;
        writeln!(f, "}}")
    }
}