use crate::alloc::string::ToString;
use crate::crypto::G2;
use crate::types::common::{Address, Bloom, Hash};
use codec::{Decode, Encode};
use hex::FromHex;
use rlp::{Encodable, Rlp, RlpStream};
use scale_info::prelude::string::String;
use scale_info::TypeInfo;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use sp_io::hashing::keccak_256;
use sp_std::vec;
use sp_std::vec::Vec;
#[derive(Serialize, Deserialize, Clone, Debug, Encode, Decode, TypeInfo, Eq, PartialEq)]
pub struct ReceiptProof {
    pub header: Vec<u8>,
    pub agg_pk: G2,
    pub receipt: Receipt,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub key_index: Vec<u8>,
    pub proof: Vec<ProofEntry>,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo, Eq, PartialEq)]
pub struct ProofEntry(Vec<u8>);

impl Serialize for ProofEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = hex::encode(self.0.as_slice());
        if hex_string.is_empty() {
            return serializer.serialize_str("");
        }
        serializer.serialize_str(&(String::from("0x") + &hex_string))
    }
}

impl<'de> Deserialize<'de> for ProofEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String as Deserialize>::deserialize(deserializer)?;
        if !s.starts_with("0x") {
            return Err(Error::custom("should start with 0x"));
        }

        let data = Vec::from_hex(&s[2..]).map_err(|err| Error::custom(err.to_string()))?;
        Ok(ProofEntry(data))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Encode, Decode, TypeInfo, Eq, PartialEq)]
pub struct Receipt {
    pub receipt_type: u128,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub post_state_or_status: Vec<u8>,
    pub cumulative_gas_used: u64,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub bloom: Bloom,
    pub logs: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Encode, Decode, TypeInfo, PartialEq)]
pub struct LogEntry {
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub address: Address,
    #[serde(with = "crate::serialization::bytes::hexvec")]
    pub topics: Vec<Hash>,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub data: Vec<u8>,
}

impl Receipt {
    pub fn encode_index(&self) -> Vec<u8> {
        if let 0 = self.receipt_type {
            return rlp::encode(self);
        }

        let mut res: Vec<u8> = Vec::new();
        res.push(self.receipt_type as _);
        res.append(&mut rlp::encode(self));
        res
    }
}

impl Encodable for Receipt {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4);

        // post_state_or_status
        s.append(&self.post_state_or_status);

        // cumulative_gas_used
        s.append(&self.cumulative_gas_used);

        // bloom
        s.append(&self.bloom.as_ref());

        // logs
        s.begin_list(self.logs.len());
        for log in self.logs.iter() {
            s.append(log);
        }
    }
}

impl Encodable for LogEntry {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);

        // address
        s.append(&self.address.as_ref());

        // topics
        s.begin_list(self.topics.len());
        for topic in self.topics.iter() {
            s.append(&topic.as_ref());
        }

        // data
        s.append(&self.data);
    }
}

pub fn verify_trie_proof(expected_root: Hash, key: Vec<u8>, proof: Vec<ProofEntry>) -> Vec<u8> {
    let mut actual_key = vec![];
    for el in key {
        actual_key.push(el / 16);
        actual_key.push(el % 16);
    }
    _verify_trie_proof(expected_root.to_vec(), &actual_key, &proof, 0, 0)
}

fn _verify_trie_proof(
    expected_root: Vec<u8>,
    key: &Vec<u8>,
    proof: &Vec<ProofEntry>,
    key_index: usize,
    proof_index: usize,
) -> Vec<u8> {
    let node = &proof[proof_index].0;

    if key_index == 0 || node.len() >= 32 {
        assert_eq!(
            expected_root.as_slice(),
            near_keccak256(node),
            "incorrect root for node {:?}",
            node
        );
    } else {
        assert_eq!(expected_root, node.as_slice(), "incorrect node root");
    }

    let node = Rlp::new(node.as_slice());

    if node.iter().count() == 17 {
        // Branch node
        if key_index == key.len() {
            assert_eq!(
                proof_index + 1,
                proof.len(),
                "incorrect proof length for branch node"
            );
            get_vec(&node, 16)
        } else {
            let new_expected_root = get_vec(&node, key[key_index] as usize);
            _verify_trie_proof(
                new_expected_root,
                key,
                proof,
                key_index + 1,
                proof_index + 1,
            )
        }
    } else {
        // Leaf or extension node
        assert_eq!(node.iter().count(), 2, "incorrect node count");
        let path_u8 = get_vec(&node, 0);
        // Extract first nibble
        let head = path_u8[0] / 16;
        // assert!(0 <= head); is implicit because of type limits
        assert!(head <= 3, "incrorrect head {}", head);

        // Extract path
        let mut path = vec![];
        if head % 2 == 1 {
            path.push(path_u8[0] % 16);
        }
        for val in path_u8.into_iter().skip(1) {
            path.push(val / 16);
            path.push(val % 16);
        }
        assert_eq!(
            path.as_slice(),
            &key[key_index..key_index + path.len()],
            "incorrect path"
        );

        if head >= 2 {
            // Leaf node
            assert_eq!(
                proof_index + 1,
                proof.len(),
                "incorrect proof length for leaf node"
            );
            assert_eq!(
                key_index + path.len(),
                key.len(),
                "incorrect key length for leaf node"
            );
            get_vec(&node, 1)
        } else {
            // Extension node
            let new_expected_root = get_vec(&node, 1);
            _verify_trie_proof(
                new_expected_root,
                key,
                proof,
                key_index + path.len(),
                proof_index + 1,
            )
        }
    }
}

pub fn near_keccak256(data: &[u8]) -> [u8; 32] {
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(keccak_256(data).as_slice());
    buffer
}

/// Get element at position `pos` from rlp encoded data,
/// and decode it as vector of bytes
fn get_vec(data: &Rlp, pos: usize) -> Vec<u8> {
    data.at(pos).unwrap().as_val::<Vec<u8>>().unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::FromBytes;
    use crate::types::common::{Nonce, ADDRESS_LENGTH, BLOOM_BYTE_LENGTH};

    const RECEIPT_TINY_LEGACY: &str = "f901068080b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0";
    const RECEIPT_SIMPLE_ACCESS_LIST: &str = "f9011e0102b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d8d7940000000000000000000000000000000000000000c080";

    #[test]
    fn encodes_receipt_to_rlp() {
        let mut receipt = Receipt {
            receipt_type: 0u128,
            post_state_or_status: vec![],
            cumulative_gas_used: 0u64,
            bloom: [0; BLOOM_BYTE_LENGTH],
            logs: vec![],
        };

        let bytes: Vec<u8> = rlp::encode(&receipt);

        assert_eq!(hex::decode(RECEIPT_TINY_LEGACY).unwrap(), bytes);

        receipt.receipt_type = 1u128;
        receipt.post_state_or_status = vec![1];
        receipt.cumulative_gas_used = 2u64;
        receipt.logs = vec![LogEntry {
            address: [0; ADDRESS_LENGTH],
            topics: vec![],
            data: vec![],
        }];
        assert_eq!(
            hex::decode(RECEIPT_SIMPLE_ACCESS_LIST).unwrap(),
            rlp::encode(&receipt)
        );
    }

    #[test]
    fn encodes_log_entry_to_rlp() {
        const LOG_ENTRY_EMPTY: &str = "d7940000000000000000000000000000000000000000c080";
        let entry = LogEntry {
            address: [0; ADDRESS_LENGTH],
            topics: vec![],
            data: vec![],
        };

        let bytes: Vec<u8> = rlp::encode(&entry);

        assert_eq!(hex::decode(LOG_ENTRY_EMPTY).unwrap(), bytes);
    }

    #[test]
    fn test_verify_proof() {
        let expected_value = "f902a60183af4adfb9010000000000000000000000000000000000000000000000000000000000000000000800010000000000000002000100000000000000000000000000000000000000000000000000000008000008000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000010000000040000000000000000000000000000000200000000010000000000000000000000000000000000200080000000202000000000000000000000000000004000000000000002000000000000000000000000000000000000080000000000000000000000000000000000000000000200000004000000000000000000000000000000f9019bf89b94a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000c22df065a81f6e0107e214991b9d7fb179d401b3a000000000000000000000000023ddd3e3692d1861ed57ede224608875809e127fa00000000000000000000000000000000000000000000000000000000005f5e100f8fc9423ddd3e3692d1861ed57ede224608875809e127ff863a0dd85dc56b5b4da387bf69c28ec19b1d66e793e0d51b567882fa31dc50bbd32c5a0000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48a0000000000000000000000000c22df065a81f6e0107e214991b9d7fb179d401b3b8800000000000000000000000000000000000000000000000000000000005f5e1000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000a6d616b6b652e6e65617200000000000000000000000000000000000000000000";
        let expected_root = "73733f420161b4189ea48140489bfada55d485fd580ab7e6f0f4b6de229f5177";
        let key = "820137";
        let proof_rlp = vec!["f90131a07dc6deefc13acb3456defc30824e9ba7d344e0fed67d6fe918a845ac6c7ff68ca00743abb9e8a2419e731aabac46e87dd9166ef04f4c0e17b607f699169fd16effa0de6439dd92daf3fe5ae984f9176a1d53f0ab2c03a73ea9ee2c94c79a87b82386a08eccde52f7cdcfa2207e2e8256bbd05a78fbab4f1b564a98a7f5b259b9fcb05da0196a72fd5279acc9146896618d5a134398bfc5d84063bcb2dc4f206bceb1526fa0daa3100c65bc47d986741898d7dfa1cc2f944d9f621b33a53d52047d98ab6e84a0126a9c69a2fb01312dffd739ee2a86c15106497d5e53314875e3a83c915b40c3a0b89b77f6776de33f0d291891d4271546d3b4946325f6fa66d38a1618f699b7b0a06b4f2fac50925da7c11ddac2321257cf157d426bffedcce8c3e999f8dd3902ff8080808080808080", "f871a073170337a44a638efb6d735150b3a06346b54b6176c9424307e6c1f4a4604131a0409f60141274adbaf1fd8808c432c599025a80763a61aca8710ba5416436c885a064d0127fe80ad8301e425eece21dd4811515312fad7e95b9ad4f853d003582a88080808080808080808080808080", "e4820001a022d9ed1b1940164d904d587080c9ca1d5ebb7e711211233bee7ecf6f0fba3d8e", "f8d1a0af41ab83382da16fba21a258c18a231957c14eb91ede9b75b089d37474efe1b8a0102af48a2d48aa200cb90bafdb43c3845ed09e2d34f333944ac7c172f2becec4a0644f776baaf4dd2a45e817c3b70ed881419f31d966debf0e2dac62426b1308eea08498703814dfa09c76b9f8dde1d5e3865b92b805e9ffd77d12cd8221497fe604a02a6a5cc557e67488aef767895f2ad789fa339aab229d7e94b78d6a8187989d3ea002ec005b9dafdfd58601cd6dc96fd958a8681981442c485c87142ae85acc1fd28080808080808080808080", "f90211a02e04ddc4ab28665d70404d04601838d03b219207d68a477e086144d5452b035ea07458b1e7734dcde7a48e763f57b533d39a9893b2fd05ece758eb95c45230b69aa05ce53207da7cb7efdfa60bfc57dc23a5469d2823ac6d94377fda37d7d6e77a23a0ba360fa8bb757bec0086b0d2973bb39de6b874ae3558e0b91eb54579022bc68fa023068af8cff2927c6c437840b4bee730c5ef2d918c0bb086b453da9071f3e3a8a05a6cb3455636113070e724682c1e852b564bc26195690b57adf95b03e453fb56a090e6afaa341c8c8583ba621e0f4369e0a36e488d167061bf0627c8de8e4b1b53a0c974bc6676b17c2e0e0016b86e2261cc69fa10b06ad22938f851674c853face6a0134724224ee173faf5807c3b963e4aff5d8435c2296230ba6fbdf222262ee7a5a0ea6f74b84a4ee7f7d557bb27e61c75ae30ead9092faab3b441c4e5055c8768d7a09e5ab3942cfe8410180611c7eddb2364ba2022e53971b50250636f2576f1528fa0fe27620c114d5ce5b8c96859b754cf0a40a9a0cb7a6c88a492ca906b09db6c5aa023558ac1d7facb4cb81d5ced13c9126b70a898a81b63fe52117792ed5bcadb06a00b52d0a4cd96595521a0783e6f17de8b9bdd15f1e50ab8b51179a98ae6df18e5a0462c433d431953bcead5900c4b372c60b20280d05366fe48de6384152cfc8da9a0a5a9b086a22dc344a40496a9de4ebab616fa87dfc77760eef7939ba51dd193cd80", "f902ad20b902a9f902a60183af4adfb9010000000000000000000000000000000000000000000000000000000000000000000800010000000000000002000100000000000000000000000000000000000000000000000000000008000008000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000010000000040000000000000000000000000000000200000000010000000000000000000000000000000000200080000000202000000000000000000000000000004000000000000002000000000000000000000000000000000000080000000000000000000000000000000000000000000200000004000000000000000000000000000000f9019bf89b94a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000c22df065a81f6e0107e214991b9d7fb179d401b3a000000000000000000000000023ddd3e3692d1861ed57ede224608875809e127fa00000000000000000000000000000000000000000000000000000000005f5e100f8fc9423ddd3e3692d1861ed57ede224608875809e127ff863a0dd85dc56b5b4da387bf69c28ec19b1d66e793e0d51b567882fa31dc50bbd32c5a0000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48a0000000000000000000000000c22df065a81f6e0107e214991b9d7fb179d401b3b8800000000000000000000000000000000000000000000000000000000005f5e1000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000a6d616b6b652e6e65617200000000000000000000000000000000000000000000"];

        let decoded_root = hex::decode(expected_root).unwrap();
        let mut expected_root = Hash::default();
        expected_root.copy_from_slice(&*decoded_root);
        let key = hex::decode(key).unwrap();
        let proof = proof_rlp
            .into_iter()
            .map(|x| ProofEntry {
                0: hex::decode(x).unwrap(),
            })
            .collect();
        let expected_value = hex::decode(expected_value).unwrap();
        assert_eq!(verify_trie_proof(expected_root, key, proof), expected_value);
    }

    #[test]
    fn test_serde_json_receipt() {
        let header = Header {
            parent_hash: Hash::from_bytes(
                &hex::decode("7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7")
                    .unwrap(),
            )
            .unwrap()
            .to_owned(),
            coinbase: Address::from_bytes(
                &hex::decode("908D0FDaEAEFbb209BDcb540C2891e75616154b3").unwrap(),
            )
            .unwrap()
            .to_owned(),
            root: Hash::from_bytes(
                &hex::decode("7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7")
                    .unwrap(),
            )
            .unwrap()
            .to_owned(),
            tx_hash: Hash::from_bytes(
                &hex::decode("7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7")
                    .unwrap(),
            )
            .unwrap()
            .to_owned(),
            receipt_hash: Hash::from_bytes(
                &hex::decode("7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7")
                    .unwrap(),
            )
            .unwrap()
            .to_owned(),
            bloom: [1; BLOOM_BYTE_LENGTH],
            number: Default::default(),
            gas_limit: Default::default(),
            gas_used: Default::default(),
            time: Default::default(),
            extra: vec![1, 2, 3],
            mix_digest: Hash::from_bytes(
                &hex::decode("7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7")
                    .unwrap(),
            )
            .unwrap()
            .to_owned(),
            nonce: Nonce::from_bytes(&hex::decode("7285abd5b24742ff").unwrap())
                .unwrap()
                .to_owned(),
            base_fee: Default::default(),
        };

        let agg_pk = G2 {
            xr: [0; 32],
            xi: [1; 32],
            yr: [2; 32],
            yi: [3; 32],
        };
        let receipt = Receipt {
            receipt_type: 1u128,
            post_state_or_status: vec![1, 2, 3],
            cumulative_gas_used: 10000u64,
            bloom: [0; 256],
            logs: vec![LogEntry {
                address: [1; 20],
                topics: vec![[1; 32], [2; 32]],
                data: vec![3, 2, 1, 9],
            }],
        };

        // let v = header.encode();
        /*        let receipt_proof = ReceiptProof {
            header,
            agg_pk,
            receipt,
            key_index: vec![1, 2],
            proof: vec![ProofEntry { 0: vec![1, 2] }, ProofEntry { 0: vec![1, 2] }],
        };

        let exp = r#"{"header":{"parentHash":"0x7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7","coinbase":"0x908d0fdaeaefbb209bdcb540c2891e75616154b3","root":"0x7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7","txHash":"0x7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7","receiptHash":"0x7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7","bloom":"0x01010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101","number":"0x0","gasLimit":"0x0","gasUsed":"0x0","time":"0x0","extra":"0x010203","mixDigest":"0x7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7","nonce":"0x7285abd5b24742ff","baseFee":"0x0"},"agg_pk":{"xr":"0x0000000000000000000000000000000000000000000000000000000000000000","xi":"0x0101010101010101010101010101010101010101010101010101010101010101","yr":"0x0202020202020202020202020202020202020202020202020202020202020202","yi":"0x0303030303030303030303030303030303030303030303030303030303030303"},"receipt":{"receipt_type":"1","post_state_or_status":"0x010203","cumulative_gas_used":"10000","bloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","logs":[{"address":"0x0101010101010101010101010101010101010101","topics":["0x0101010101010101010101010101010101010101010101010101010101010101","0x0202020202020202020202020202020202020202020202020202020202020202"],"data":"0x03020109"}]},"key_index":"0x0102","proof":["0x0102","0x0102"]}"#;

        let serialized = serde_json::to_string(&receipt_proof).unwrap();
        assert_eq!(exp, serialized);

        let deserialized: ReceiptProof = serde_json::from_str(&serialized).unwrap();
        println!("{:?}", deserialized);

        let serialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(exp, serialized);*/
    }
}
