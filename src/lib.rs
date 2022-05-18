//! # Payment Engine
//!
//! A payment engine that can process transactions from csv files, execute them and return
//! a list of client accounts with balances, etc.
//!
//! Each transaction has a type, a unique client id, a transaction id and an optional amount.
//! The different transaction types are listed below.
//!
//! * **deposit** - Deposit a certain amount into the client account.
//! * **withdrawal** - Withdraw a certain amount from the client account.
//! * **dispute** - Dispute the transaction with the given transaction id. Disputed funds are held
//! until they are released. You can only dispute a deposit with a valid transaction id otherwise
//! the dispute will be ignored. Additionally, you can only dispute a transaction once.
//! * **resolve** - Resolves a disputed transaction with a given transaction id.
//! * **chargeback** - Charges back the amount of a given transaction id from the client's balance.
//!
//! ## Example
//!
//! ```
//! use std::{io, process};
//! use payments_engine_rs::{Config, run};
//! let reader = "type,client,tx,amount\ndeposit,1,1,1.0".as_bytes();
//! let writer = io::stdout();
//! if let Err(e) = run(Config{reader, writer}) {
//!     eprintln!("Application error: {}", e);
//!     process::exit(1);
//! }
//! ```
//!

mod csv_reader;
mod transaction_manager;
mod csv_writer;

use std::error::Error;
use std::{env, fmt, io};
use std::fs::File;
use crate::transaction_manager::{Transaction, TransactionManager};

/// Stores the config required to run the payments engine.
///
/// The config includes a reader that can be used to read the csv file.
pub struct Config<R: io::Read, W: io::Write> {
    pub reader: R,
    pub writer: W,
}

impl<R: io::Read, W: io::Write> Config<R, W> {
    pub fn new(mut args: env::Args) -> Result<Config<File, io::Stdout>, Box<dyn Error>> {
        args.next();

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err(Box::new(ConfigFileError(String::from("Didn't get a file name")))),
        };
        let reader = File::open(filename)?;
        let writer = io::stdout();

        Ok(Config { reader, writer })
    }
}

#[derive(Debug)]
struct ConfigFileError(String);

impl std::error::Error for ConfigFileError {}

impl fmt::Display for ConfigFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Run the payments engine with the given configuration.
pub fn run<R: io::Read, W: io::Write>(config: Config<R, W>) -> Result<W, Box<dyn Error>> {
    let mut transaction_manager = TransactionManager::new();
    let mut csv_reader = csv_reader::CsvReader::new(config.reader);

    while let Some(transaction) = csv_reader.next()? {
        transaction_manager.process_transaction(transaction)?;
    }

    let mut csv_writer = csv_writer::CsvWriter::new(config.writer);
    for (_, client_account) in transaction_manager.client_account_index.iter() {
        csv_writer.write(&client_account)?;
    }

    Ok(csv_writer.into_inner().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_transactions() {
        let reader =
            "type,client,tx,amount\n\
            deposit,1,1,1.0\n\
            withdrawal,1,2,1.0\n\
            dispute,1,1,\n\
            resolve,1,1,\n\
            deposit,1,3,1.0\n\
            withdrawal,1,4,1.0\n\
            dispute,1,3,\n\
            chargeback,1,3,".as_bytes();
        let writer = run(Config { reader, writer: vec![] }).unwrap();
        assert_eq!("client,available,held,total,locked\n1,-1.0,0.0,-1.0,true\n", std::str::from_utf8(&writer).unwrap());
    }

    #[test]
    fn process_transactions_with_spaces() {
        let reader =
            "type, client, tx, amount\n\
             deposit, 1, 1, 1.0\n\
             withdrawal,  1,  2, 1.0\n".as_bytes();
        let writer = run(Config { reader, writer: vec![] }).unwrap();
        assert_eq!("client,available,held,total,locked\n1,0.0,0.0,0.0,false\n", std::str::from_utf8(&writer).unwrap());
    }

    #[test]
    fn process_transactions_with_4_decimals() {
        let reader =
            "type,client,tx,amount\n\
            deposit,1,1,1.9999\n\
            withdrawal,1,2,0.1111\n".as_bytes();
        let writer = run(Config { reader, writer: vec![] }).unwrap();
        assert_eq!("client,available,held,total,locked\n1,1.8888,0.0,1.8888,false\n", std::str::from_utf8(&writer).unwrap());
    }
}