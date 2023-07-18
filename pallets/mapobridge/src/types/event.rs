use crate::alloc::string::ToString;
use crate::types::proof::LogEntry;
use crate::types::{
    common::Address,
    prover::{EthEventParams, MapEvent},
};
use ethabi::ParamType;
use serde::{Deserialize, Serialize};
use sp_std::vec;
use sp_std::vec::Vec;

pub type CryptoHash = [u8; 32];

/// Data that was emitted by the Ethereum Locked event.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct MapTransferOutEvent {
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub map_bridge_address: Address,
    pub from_chain: u128,
    pub to_chain: u128,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token: Vec<u8>,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub amount: u128,
    pub to_chain_token: Vec<u8>,
}

impl MapTransferOutEvent {
    fn event_params() -> EthEventParams {
        vec![
            ("fromChain".to_string(), ParamType::Uint(256), true),
            ("toChain".to_string(), ParamType::Uint(256), true),
            ("orderId".to_string(), ParamType::FixedBytes(32), false),
            ("token".to_string(), ParamType::Bytes, false),
            ("from".to_string(), ParamType::Bytes, false),
            ("to".to_string(), ParamType::Bytes, false),
            ("amount".to_string(), ParamType::Uint(256), false),
            ("toChainToken".to_string(), ParamType::Bytes, false),
        ]
    }

    /// Parse raw log entry data.
    pub fn from_log_entry_data(data: &LogEntry) -> Option<Self> {
        let event = MapEvent::from_log_entry_data(
            "mapTransferOut",
            MapTransferOutEvent::event_params(),
            data,
        )?;
        let from_chain = event.log.params[0]
            .value
            .clone()
            .into_uint()?
            .as_u128()
            .into();
        let to_chain = event.log.params[1]
            .value
            .clone()
            .into_uint()?
            .as_u128()
            .into();
        let order_id: CryptoHash = event.log.params[2]
            .value
            .clone()
            .into_fixed_bytes()?
            .try_into()
            .ok()?;
        let token = event.log.params[3].value.clone().into_bytes()?;
        let from = event.log.params[4].value.clone().into_bytes()?;
        let to = event.log.params[5].value.clone().into_bytes()?;
        let amount = event.log.params[6]
            .value
            .clone()
            .into_uint()?
            .as_u128()
            .into();
        let to_chain_token = event.log.params[7].value.clone().into_bytes()?;
        Some(Self {
            map_bridge_address: event.mcs_address,
            from_chain,
            to_chain,
            order_id,
            token,
            from,
            to,
            amount,
            to_chain_token,
        })
    }
}
/*
#[derive(Serialize, Deserialize, Debug)]
pub struct TransferOutEvent {
    pub from_chain: U128,
    pub to_chain: U128,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token: String,
    pub from: String,
    pub to: Vec<u8>,
    pub amount: U128,
    pub to_chain_token: String,
}

impl Encodable for TransferOutEvent {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(8);

        s.append(&self.from_chain.0);
        s.append(&self.to_chain.0);
        s.append(&self.order_id.as_ref());
        s.append(&self.token);
        s.append(&self.from);
        s.append(&self.to);
        s.append(&self.amount.0);
        s.append(&self.to_chain_token);
    }
}
*/

/*#[derive(Serialize, Deserialize, Debug)]
pub struct DepositOutEvent {
    pub from_chain: U128,
    pub to_chain: U128,
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub order_id: CryptoHash,
    pub token: String,
    pub from: String,
    pub to: Vec<u8>,
    pub amount: U128,
}*/

/*impl Encodable for DepositOutEvent {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(7);

        s.append(&self.from_chain.0);
        s.append(&self.to_chain.0);
        s.append(&self.order_id.as_ref());
        s.append(&self.token);
        s.append(&self.from);
        s.append(&self.to);
        s.append(&self.amount.0);
    }
}*/

/*impl std::fmt::Display for DepositOutEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(rlp::encode(self)))
    }
}*/

#[cfg(test)]
mod tests {
    use super::*;
    use ethabi::Token;
    use hex;
    use std::str::FromStr;
    use std::string::String;

    impl MapTransferOutEvent {
        pub fn to_log_entry_data(&self) -> LogEntry {
            MapEvent::to_log_entry_data(
                "mapTransferOut",
                MapTransferOutEvent::event_params(),
                self.map_bridge_address,
                vec![
                    self.from_chain.to_be_bytes().to_vec(),
                    self.to_chain.to_be_bytes().to_vec(),
                ],
                vec![
                    Token::FixedBytes(self.order_id.clone().to_vec()),
                    Token::Bytes(self.token.clone()),
                    Token::Bytes(self.from.clone()),
                    Token::Bytes(self.to.clone()),
                    Token::Uint(self.amount.into()),
                    Token::Bytes(self.to_chain_token.clone()),
                ],
            )
        }
    }

    #[test]
    fn test_event_data() {
        let logs_str = r#"[
            {
                "address": "0xe2123fa0c94db1e5baeff348c0e7aecd15a11b45",
                "topics": [
                    "0x44ff77018688dad4b245e8ab97358ed57ed92269952ece7ffd321366ce078622",
                    "0x00000000000000000000000000000000000000000000000000000000000000d4",
                    "0x000000000000000000000000000000000000000000000000000000004e454153"
                ],
                "data": "0x64e604787cbf194841e7b68d7cd28786f6c9a0a3ab9f8b0a0e87cb4387ab010700000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000014ec3e016916ba9f10762e33e03e8556409d096fb40000000000000000000000000000000000000000000000000000000000000000000000000000000000000014223e016916ba9f10762e33e03e8556409d096f22000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f70616e646172722e746573746e6574000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000196d63735f746f6b656e5f302e6d63732e746573742e6e65617200000000000000"
            }
        ]"#;

        let logs: Vec<LogEntry> = serde_json::from_str(&logs_str).unwrap();
        assert_eq!(1, logs.len(), "should have only 1 log");

        let event = MapTransferOutEvent::from_log_entry_data(logs.get(0).unwrap()).unwrap();
        assert_eq!(
            "ec3e016916ba9f10762e33e03e8556409d096fb4",
            hex::encode(event.token.clone())
        );
        assert_eq!(
            "223e016916ba9f10762e33e03e8556409d096f22",
            hex::encode(event.from.clone())
        );
        assert_eq!(212, event.from_chain);
        assert_eq!(1313161555, event.to_chain);
        assert_eq!(
            "pandarr.testnet",
            String::from_utf8(event.to.clone()).unwrap()
        );
        assert_eq!(100, event.amount);
        assert_eq!(
            "mcs_token_0.mcs.test.near",
            String::from_utf8(event.to_chain_token.clone()).unwrap()
        );
        assert_eq!(
            "e2123fa0c94db1e5baeff348c0e7aecd15a11b45".to_lowercase(),
            hex::encode(event.map_bridge_address)
        );

        let data = event.to_log_entry_data();
        let result = MapTransferOutEvent::from_log_entry_data(&data).unwrap();
        assert_eq!(result, event);
    }
    /*
        #[test]
        fn test_event_json() {
            let event = TransferOutEvent {
                from_chain: 1000u128,
                to_chain: 2000u128,
                order_id: [
                    1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3,
                    4, 5, 1, 2,
                ],
                token: "from.token.near".to_string(),
                from: "alice.near".to_string(),
                to: vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5],
                amount: 100u128,
                to_chain_token: "".to_string(),
            };

            println!("{}", serde_json::to_string(&event).unwrap())
        }
    */
    #[test]
    fn test_gen_event() {
        let event = MapTransferOutEvent {
            map_bridge_address: Address::try_from(
                hex::decode("765a5a86411ab8627516cbb77d5db00b74fe610d").unwrap(),
            )
            .unwrap(),
            from_chain: 212u128,
            to_chain: 1313161555u128,
            order_id: sp_io::hashing::keccak_256("123".as_bytes()),
            token: hex::decode("ec3e016916ba9f10762e33e03e8556409d096fb4").unwrap(),
            from: hex::decode("223e016916ba9f10762e33e03e8556409d096f22").unwrap(),
            to: "pandarr.test.near".as_bytes().to_vec(),
            amount: 100u128,
            to_chain_token: "wrap.test.near".as_bytes().to_vec(),
        };

        let data = event.to_log_entry_data();
        let result = MapTransferOutEvent::from_log_entry_data(&data).unwrap();
        assert_eq!(result, event);

        println!("{:?}", serde_json::to_string(&data).unwrap());
    }
}
