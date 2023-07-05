default: check

check:
   SKIP_WASM_BUILD=1 cargo check --features runtime-benchmarks,try-runtime

c: check

clippy:
    cargo +nightly clippy --features=runtime-benchmarks

dev chain="calamari":
    cargo run --release -- --chain={{ chain }}-localdev --tmp

rel chain="calamari":
    cargo build --release

fmt: 
    cargo +nightly fmt --all

t:
    RUST_LOG=verbose cargo test --package pallet-lottery
    # RUST_LOG=verbose cargo test --package pallet-lottery --  --exact --nocapture 
    # RUST_LOG=verbose cargo test --release --features=runtime-benchmarks,try-runtime --workspace --exclude integration-tests --package pallet-lottery
    # RUST_LOG=verbose cargo test --package pallet-lottery --lib -- tests::depsit_withdraw_deposit_works --exact --nocapture 
    # RUST_LOG=verbose cargo test --release --features=runtime-benchmarks,try-runtime --workspace --exclude integration-tests -p pallet-lottery
    # RUST_LOG=verbose cargo test --release --features runtime-benchmarks -p pallet-lottery
    # RUST_LOG=verbose cargo test --release --features runtime-benchmarks -p pallet-randomness

bench:
    cargo run --release --features runtime-benchmarks -- benchmark pallet --chain calamari-local --pallet "*" --extrinsic "*"

ba:
    cargo run --release --features runtime-benchmarks -- benchmark pallet --chain calamari-local --pallet "pallet-author-inherent" --extrinsic "*"

bl:
    RUST_LOG=verbose cargo run --release --features runtime-benchmarks -- benchmark pallet --chain manta-local --pallet "pallet-lottery" --extrinsic "*" --steps 10
    RUST_LOG=verbose cargo run --release --features runtime-benchmarks -- benchmark pallet --chain calamari-local --pallet "pallet-lottery" --extrinsic "*" --steps 10
    # RUST_LOG=error cargo run --release --features runtime-benchmarks -- benchmark pallet --chain calamari-local --pallet "pallet-lottery" --extrinsic "request_withdraw" --steps 20

bs:
    RUST_LOG=verbose cargo run --release --features runtime-benchmarks -- benchmark pallet --chain manta-local --pallet "pallet-parachain-staking" --extrinsic "*" --steps 10

try:
    cargo run --release --features try-runtime -- try-runtime --chain calamari-dev --wasm-execution=compiled

fast:
    cargo build --release --features=fast-runtime

manta-staging:
    rm staging-chainspec.json staging-genesis.state staging-genesis.wasm 
    cargo run --release -- build-spec --chain manta-testnet --raw > staging-chainspec.json
    cargo run --release -- export-genesis-state --chain=staging-chainspec.json > staging-genesis.state
    cargo run --release -- export-genesis-wasm --chain=staging-chainspec.json > staging-genesis.wasm

changelog:
    cd /Users/adam/git/dev-tools/changelog-generator && cargo run -- -u garandor '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' -r "/Users/adam/git/manta" -c config.toml

weekly:
    cd /Users/Adam/git/foam-knowledgebase/journal && python3 /Users/Adam/git/foam-knowledgebase/_scripts/weekly.py

rpc url="https://crispy.baikal.testnet.calamari.systems":
    curl {{ url }} -H "Content-Type:application/json;charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"lottery_next_drawing_at","params": []}' && echo "\n"
    curl {{ url }} -H "Content-Type:application/json;charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"lottery_current_prize_pool","params": []}'&& echo "\n"
    curl {{ url }} -H "Content-Type:application/json;charset=utf-8" -d '{"jsonrpc":"2.0","id":1,"method":"lottery_not_in_drawing_freezeout","params": []}'&& echo "\n"