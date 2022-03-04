#!/usr/bin/env bash

########################## Check Manta-Node ##########################

git clone https://github.com/Manta-Network/Manta.git

cd Manta/

sed -i "s@pallet-manta-pay = { git='https://github.com/Manta-Network/pallet-manta-pay', branch='calamari', default-features = false }@pallet-manta-pay = {path= '../../../../', default-features = false }@g" ./runtimes/manta/runtime/Cargo.toml
         
cargo build
cargo build --all-features
