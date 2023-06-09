
mod transaction;
mod block;
mod utils;
mod miner;
mod blockchain;
mod wallet;
mod testing;
mod application;
mod cli;
mod database;

use crate::application::Application;
use crate::cli::CLI;
use crate::wallet::Wallet;

fn main() {
    let mut app = Application::new(2);
    app.run();
}
