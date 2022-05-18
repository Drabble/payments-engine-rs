//! # CSV Reader utilities for transactions.

use std::error::Error;
use std::{fmt, io};
use csv;
use csv::Trim;
use serde::{Deserialize};
use crate::Transaction;
use crate::transaction_manager::TransactionType;

/// CSV reader for transaction files.
///
/// `csv::Reader` uses a `BufReader` internally which will read large parts of the file at once
/// and store them into a buffer until they are consumed. This should suffice for big files as we won't
/// load the entire file at once.
pub struct CsvReader<R: io::Read> {
    csv_reader: csv::Reader<R>,
}

impl<R: io::Read> CsvReader<R> {
    pub fn new(reader: R) -> CsvReader<R> {
        CsvReader {
            csv_reader: csv::ReaderBuilder::new()
                .has_headers(true) // Include headers
                .delimiter(b',') // Delimited by commas
                .trim(Trim::All) // Ignore all whitespaces
                .flexible(true) // Allow records of unequal length
                .from_reader(reader)
        }
    }

    /// Retrieve the next transaction in the csv.
    pub fn next(&mut self) -> Result<Option<Transaction>, Box<dyn Error>> {
        if let Some(record) = self.csv_reader.deserialize().next() {
            let record: Record = record?; // Deserialization
            return Ok(Some(record.to_transaction()?));
        }
        Ok(None)
    }
}

#[derive(Debug)]
struct CsvReaderError(String);

impl std::error::Error for CsvReaderError {}

impl fmt::Display for CsvReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize)]
struct Record {
    #[serde(rename = "type")]
    transaction_type: RecordType,
    client: u16,
    tx: u32,
    amount: Option<f64>,
}

impl Record {
    pub fn to_transaction(self) -> Result<Transaction, Box<dyn Error>> {
        let transaction = match self.transaction_type {
            RecordType::Deposit => {
                Transaction::new(
                    TransactionType::Deposit { amount: self.amount.ok_or_else(|| CsvReaderError(String::from("Missing amount for deposit")))? },
                    self.client,
                    self.tx,
                )
            }
            RecordType::Withdrawal => {
                Transaction::new(
                    TransactionType::Withdrawal { amount: self.amount.ok_or_else(|| CsvReaderError(String::from("Missing amount for withdrawal")))? },
                    self.client,
                    self.tx,
                )
            }
            RecordType::Dispute => {
                Transaction::new(
                    TransactionType::Dispute,
                    self.client,
                    self.tx,
                )
            }
            RecordType::Resolve => {
                Transaction::new(
                    TransactionType::Resolve,
                    self.client,
                    self.tx,
                )
            }
            RecordType::Chargeback => {
                Transaction::new(
                    TransactionType::Chargeback,
                    self.client,
                    self.tx,
                )
            }
        };
        Ok(transaction)
    }
}

#[derive(Debug, Deserialize)]
enum RecordType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}