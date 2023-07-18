pub(crate) mod hexstring {
    use crate::alloc::string::String;
    use hex::FromHex;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use sp_std::vec::Vec;
    /// Deserialize string into T
    pub(crate) fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: hex::FromHex,
        <T as FromHex>::Error: sp_std::fmt::Display,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        if s.len() <= 2 || !s.starts_with("0x") {
            return T::from_hex(Vec::new()).map_err(D::Error::custom);
        }

        T::from_hex(&s[2..]).map_err(D::Error::custom)
    }

    /// Serialize from T into string
    pub(crate) fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let hex_string = hex::encode(value.as_ref());
        if hex_string.is_empty() {
            return serializer.serialize_str("");
        }

        serializer.serialize_str(&(String::from("0x") + &hex_string))
    }
}

pub(crate) mod hexbigint {
    use num::Num;
    use num_bigint::BigInt as Integer;
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

    /// Deserialize string into Integer
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Integer, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        if s.len() <= 2 || !s.starts_with("0x") {
            return Err(D::Error::custom(alloc::fmt::format(format_args!(
                "hex string should start with '0x', got: {}",
                s
            ))));
        }
        Integer::from_str_radix(&s[2..], 16).map_err(D::Error::custom)
    }

    /// Serialize from T into string
    pub(crate) fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: sp_std::fmt::LowerHex,
    {
        alloc::fmt::format(format_args!("0x{:x}", value)).serialize(serializer)
    }
}

pub(crate) mod hexvec {
    use crate::alloc::borrow::ToOwned;
    use crate::alloc::string::String;
    use crate::traits::FromBytes;
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
    use sp_std::vec::Vec;
    /// Deserialize vector into Vec<T>
    pub(crate) fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromBytes + Clone,
    {
        let items: Vec<String> = Deserialize::deserialize(deserializer)?;
        let mut out: Vec<T> = Vec::new();

        for item in items {
            if item.len() <= 2 || !item.starts_with("0x") {
                return Err(D::Error::custom(alloc::fmt::format(format_args!(
                    "hex string should start with '0x', got: {}",
                    item
                ))));
            }

            match hex::decode(&item[2..]) {
                Ok(decoded) => match T::from_bytes(&decoded) {
                    Ok(concrete) => out.push(concrete.to_owned()),
                    Err(e) => {
                        return Err(D::Error::custom(alloc::fmt::format(format_args!(
                            "failed to call from_bytes, got: {:?}",
                            e
                        ))))
                    }
                },
                Err(e) => {
                    return Err(D::Error::custom(alloc::fmt::format(format_args!(
                        "failed to decode hex data, got: {}",
                        e
                    ))))
                }
            }
        }
        Ok(out)
    }

    /// Serialize from &[T] into vector of strings
    pub(crate) fn serialize<S, T>(value: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        value
            .iter()
            .map(|v| alloc::fmt::format(format_args!("0x{}", hex::encode(v))))
            .collect::<Vec<String>>()
            .serialize(serializer)
    }
}
