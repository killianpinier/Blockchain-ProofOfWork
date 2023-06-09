use std::rc::Rc;
use std::cell::RefCell;

use crate::miner::Miner;
use crate::wallet::Wallet;
use crate::blockchain::Blockchain;
use crate::cli::{CLI, Program, Command, CLICommandExec};

use crate::testing::tx_generation::{generate_tx};

pub struct Application {
    blockchain: Rc<RefCell<Blockchain>>,
    miner: Miner,
    wallet: Wallet,
}


impl Application {
    pub fn new(difficulty: u8) -> Application {
        let blockchain: Rc<RefCell<Blockchain>> = Rc::new(RefCell::new(Blockchain::new(difficulty)));

        // Create and initialize wallet
        let mut wallet: Wallet = Wallet::new(Rc::clone(&blockchain), String::from("keys.txt"));
        wallet.initialize();

        // Create and initialize miner
        let miner;
        if let Ok(address) = wallet.get_address(0) {
            miner = Miner::new(address.clone(), Rc::clone(&blockchain));
        } else {
            panic!("Wallet was not initialized properly: could not get default address")
        }

        Application { blockchain, miner, wallet }
    }


    pub fn run(&mut self) {

        let cli = CLI::new("bitcoin".to_string());
        let stop = false;

        while !stop {
            if let Some(instruction) = cli.get_instruction() {
                match instruction.program {
                    Program::WALLET => self.wallet.execute(instruction),
                    Program::MINER  => self.miner.execute(instruction),
                    Program::NONE   => (),
                }
            }
        }

        // // Block 1
        // let tx1 = generate_tx(2, 2);
        // let tx2 = generate_tx(1, 2);
        //
        // self.miner.add_tx_to_tx_pool(tx1);
        // self.miner.add_tx_to_tx_pool(tx2);
        // self.miner.mine();
        //
        // // Block 2
        // let tx1 = generate_tx(3, 2);
        // let tx2 = generate_tx(1, 3);
        //
        // self.miner.add_tx_to_tx_pool(tx1);
        // self.miner.add_tx_to_tx_pool(tx2);
        // self.miner.mine();
        //
        // println!("{}", self.blockchain.borrow())
    }

    // ------ Private
    fn create_and_initialize_wallet(&self) {

    }


}