# manta-xt
A client APIs written by rust-lang for Manta/Dolpjin/Calamari.

## How to update
1. Install `subxt-cli`.
```
cargo install subxt-cli
```

2. Get latest metadata.
```
subxt metadata -f bytes --url http://localhost:9967/ > metadata/metadata.scale
```
Point the url to the node you start.
If everything works fine, you will get a encoded metadata file in the root folder.

3. And put the metadata to the folder `metadata`.

## Examples
