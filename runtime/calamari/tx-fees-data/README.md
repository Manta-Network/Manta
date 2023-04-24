# Transaction Fees History for Calamari Runtime

- The code about estmating all tx fees locates at [`diff_gas_fees.rs`](../src/diff_gas_fees.rs).
- All tx fees history locates at [`tx-fees-data`](.).
- If any extrinsic fluctuates over **10**, the test case `diff_gas_fees::diff_gas_fees` will fail.

## Generate latest tx fees

Run the command.
```sh

cargo t write_all_current_extrinsic_gas_fee_to_csv
```
> When you run this test case, please disable this line of code: `#[ignore]`.

It will generate a csv file located at `tx-fees-data/{crate-version}-tx-fees.csv`.

## When add new extrinsics to diff_gas_fees
If there's new pallet or new extrinsic introduced, please add them to [`diff_gas_fees.rs`](../src/diff_gas_fees.rs).
