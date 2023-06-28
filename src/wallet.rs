use std::rc::Rc;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read};

use k256::ecdsa::SigningKey;
use thiserror::Error;

use crate::blockchain::Blockchain;
use crate::cli::{CLICommandExec, Command, Instruction};
use crate::crypto;
use crate::database::Database;
use crate::transaction::{Transaction, TxIn, TxOut, UTXO};

#[derive(Error, Debug)]
pub enum WalletError {
    Io(#[from] io::Error),
    InvalidTxSig,
    IndexOutOfRange,
    InvalidSigningKey,
    NotEnoughFunds,
    HexDecode(#[from] hex::FromHexError),
    CryptoError(#[from] crypto::CryptoError)
}

pub type Result<T> = std::result::Result<T, WalletError>;

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wallet error")
    }
}

pub struct Wallet {
    private_keys: Vec<[u8; 32]>,
    current_private_key: usize,
    storage_file_name: String,
    utxo : Vec<UTXO>,
    database: Rc<Database>,
}

// ------ General
impl Wallet {

    // ------ Public
    pub fn new(database: Rc<Database>, storage_file_name: String) -> Wallet {
        Wallet { private_keys: Vec::new(), current_private_key: 0, database, utxo: Vec::new(), storage_file_name }
    }

    pub fn initialize(&mut self) {
        if let Err(_) = self.get_keys_from_file() {
            panic!("Wallet was not initialized properly: error while getting keys from file.")
        }
        self.get_and_set_utxo();
    }

    // ------ Private
    // --- Keys management
    fn create_and_store_private_key(&mut self) -> Result<()> {
        let private_key = self.generate_private_key();
        self.store_private_key(private_key)?;
        Ok(())
    }

    fn generate_private_key(&mut self) -> String {
        let signing_key = crypto::create_signing_key();
        let private_key = crypto::get_private_key(&signing_key);
        self.private_keys.push(private_key);
        hex::encode(private_key)
    }

    fn get_signing_key(&self, index: usize) -> Result<SigningKey> {
        if index < self.private_keys.len() {
            if let Ok(signing_key) = SigningKey::from_slice(&self.private_keys[index]) {
                return Ok(signing_key);
            }
            return Err(WalletError::InvalidSigningKey);
        }
        Err(WalletError::IndexOutOfRange)
    }

    // Create private key if file is empty, otherwise add keys into 'private_keys'
    fn get_keys_from_file(&mut self) -> Result<()> {
        let mut buffer = String::new();
        self.read_file(&mut buffer)?;

        if buffer.lines().count() == 0 {
            let private_key = self.generate_private_key();
            self.store_private_key(private_key)?;
        } else {
            // If file contains some keys, check if 'private_keys' contains them, them add them
            for line in buffer.lines() {
                self.add_key_to_wallet(line)?;
            }
        }

        Ok(())
    }

    fn add_key_to_wallet(&mut self, line: &str) -> Result<()> {
        let mut key = [0u8; 32];
        key.copy_from_slice(hex::decode(line)?.as_slice());
        if !self.private_keys.contains(&key) {
            self.private_keys.push(key);
        }
        Ok(())
    }

    fn store_private_key(&self, key: String) -> Result<()> {
        let mut file = self.get_file()?;
        writeln!(file, "{}", key)?;
        Ok(())
    }


    // --- Transaction management
    fn create_transaction(&self, amount: f32, destination: [u8; 20]) -> Result<Transaction> {
        let inputs_total_amount = self.utxo[0].amount + self.utxo[1].amount;

        match self.get_public_key_hash() {
            Ok(wallet_pub_key_hash) => {
                if inputs_total_amount > amount {
                    let inputs = vec![
                        TxIn::new(
                            0,
                            hex::encode(self.get_public_key(self.current_private_key).unwrap()),
                            self.utxo[0].reference
                        ),
                        TxIn::new(
                            0,
                            hex::encode(self.get_public_key(self.current_private_key).unwrap()),
                            self.utxo[1].reference
                        )
                    ];

                    let outputs = vec![
                        TxOut::new(amount, destination),
                        TxOut::new(inputs_total_amount-amount, wallet_pub_key_hash)
                    ];

                    return Ok(Transaction::new(inputs, outputs));
                }

                return Err(WalletError::NotEnoughFunds)
            }
            Err(e) => Err(e)
        }
    }

    fn sign_tx(&self, tx: &mut Transaction) -> Result<()> {
        if let Ok(signing_key) = self.get_signing_key(self.current_private_key) {
            // Transaction data
            let mut transaction_data_buffer = [0u8; 32];
            crypto::calculate_sha256_hash(tx.get_transaction_data(false).as_bytes(), &mut transaction_data_buffer);
            // Signature
            let signature = crypto::get_signature(&signing_key, &transaction_data_buffer);
            // Signature check
            let public_key = crypto::get_public_key(&signing_key);
            if crypto::verify_signature(public_key.as_slice(), signature.as_slice(), &transaction_data_buffer).unwrap() {
                tx.set_signature(hex::encode(signature));
                return Ok(());
            }
        }

        //Err("Error: could not sign transaction")
        Err(WalletError::InvalidSigningKey)
    }

    fn get_and_set_utxo(&mut self) {
        self.utxo.push(UTXO::new([1u8; 32], 123, 10.0));
        self.utxo.push(UTXO::new([2u8; 32], 123, 5.0));
    }


    // --- Private keys file management
    fn read_file(&self, buffer: &mut String) -> Result<()> {
        let mut file = self.get_file()?;
        file.read_to_string( buffer)?;
        Ok(())
    }

    fn get_file(&self) -> Result<File> {
        match OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("keys.txt") {
            Ok(f) => Ok(f),
            Err(e) => Err(WalletError::Io(e)),
        }
    }
}


// ------ Getters/Setters
impl Wallet {
    pub fn print_private_keys(&self) {
        self.private_keys.iter().for_each( |key|
            println!("{}", hex::encode(key))
        );
    }

    pub fn get_address(&self, index: usize) -> Result<String> {
        if index < self.private_keys.len() {
            let signing_key = self.get_signing_key(index);
            if let Ok(result) = signing_key {
                return Ok(crypto::get_address(result));
            }
            return Err(WalletError::InvalidSigningKey);
        }
        Err(WalletError::IndexOutOfRange)
    }

    pub fn get_private_key(&self, index: usize) -> Option<[u8; 32]> {
        if index < self.private_keys.len() {
            return Some(self.private_keys[index]);
        }
        None
    }

    pub fn get_public_key_hash(&self) -> Result<[u8; 20]> {
        match self.get_address(self.current_private_key) {
            Ok(address) => {
                let pub_key_hash = crypto::address_to_public_key_hash(&address)?;
                Ok(pub_key_hash)
            },
            Err(e) => Err(e)
        }
    }

    pub fn get_public_key(&self, index: usize) -> Option<Vec<u8>> {
        if let Ok(signing_key) = self.get_signing_key(index) {
            return Some(crypto::get_public_key(&signing_key));
        }
        None
    }
}


// ------ Instruction execution
impl CLICommandExec for Wallet {
    fn execute(&mut self, instruction: Instruction) {
        match instruction.command {
            Command::NEWPRIVATEKEY  => self.cli_new_private_key(),
            Command::GETADDRESS     => self.cli_get_address(instruction),
            Command::SHOWUTXO       => self.cli_show_utxo(),
            Command::SEND           => self.cli_send(instruction),

            _ => (),
        };
    }
}

impl Wallet {
    fn cli_new_private_key(&mut self) {
        if let Err(_) = self.create_and_store_private_key() {
            println!("Error: failed storing generated private key");
        }
    }

    fn cli_get_address(&self, instruction: Instruction) {
        let mut index = 0;
        if instruction.args.len() > 0 {
            match instruction.args[0].parse() {
                Ok(i) => index = i,
                Err(_) => { println!("Please enter a valid index"); return; }
            }
        }
        let address = self.get_address(index);
        match address {
            Ok(addr) => println!("Address: {}", addr),
            Err(e) => println!("Error: {}", e)
        };
    }

    fn cli_send(&self, instruction: Instruction) {
        if instruction.args.len() > 1 {
            // Check if amount was correctly typed
            if let Ok(amount) = instruction.args[0].parse::<f32>() {
                // Check if address is valid and convert it to public key hash
                if let Ok(destination) = crypto::address_to_public_key_hash(&instruction.args[1]) {
                    match self.create_transaction(amount, destination) {
                        Ok(mut transaction) => {
                            // Sign Transaction
                            if let Err(e) = self.sign_tx(&mut transaction) {
                                println!("{e}");
                                return;
                            }

                            transaction.hash();
                            println!("{}", transaction);
                        },
                        Err(e) => println!("{e}")
                    }
                } else {
                    println!("Please, provide a valid address");
                }
            } else {
                println!("Please, provide a valid amount");
            }
        } else {
            println!("Wrong number of arguments");
        }
    }

    fn cli_show_utxo(&self) {
        self.utxo.iter().for_each( |tx| println!("{}", tx))
    }
}


#[cfg(test)]
mod tests {
    use crate::transaction::{TxIn, TxOut};
    use super::*;

    //#[test]
    fn test_wallet_creation() {
        let mut wallet = Wallet::new(Rc::new(Database::open("database-test").unwrap()), String::from("keys.txt"));
        wallet.initialize();

        assert_eq!(wallet.get_address(0).unwrap(), crypto::get_address(SigningKey::from_slice(&wallet.get_private_key(0).unwrap()).unwrap()))
    }

    //#[test]
    fn test_wallet_creation_from_file() {
        let mut wallet = Wallet::new(Rc::new(Database::open("database-test").unwrap()), String::from("keys.txt"));
        wallet.initialize();
        wallet.create_and_store_private_key();
        println!("{}", wallet.get_address(0).unwrap());
        println!("{}", wallet.get_address(1).unwrap());

        assert_eq!(wallet.get_address(1).unwrap(), crypto::get_address(SigningKey::from_slice(&wallet.get_private_key(1).unwrap()).unwrap()))
    }

    //#[test]
    fn test_transaction_signature() {
        let mut wallet = Wallet::new(Rc::new(Database::open("database-test").unwrap()), String::from("keys.txt"));
        wallet.initialize();

        // Create test Transaction
        let inputs = vec![
            TxIn::new(0, String::from("04d2bb60cc37f89b5b07ea53724cd198acb5223b72ba98017278a428fdace203aedb21e038e8f7546a6d45e30737ad2d85236e187ee30f01bcb2aee6e94a3f143c"), [0u8; 32]),
            TxIn::new(1, String::from("04d2bb60cc37f89b5b07ea53724cd198acb5223b72ba98017278a428fdace203aedb21e038e8f7546a6d45e30737ad2d85236e187ee30f01bcb2aee6e94a3f143c"), [1u8; 32])
        ];
        let outputs = vec![
            TxOut::new(10.0, [3u8; 20]),
            TxOut::new(5.0, [4u8; 20]),
        ];
        let mut transaction = Transaction::new(inputs, outputs);

        wallet.sign_tx(&mut transaction).expect("Could not sign transaction");

        let mut transaction_data_buffer = [0u8; 32];
        crypto::calculate_sha256_hash(transaction.get_transaction_data(false).as_bytes(), &mut transaction_data_buffer);


        assert!(crypto::verify_signature(wallet.get_public_key(0).unwrap().as_slice(), hex::decode(transaction.get_signature()).unwrap().as_slice(), &transaction_data_buffer).unwrap());
    }
}