/// Transaction.rs
///
/// A Transaction is the way that is used to distribute coins to different addresses over the network
/// A Transaction is composed of:
///     - TxIn (Transaction input): Used to reference an unspent transaction output (UTXO)
///     - TxOut (Transaction Output): Used to change ownership of some coins to another address
///
/// A Transaction is created by the Wallet, and shared to the network by the Miner
/// Once the Wallet has created a Transaction, it will be send to the Miner, which will verify if
///     the Transaction is correct (valid address, valid signature, referenced UTXO not already spent, etc).
///     If Transaction is validated by the Miner, it will be added to the transaction pool (see miner.rs for more
///     information on the how the Transaction is handled after it has been added to the transaction pool)
///
///
/// For now, the protocol only allows one address to sign transaction inputs

use std::fmt;

use serde::{Deserialize, Serialize};
use crate::crypto;

// Size in bytes
const TRANSACTION_HASH_SIZE: usize = 32;
const PUB_KEY_HASH_SIZE: usize = 20;

// Unspent transaction output
pub struct UTXO {
    pub reference: [u8; 32], // Transaction hash
    pub n: usize,
    pub amount: f32,
}

impl UTXO {
    pub fn new(reference: [u8; 32], n: usize, amount: f32) -> UTXO {
        UTXO{ reference, n, amount }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxIn {
    n: usize,
    prev_utxo: [u8; TRANSACTION_HASH_SIZE],
    public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOut {
    amount: f32,
    destination: [u8; PUB_KEY_HASH_SIZE], // Hash of the public key (Ripemd160(Sha256(PubKey)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    hash: [u8; TRANSACTION_HASH_SIZE],
    tx_in_sz: usize,
    tx_out_sz: usize,
    signature: String,

    inputs: Vec<TxIn>,
    outputs: Vec<TxOut>,
}


// ------ TxIn implementation
impl TxIn {
    pub fn new(n: usize, public_key: String, prev_utxo: [u8; TRANSACTION_HASH_SIZE]) -> TxIn {
        TxIn { n, prev_utxo, public_key }
    }
}


// ------ TxOut implementation
impl TxOut {
    pub fn new(amount: f32, destination: [u8; PUB_KEY_HASH_SIZE]) -> TxOut {
        TxOut { amount, destination }
    }
}


// ------ Transaction implementation
impl Transaction {

    // --- Public
    pub fn new(inputs: Vec<TxIn>, outputs: Vec<TxOut>) -> Transaction {
        let tx = Transaction {
            hash: [0u8; TRANSACTION_HASH_SIZE],
            tx_in_sz: inputs.len(),
            tx_out_sz: outputs.len(),
            signature: String::new(),
            inputs,
            outputs,
        };

        tx
    }

    // Calculate and set hash to transaction
    pub fn hash(&mut self) {
        crypto::calculate_sha256_hash(self.get_transaction_data(true).as_bytes(),&mut self.hash);
    }

    pub fn add_tx_input(&mut self, tx_in: TxIn) {
        self.inputs.push(tx_in);
    }
    pub fn add_tx_output(&mut self, tx_out: TxOut) {
        self.outputs.push(tx_out);
    }

    // Concatenate fields of self. If 'add_signature' set to true, signature is not used for the concatenation
    pub fn get_transaction_data(&self, add_signature: bool) -> String {
        let mut data = String::new();

        if add_signature {
            data.push_str(&self.signature);
        }

        data.push_str(&self.tx_in_sz.to_string());
        data.push_str(&self.tx_out_sz.to_string());

        data.push_str(&hex::encode(&self.calculate_inputs_hash()));
        data.push_str(&hex::encode(&self.calculate_outputs_hash()));

        data
    }

    pub fn set_signature(&mut self, signature: String) {
        self.signature = signature;
    }

    // --- Private
    fn get_concatenated_inputs(&self) -> String {
        let mut data = String::new();
        for input in &self.inputs {
            let mut cur_tx_in_hash = [0u8; TRANSACTION_HASH_SIZE];
            let cur_tx_in_data = input.n.to_string()
                + &hex::encode(input.prev_utxo)
                + &input.public_key;
            crypto::calculate_sha256_hash(cur_tx_in_data.as_bytes(), &mut cur_tx_in_hash);
            data.push_str(&hex::encode(cur_tx_in_hash));
        }

        data
    }

    fn get_concatenated_outputs(&self) -> String {
        let mut data = String::new();
        for output in &self.outputs {
            let mut cur_tx_out_hash = [0u8; TRANSACTION_HASH_SIZE];
            let cur_tx_out_data = output.amount.to_string() + &hex::encode(&output.destination);
            crypto::calculate_sha256_hash(cur_tx_out_data.as_bytes(), &mut cur_tx_out_hash);
            data.push_str(&hex::encode(cur_tx_out_hash));
        }

        data
    }

    fn calculate_inputs_hash(&self) -> [u8; TRANSACTION_HASH_SIZE] {
        let data = self.get_concatenated_inputs();
        let mut inputs_hash = [0u8; TRANSACTION_HASH_SIZE];

        crypto::calculate_sha256_hash(data.as_bytes(), &mut inputs_hash);
        inputs_hash
    }

    fn calculate_outputs_hash(&self) -> [u8; TRANSACTION_HASH_SIZE] {
        let data = self.get_concatenated_outputs();
        let mut outputs_hash = [0u8; TRANSACTION_HASH_SIZE];

        crypto::calculate_sha256_hash(data.as_bytes(), &mut outputs_hash);
        outputs_hash
    }
}

// ------ Getters/Setters
impl Transaction {
    // ---Getters
    pub fn get_hash(&self) -> &[u8; TRANSACTION_HASH_SIZE] {
        &self.hash
    }
    pub fn get_signature(&self) -> &String { &self.signature }
}


impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        if self.get_hash() == other.get_hash() {
            return true;
        }
        false
    }
}


impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tab = "                ";

        writeln!(f, "Transaction {{")?;
        writeln!(f, "{}    hash: {},", tab, hex::encode(self.hash))?;
        writeln!(f, "{}    signature: {},", tab, self.signature)?;
        writeln!(f, "{}    inputs: [", tab)?;
        self.inputs.iter().for_each(|tx| {
            writeln!(f, "{}        {{", tab);
            writeln!(f, "{}            n: {},", tab, tx.n);
            writeln!(f, "{}            prev_utxo: {},", tab, &hex::encode(tx.prev_utxo));
            writeln!(f, "{}            public_key: {},", tab, tx.public_key);
        });
        writeln!(f, "{}    ],", tab)?;
        writeln!(f, "{}    outputs: [", tab)?;
        self.outputs.iter().for_each(|tx| {
            writeln!(f, "{}        {{", tab);
            writeln!(f, "{}            amount: {},", tab, tx.amount);
            writeln!(f, "{}            destination: {},", tab, hex::encode(tx.destination));
            writeln!(f, "{}        }},", tab);
        });
        writeln!(f, "{}    ],", tab)?;
        writeln!(f, "{}}}", tab)
    }
}

impl fmt::Display for UTXO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TxOut {{")?;
        writeln!(f, "   reference: {}", hex::encode(self.reference))?;
        writeln!(f, "   n: {}", self.n)?;
        writeln!(f, "   amount: {}", self.amount)?;
        writeln!(f, "}}")
    }
}