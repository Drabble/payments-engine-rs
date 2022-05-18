//! # Payment engine cli
//!
//! The command takes a single filename for a csv file with the following format:
//!
//! > type,client,tx,amount
//! > deposit,1,1,1.0
//! > withdrawal,1,1,1.0
//! > dispute,1,1,
//! > resolve,1,1,
//! > deposit,1,2,1.0
//! > withdrawal,1,2,1.0
//! > dispute,1,2,
//! > chargeback,1,2,
//!
//! > Check the library for more information.
//!
//! ## Example
//!
//! ```bash
//! cargo run -- transactions.csv
//! ```
//!

use std::{env};
use std::fs::File;
use std::io::Stdout;
use std::process;

use payments_engine_rs::{Config, run};

fn main() {
    let config = Config::<File, Stdout>::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}