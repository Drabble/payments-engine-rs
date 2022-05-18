//! # Transaction manager
//!
//! The transaction manager processes transactions and generates an index of client accounts.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// The state of the transactions.
#[derive(Debug)]
pub enum TransactionState {
    Executed,
    Disputed,
    Resolved,
    Chargedback
}

/// The types of transactions with related data.
#[derive(Debug)]
pub enum TransactionType {
    Deposit {
        amount: f64,
    },
    Withdrawal {
        amount: f64,
    },
    Dispute,
    Resolve,
    Chargeback,
}

/// The transaction model.
#[derive(Debug)]
pub struct Transaction {
    transaction_type: TransactionType,
    client: u16, // Client id
    tx: u32, // Transaction id
    state: TransactionState
}

impl Transaction {
    pub fn new(transaction_type: TransactionType, client: u16, tx: u32) -> Transaction {
        Transaction {
            transaction_type,
            client,
            tx,
            state: TransactionState::Executed
        }
    }
}

#[derive(Debug)]
pub struct ClientAccount {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub locked: bool,
    pub transaction_index: HashMap<u32, Transaction>,
}

impl ClientAccount {
    pub fn new(
        client: u16,
        available: f64,
        held: f64,
    ) -> ClientAccount {
        ClientAccount {
            client,
            available,
            held,
            locked: false,
            transaction_index: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct ClientAccountLockedError();

impl std::error::Error for ClientAccountLockedError {}

impl fmt::Display for ClientAccountLockedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The client account is locked")
    }
}

/// Processor for transactions and the generation of the client account index.
pub struct TransactionManager
{
    pub client_account_index: HashMap<u16, ClientAccount>,
}

impl TransactionManager {
    pub fn new() -> TransactionManager {
        TransactionManager {
            client_account_index: HashMap::new()
        }
    }

    /// Process a single transaction.
    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        // Create the client if he doesn't exist.
        if !self.client_account_index.contains_key(&transaction.client) {
            self.client_account_index.insert(transaction.client, ClientAccount::new(transaction.client.clone(), 0.0, 0.0));
        }

        // Borrow the client from the index.
        let mut client_account = self.client_account_index
            .get_mut(&transaction.client).unwrap(); // Should never panic

        if client_account.locked {
            return Err(Box::new(ClientAccountLockedError()));
        }

        // Treat all the transaction types.
        match transaction.transaction_type {
            TransactionType::Deposit { amount } => {
                client_account.available += amount;
                client_account.transaction_index.insert(transaction.tx, transaction);
            }
            TransactionType::Withdrawal { amount } => {
                if client_account.available >= amount {
                    client_account.available -= amount;
                    client_account.transaction_index.insert(transaction.tx, transaction);
                }
            }
            TransactionType::Dispute => {
                // What if we dispute an invalid element or a deposit on a locked account?
                if let Some(disputed_transaction) = client_account.transaction_index.get_mut(&transaction.tx) {
                    if let TransactionState::Executed = disputed_transaction.state {
                        match disputed_transaction.transaction_type {
                            TransactionType::Deposit { amount } => {
                                client_account.held += amount;
                                client_account.available -= amount;
                                disputed_transaction.state = TransactionState::Disputed;
                            }
                            _ => {}
                        }
                    }
                }
            }
            TransactionType::Resolve => {
                if let Some(disputed_transaction) = client_account.transaction_index.get_mut(&transaction.tx) {
                    if let TransactionState::Disputed = disputed_transaction.state {
                        match disputed_transaction.transaction_type {
                            TransactionType::Deposit { amount } => {
                                client_account.held -= amount;
                                client_account.available += amount;
                                disputed_transaction.state = TransactionState::Resolved;
                            }
                            _ => {}
                        }
                    }
                }
            }
            TransactionType::Chargeback => {
                if let Some(disputed_transaction) = client_account.transaction_index.get_mut(&transaction.tx) {
                    if let TransactionState::Disputed = disputed_transaction.state {
                        match disputed_transaction.transaction_type {
                            TransactionType::Deposit { amount } => {
                                client_account.held -= amount;
                                client_account.locked = true;
                                disputed_transaction.state = TransactionState::Chargedback;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Transaction, TransactionManager};
    use crate::transaction_manager::TransactionType;

    #[test]
    fn deposit() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, false);
        assert_eq!(client_account.available, 10.);
        assert_eq!(client_account.held, 0.);
        assert_eq!(client_account.transaction_index.len(), 1);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    fn withdraw() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Withdrawal { amount: 10.0 }, 1, 2)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, false);
        assert_eq!(client_account.available, 0.);
        assert_eq!(client_account.held, 0.);
        assert_eq!(client_account.transaction_index.len(), 2);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    fn withdraw_too_much_is_ignored() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Withdrawal { amount: 20.0 }, 1, 2)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, false);
        assert_eq!(client_account.available, 10.);
        assert_eq!(client_account.held, 0.);
        assert_eq!(client_account.transaction_index.len(), 1);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    fn dispute() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Dispute, 1, 1)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, false);
        assert_eq!(client_account.available, 0.);
        assert_eq!(client_account.held, 10.);
        assert_eq!(client_account.transaction_index.len(), 1);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    fn dispute_twice_is_ignored() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Dispute, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Dispute, 1, 1)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, false);
        assert_eq!(client_account.available, 0.);
        assert_eq!(client_account.held, 10.);
        assert_eq!(client_account.transaction_index.len(), 1);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    fn resolve_disputed_tx() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Dispute, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Resolve, 1, 1)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, false);
        assert_eq!(client_account.available, 10.);
        assert_eq!(client_account.held, 0.);
        assert_eq!(client_account.transaction_index.len(), 1);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    fn resolve_undisputed_tx_is_ignored() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Resolve, 1, 1)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, false);
        assert_eq!(client_account.available, 10.);
        assert_eq!(client_account.held, 0.);
        assert_eq!(client_account.transaction_index.len(), 1);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    fn chargeback() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Dispute, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Chargeback, 1, 1)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, true);
        assert_eq!(client_account.available, 0.);
        assert_eq!(client_account.held, 0.);
        assert_eq!(client_account.transaction_index.len(), 1);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    fn chargeback_undisputed_tx_is_ignored() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Chargeback, 1, 1)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, false);
        assert_eq!(client_account.available, 10.);
        assert_eq!(client_account.held, 0.);
        assert_eq!(client_account.transaction_index.len(), 1);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    fn chargeback_withdrawn_amount() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Withdrawal { amount: 10.0 }, 1, 2)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Dispute, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Chargeback, 1, 1)
        ).unwrap();
        assert_eq!(transaction_manager.client_account_index.len(), 1);
        let client_account = transaction_manager.client_account_index.get(&1).unwrap();
        assert_eq!(client_account.locked, true);
        assert_eq!(client_account.available, -10.);
        assert_eq!(client_account.held, 0.);
        assert_eq!(client_account.transaction_index.len(), 2);
        assert_eq!(client_account.client, 1);
    }

    #[test]
    #[should_panic]
    fn deposit_locked_account_panics() {
        let mut transaction_manager = TransactionManager::new();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Dispute, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Chargeback, 1, 1)
        ).unwrap();
        transaction_manager.process_transaction(
            Transaction::new(TransactionType::Deposit { amount: 10.0 }, 1, 2)
        ).unwrap();
    }
}