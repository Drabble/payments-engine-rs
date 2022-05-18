# Payment Engine Rs

Payment Engine Rs is a simple transaction processor for csv files.

Each transaction has a type, a unique client id, a transaction id and an optional amount.
The different transaction types are listed below.

* **deposit** - Deposit a certain amount into the client account.
* **withdrawal** - Withdraw a certain amount from the client account.
* **dispute** - Dispute the transaction with the given transaction id. Disputed funds are held
until they are released.
* **resolve** - Resolves a disputed transaction with a given transaction id.
* **chargeback** - Charges back the amount of a given transaction id from the client's balance.

## Usage

In order to run the binary with from a file on your filesystem, use the following command:

```bash
cargo run -- sample.csv
```

## Example

The code below shows how to run the payments engine with the arguments provided to the command.

```rust
use std::env;
use std::fs::File;
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
```

## Improving performances

Here are some ideas to improve the performances of the engine.

## Using an async csv parser

We could use an async csv parser as described [here](https://dfrasca.hashnode.dev/rust-building-an-async-csv-parser).
We could also make our own async csv parser instead of [csv-async](https://github.com/gwierzchowski/csv-async) to have
more control and safety.

Using an async csv parser could help us read parts of the csv on a separate thread while the main thread handles the
processing of the csv records.

## Using map-reduce to parallelize the processing of chunks of the file

We could parallelize the entire processing with a map reduce instead of iterating through lines. In the map we compute 
the output for each batch of ordered transactions and return client accounts. In the reduce we compute sum the client 
accounts returned by the map operations.

1. A problem is that we don't know if the user has gone under the 0 value or if it has been locked.
To solve that we allow negative values and return the minimum available value that has been reached during the map.

2. Another problem is that we might have withdrawals that would bring the available funds under 0. These withdrawals
should be ignored, but we don't know the total available funds to the user. We will have to find another way to
resolve them in the reducer if the available value goes under 0. 

3. Finally disputes resolves and paybacks have to be bundled together someway, otherwise we
would need to have an index that can read any transactions by id.

## Tests

Many use cases are tested directly within the unit tests in the project. You can run them using the following command:

```
cargo test
```
