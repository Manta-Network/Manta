use crate::alloc::string::{String, ToString};
use crate::crypto::{check_aggregated_g2_pub_key, check_sealed_signature};
use crate::crypto::{G1, G2};
use crate::traits::FromRlp;
use crate::types::{
    common::Address,
    errors::Kind,
    header::Header,
    istanbul::get_epoch_number,
    istanbul::IstanbulExtra,
    proof::{verify_trie_proof, ReceiptProof},
};
use codec::{Decode, Encode};
use frame_support::ensure;
use frame_support::log::info;
use num::cast::ToPrimitive;
use num_bigint::BigInt as Integer;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::vec::Vec;

const ECDSA_SIG_LENGTH: usize = 65;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Encode, Decode, TypeInfo)]
pub struct MapLightClient {
    pub epoch_records: BTreeMap<u64, EpochRecord>,
    pub epoch_size: u64,
    pub header_height: u64,
    pub max_records: u64,
}

impl Default for MapLightClient {
    fn default() -> Self {
        MapLightClient {
            epoch_records: Default::default(),
            epoch_size: 0,
            header_height: 0,
            max_records: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Encode, Decode, TypeInfo)]
pub struct EpochRecord {
    pub threshold: u64,
    pub epoch: u64,
    pub validators: Vec<Validator>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Eq, Encode, Decode, TypeInfo)]
pub struct Validator {
    g1_pub_key: G1,
    weight: u64,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    address: Address,
}

impl MapLightClient {
    pub fn verify_signatures(
        &self,
        header: &Header,
        agg_pk: G2,
        extra: &IstanbulExtra,
        epoch_record: &EpochRecord,
    ) -> Result<(), String> {
        let addresses = epoch_record.validators.iter().map(|x| x.address).collect();
        // check ecdsa signature
        self.verify_ecdsa_signature(header, &extra.seal, &addresses)?;
        // check agg seal
        self.verify_aggregated_seal(header, extra, epoch_record, &agg_pk)?;
        Ok(())
    }

    pub fn verify_ecdsa_signature(
        &self,
        header: &Header,
        signature: &Vec<u8>,
        addresses: &Vec<Address>,
    ) -> Result<(), String> {
        ensure!(
            ECDSA_SIG_LENGTH == signature.len(),
            "invalid ecdsa signature length".to_string()
        );

        let res = addresses
            .iter()
            .filter(|x| x.as_slice() == header.coinbase.as_slice())
            .count();
        ensure!(
            1 == res,
            "the header's coinbase is not in validators".to_string()
        );

        let v = signature.last().unwrap();
        let header_hash = header
            .hash_without_seal()
            .map_err(|_| "header hash error".to_string())?;
        let hash = sp_io::hashing::keccak_256(header_hash.as_slice());
        let mut sig = [0u8; 65];
        sig.copy_from_slice(signature.as_slice());
        let res = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &hash)
            .map_err(|_| "ecdsa signature error".to_string())?;

        let pub_key_hash = sp_io::hashing::keccak_256(res.as_slice());
        ensure!(
            &header.coinbase == &pub_key_hash[12..],
            "ecdsa signer is not correct"
        );
        Ok(())
    }

    fn verify_aggregated_seal(
        &self,
        header: &Header,
        extra: &IstanbulExtra,
        epoch_record: &EpochRecord,
        agg_g2_pk: &G2,
    ) -> Result<(), String> {
        let pair_keys = epoch_record
            .validators
            .iter()
            .map(|x| x.g1_pub_key.clone())
            .collect();
        ensure!(
            self.is_quorum(
                &extra.aggregated_seal.bitmap,
                &epoch_record.validators,
                epoch_record.threshold.into()
            ),
            "threshold is not satisfied".to_string()
        );

        ensure!(
            check_aggregated_g2_pub_key(&pair_keys, &extra.aggregated_seal.bitmap, agg_g2_pk),
            "check g2 pub key failed".to_string()
        );

        let header_hash = header.hash().map_err(|_| "hash error".to_string())?;
        ensure!(
            check_sealed_signature(&extra.aggregated_seal, &header_hash, agg_g2_pk),
            "check sealed signature failed".to_string()
        );
        Ok(())
    }

    fn is_quorum(&self, bitmap: &Integer, validators: &Vec<Validator>, threshold: u64) -> bool {
        let weight: u64 = validators
            .iter()
            .enumerate()
            .filter(|(i, _)| bitmap.bit(*i as u64))
            .map(|(_, v)| v.weight)
            .sum();

        weight >= threshold
    }

    pub fn update_next_validators(
        &mut self,
        cur_epoch_record: &EpochRecord,
        extra: &mut IstanbulExtra,
    ) {
        let mut validator_list: Vec<Validator> = cur_epoch_record
            .validators
            .iter()
            .enumerate()
            .filter(|(i, _)| !extra.removed_validators.bit(*i as _))
            .map(|(_, v)| *v)
            .collect();

        let mut added_validators: Vec<Validator> = extra
            .added_g1_public_keys
            .iter()
            .zip(extra.added_validators.iter())
            .map(|(g1_key, address)| Validator {
                g1_pub_key: G1::from_slice(g1_key).unwrap(),
                weight: 1u64,
                address: *address,
            })
            .collect();
        validator_list.append(&mut added_validators);
        let total_weight: u64 = validator_list.iter().map(|x| x.weight).sum();
        let next_epoch = cur_epoch_record.epoch + 1;
        let next_epoch_record = EpochRecord {
            epoch: next_epoch,
            validators: validator_list,
            threshold: total_weight - total_weight / 3,
        };

        info!(
            "epoch {} validators: remove: {}, add: {:?}",
            next_epoch,
            extra.removed_validators,
            serde_json::to_string(&extra.added_validators).unwrap()
        );

        self.epoch_records.insert(next_epoch, next_epoch_record);
        if next_epoch >= self.max_records {
            let epoch_to_remove = next_epoch - self.max_records;
            self.epoch_records.remove(&epoch_to_remove);
        }
    }

    pub fn verify_proof_data(&self, receipt_proof: ReceiptProof) -> Result<(), Kind> {
        //let header = &receipt_proof.header;
        let header = &Header::from_rlp(&receipt_proof.header).map_err(|_| Kind::RlpDecodeError)?;
        let extra = IstanbulExtra::from_rlp(&header.extra)?;
        let block_number = header.number.to_u64().ok_or(Kind::HeaderError)?;
        // check ecdsa and bls signature
        let epoch = get_epoch_number(block_number, self.epoch_size as u64);
        let epoch_record = self.epoch_records.get(&epoch);
        if epoch_record.is_none() {
            let range = self.get_verifiable_header_range();

            let m = alloc::fmt::format(format_args!(
                "cannot get epoch record for block {}, expected range[{}, {}]",
                header.number.to_string(),
                range.0,
                range.1
            ));
            return Err(Kind::Other { msg: m });
        }
        let epoch_record = epoch_record.unwrap();
        self.verify_signatures(header, receipt_proof.agg_pk, &extra, epoch_record)
            .map_err(|_| Kind::BlsInvalidSignature)?;
        // Verify receipt included into header
        let data = verify_trie_proof(
            header.receipt_hash,
            receipt_proof.key_index,
            receipt_proof.proof,
        );
        let receipt_data = receipt_proof.receipt.encode_index();
        ensure!(
            hex::encode(receipt_data) == hex::encode(data),
            Kind::HeaderVerificationError {
                msg: "receipt data is not equal to the value in trie"
            }
        );
        Ok(())
    }

    pub fn get_verifiable_header_range(&self) -> (u64, u64) {
        let count = self.epoch_records.len() as u64 * self.epoch_size;
        let begin = self.header_height + self.epoch_size + 1u64 - (count as u64);
        let end = self.header_height + self.epoch_size;
        (begin, end)
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{G1, G2};
    use crate::mapclient::{EpochRecord, MapLightClient, Validator};
    use crate::types::header::Header;
    use crate::types::istanbul::{get_epoch_number, IstanbulExtra};
    use frame_support::ensure;
    use frame_support::log::info;
    use num_traits::ToPrimitive;
    use sp_std::map;

    #[test]
    pub fn test_verify() {
        let header_str = r#"{
		  "parentHash": "0xbf8dd177de62a1057d0dd84ed407ff3fd5f2803c0862883f6bee29e47281dd58",
		  "coinbase": "0x7607c9cdd733d8cda0a644839ec2bac5fa180ed4",
		  "root": "0xe902e134b1725e1a479421216fe7d0f1112a5538ce917ea81f812e845c08d042",
		  "txHash": "0x65e349e887f6a9270774bb269e39076286ca9a0351df7dbe9a480a98fe894720",
		  "receiptHash": "0x766dd80fb7c500c2285d4b9ca023800e5226179c6188c89f588698eb475522d4",
		  "bloom": "0x00000000000020020000000000000000000000000400000000000002001000000000000000000000000000000400000000000004000000000000000000000100000020000200000000000008000000200000000000000000200000000020000000000000020000800000100000000800000000000200000000000010000100000400002000000000000000080002002000000000000100000000000000000000000000000080000000000000000004000008000000000000020000000000040010000002000000000000000000000000820000000800000000000000000020800000400000000000000000000000000000400000000000100000000000000000",
		  "number": "0x32f407",
		  "gasLimit": "0x7a1200",
		  "gasUsed": "0x200ed3",
		  "time": "0x63bb759b",
		  "extra": "0xd883010003846765746888676f312e31352e36856c696e757800000000000000f8dbc0c0c080b841fc4dbc3092017a9c7ee5cd335ec062c432a97a14e368ce1b26c1149678c2f53974d69c279533dc5f26b60f1f5cb614a87f136b744383f78893775d98a8ebc36001f8488414f4ff8fb8402d5a9f60284463fbbaa3067cc5265b47cbb7c1a0875cfc88df494bfe39d4faf4195daa0a4a181703a540da6c6169f395697d14470f4c15b4760bbce71e74f10e80f8488414fdff4fb8400aed34b92525864130b7f0331c4e943a24d8af03393d9469a92580bc86cf46c22ce12ce2d870a2ebb1fe6c3e2078d57bde26b60f351d520899960321cd47fbb980",
		  "mixDigest": "0x0000000000000000000000000000000000000000000000000000000000000000",
		  "nonce": "0x0000000000000000",
		  "baseFee": "0x174876e800",
		  "hash": "0xbfabf2bb4b4c5a45b2a91153bbcd01f8113ee3755a9807fe14859230275f2288"
    	}"#;
        let header: Header = serde_json::from_str(header_str).unwrap();
        let agg_pk_str = r#"{
      "xi": "0x2824a53bdc947d74d72242c8d666d434a14be75d4af8764bc90b77fa53cb86bc",
      "xr": "0x17a358540cb2657063766b8c3c7046e7f8d0611b09852293cb38db7f7f537b62",
      "yi": "0x0cdc5be04b14a9de50950e4f5af1f337a29d59380878583675ecf5c381cf3ce5",
      "yr": "0x05bbde808b8b14b21e0c072ecd382cbffb85d5f1ba23d66dfd5ca5e2a822c4ba"
    }"#;
        let agg_pk: G2 = serde_json::from_str(agg_pk_str).unwrap();
        let extra = IstanbulExtra::from_rlp(&header.extra).unwrap();
        let epoch = get_epoch_number(header.number.to_u64().unwrap(), 50000 as u64);
        let mapclient = MapLightClient {
            epoch_records: Default::default(),
            epoch_size: 50000,
            header_height: 3000,
            max_records: 4000,
        };
        let mut addr = [0u8; 20];
        addr.copy_from_slice(
            hex::decode("7607c9cdd733d8cda0a644839ec2bac5fa180ed4")
                .unwrap()
                .as_slice(),
        );
        let validater = Validator {
            g1_pub_key: G1 {
                x: [0u8; 32],
                y: [1u8; 32],
            },
            weight: 1900,
            address: addr,
        };
        let epoch_record = EpochRecord {
            threshold: 0,
            epoch,
            validators: vec![validater],
        };
        /* let r = mapclient.verify_signatures(&header, agg_pk, &extra, &epoch_record);
        match r {
            Ok(_) => {}
            Err(s) => {
                assert_eq!("cc".to_string(), s);
            }
        };*/
    }
}
