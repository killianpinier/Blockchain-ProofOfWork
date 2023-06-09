use crate::transaction::{TxIn, TxOut, Transaction};


fn generate_tx_inputs(n: usize) -> Vec<TxIn> {
    let mut input_transactions = Vec::new();

    for i in 0..n {
        let tx_input = TxIn::new(0, format!("{}{}", "PublicKey-", i.to_string()), [0u8; 32]);
        input_transactions.push(tx_input);
    }

    input_transactions
}

fn generate_tx_outputs(n: usize) -> Vec<TxOut> {
    let mut output_transactions = Vec::new();

    for i in 0..n {
        let tx_output = TxOut::new(10.0, [0u8; 20]);
        output_transactions.push(tx_output);
    }

    output_transactions
}

pub fn generate_tx(in_n: usize, out_n: usize) -> Transaction {
    let tx_in = generate_tx_inputs(in_n);
    let tx_out = generate_tx_outputs(out_n);

    Transaction::new(tx_in, tx_out)
}