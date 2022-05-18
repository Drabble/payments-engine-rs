//! # CSV Writer utilities for client accounts.

use std::error::Error;
use csv;
use csv::{IntoInnerError, Writer};
use serde::{Serialize};
use crate::transaction_manager::{ClientAccount};

/// CSV writer for client accounts.
pub struct CsvWriter<W: std::io::Write> {
    csv_writer: csv::Writer<W>,
}

impl<W: std::io::Write> CsvWriter<W> {
    pub fn new(writer: W) -> CsvWriter<W> {
        CsvWriter {
            csv_writer: csv::WriterBuilder::new()
                .has_headers(true)
                .delimiter(b',')
                .double_quote(false)
                .flexible(true)
                .from_writer(writer)
        }
    }

    /// Write a single client account to the csv.
    pub fn write(&mut self, client_account: &ClientAccount) -> Result<(), Box<dyn Error>> {
        self.csv_writer.serialize(Record::new(client_account))?;
        Ok(())
    }

    pub fn into_inner(
        self,
    ) -> Result<W, IntoInnerError<Writer<W>>> {
        self.csv_writer.into_inner()
    }
}

#[derive(Serialize)]
struct Record {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool
}

impl Record {
    pub fn new(
        client_account: &ClientAccount
    ) -> Record {
        Record {
            client: client_account.client,
            available: limit_to_4_decimals(client_account.available),
            held: limit_to_4_decimals(client_account.held),
            total: limit_to_4_decimals(client_account.available + client_account.held),
            locked: client_account.locked
        }
    }
}

/// Limit the given float 64 to 4 decimals.
fn limit_to_4_decimals(val: f64) -> f64{
    f64::trunc(val  * 10000.0) / 10000.0
}