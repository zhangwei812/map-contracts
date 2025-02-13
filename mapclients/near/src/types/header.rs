use crate::istanbul::istanbul_filtered_header;
use crate::serialization::rlp::{
    big_int_to_rlp_compat_bytes, rlp_list_field_from_bytes, rlp_to_big_int,
};
use crate::slice_as_array_ref;
use crate::traits::{DefaultFrom, FromBytes, FromRlp, ToRlp};
use crate::types::istanbul::ISTANBUL_EXTRA_VANITY_LENGTH;
use num_bigint::BigInt as Integer;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use near_sdk::env::keccak256;
use near_sdk::serde::{Serialize, ser::{Serializer}, Deserialize, de::{Deserializer, self}};
use crate::types::errors::Kind;

/// HASH_LENGTH represents the number of bytes used in a header hash
pub const HASH_LENGTH: usize = 32;

/// ADDRESS_LENGTH represents the number of bytes used in a header Ethereum account address
pub const ADDRESS_LENGTH: usize = 20;

/// BLOOM_BYTE_LENGTH represents the number of bytes used in a header log bloom
pub const BLOOM_BYTE_LENGTH: usize = 256;

/// BLOCK_NONCE_LENGTH represents the number of bytes used in a header nonce
pub const NONCE_LENGTH: usize = 8;

/// Hash is the output of the cryptographic digest function
pub type Hash = [u8; HASH_LENGTH];

/// Address represents the 20 byte address of an Ethereum account
pub type Address = [u8; ADDRESS_LENGTH];

/// Bloom represents a 2048 bit bloom filter
pub type Bloom = [u8; BLOOM_BYTE_LENGTH];

/// Nonce represents a 64 bit nonce
pub type Nonce = [u8; NONCE_LENGTH];


/*
bytes parentHash;
        address coinbase;
        bytes root;
        bytes txHash;
        bytes receipHash;
        bytes bloom;
        uint256 number;
        uint256 gasLimit;
        uint256 gasUsed;
        uint256 time;
        bytes extraData;
        bytes mixDigest;
        bytes nonce;
        uint256 baseFee;
 */

/// Header contains block metadata in Celo Blockchain
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde", rename_all = "camelCase")]
pub struct Header {
    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub parent_hash: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    #[serde(rename = "miner")]
    pub coinbase: Address,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    #[serde(rename = "stateRoot")]
    pub root: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    #[serde(rename = "transactionsRoot")]
    pub tx_hash: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    #[serde(rename = "receiptsRoot")]
    pub receipt_hash: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    #[serde(rename = "logsBloom")]
    pub bloom: Bloom,

    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub number: Integer,

    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub gas_limit: Integer,

    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub gas_used: Integer,

    #[serde(rename = "timestamp")]
    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub time: Integer,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    #[serde(rename = "extraData")]
    pub extra: Vec<u8>,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub min_digest: Hash,

    #[serde(with = "crate::serialization::bytes::hexstring")]
    pub nonce: Nonce,

    #[serde(with = "crate::serialization::bytes::hexbigint")]
    pub base_fee: Integer,
}

impl Header {
    pub fn new() -> Self {
        Self {
            parent_hash: Hash::default(),
            coinbase: Address::default(),
            root: Hash::default(),
            tx_hash: Hash::default(),
            receipt_hash: Hash::default(),
            bloom: Bloom::default(),
            number: Integer::default(),
            gas_limit: Integer::default(),
            gas_used: Integer::default(),
            time: Integer::default(),
            extra: Vec::default(),
            min_digest: Hash::default(),
            nonce: Nonce::default(),
            base_fee: Integer::default(),
        }
    }

    pub fn hash(&self) -> Result<Hash, Kind> {
        if self.extra.len() >= ISTANBUL_EXTRA_VANITY_LENGTH {
            let istanbul_header = istanbul_filtered_header(&self, true);
            if istanbul_header.is_ok() {
                return rlp_hash(&istanbul_header?);
            }
        }

        rlp_hash(self)
    }

    pub fn hash_without_seal(&self) -> Result<Hash, Kind> {
        if self.extra.len() >= ISTANBUL_EXTRA_VANITY_LENGTH {
            let istanbul_header = istanbul_filtered_header(&self, false);
            if istanbul_header.is_ok() {
                return rlp_hash(&istanbul_header?);
            }
        }

        rlp_hash(self)
    }
}

impl FromRlp for Header {
    fn from_rlp(bytes: &[u8]) -> Result<Self, Kind> {
        rlp::decode(&bytes).map_err(|e| Kind::RlpDecodeError)
    }
}

impl ToRlp for Header {
    fn to_rlp(&self) -> Vec<u8> {
        rlp::encode(self)
    }
}

impl Encodable for Header {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(14);

        // parent_hash
        s.append(&self.parent_hash.as_ref());

        // coinbase
        s.append(&self.coinbase.as_ref());

        // root
        s.append(&self.root.as_ref());

        // tx_hash
        s.append(&self.tx_hash.as_ref());

        // receipt_hash
        s.append(&self.receipt_hash.as_ref());

        // bloom
        s.append(&self.bloom.as_ref());

        // number
        s.append(&big_int_to_rlp_compat_bytes(&self.number));

        // gas_limit
        s.append(&big_int_to_rlp_compat_bytes(&self.gas_limit));

        // gas_used
        s.append(&big_int_to_rlp_compat_bytes(&self.gas_used));

        // time
        s.append(&big_int_to_rlp_compat_bytes(&self.time));

        // extra
        s.append(&self.extra);

        // min_digest
        s.append(&self.min_digest.as_ref());

        // nonce
        s.append(&self.nonce.as_ref());

        // base_fee
        s.append(&big_int_to_rlp_compat_bytes(&self.base_fee));
    }
}

impl Decodable for Header {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Header {
            parent_hash: rlp_list_field_from_bytes(rlp, 0)?,
            coinbase: rlp_list_field_from_bytes(rlp, 1)?,
            root: rlp_list_field_from_bytes(rlp, 2)?,
            tx_hash: rlp_list_field_from_bytes(rlp, 3)?,
            receipt_hash: rlp_list_field_from_bytes(rlp, 4)?,
            bloom: rlp_list_field_from_bytes(rlp, 5)?,
            number: rlp_to_big_int(rlp, 6)?,
            gas_limit: rlp_to_big_int(rlp, 7)?,
            gas_used: rlp_to_big_int(rlp, 8)?,
            time: rlp_to_big_int(rlp, 9)?,
            extra: rlp.val_at(10)?,
            min_digest: rlp_list_field_from_bytes(rlp, 11)?,
            nonce: rlp_list_field_from_bytes(rlp, 12)?,
            base_fee: rlp_to_big_int(rlp, 13)?,
        })
    }
}

impl DefaultFrom for Bloom {
    fn default() -> Self {
        [0; BLOOM_BYTE_LENGTH]
    }
}

impl FromBytes for Bloom {
    fn from_bytes(data: &[u8]) -> Result<&Bloom, Kind> {
        slice_as_array_ref!(&data[..BLOOM_BYTE_LENGTH], BLOOM_BYTE_LENGTH)
    }
}

impl FromBytes for Address {
    fn from_bytes(data: &[u8]) -> Result<&Address, Kind> {
        slice_as_array_ref!(&data[..ADDRESS_LENGTH], ADDRESS_LENGTH)
    }
}

impl FromBytes for Nonce {
    fn from_bytes(data: &[u8]) -> Result<&Nonce, Kind> {
        slice_as_array_ref!(&data[..NONCE_LENGTH], NONCE_LENGTH)
    }
}

fn rlp_hash(header: &Header) -> Result<Hash, Kind> {
    let digest = keccak256(&rlp::encode(header));

    Ok(slice_as_array_ref!(&digest[..HASH_LENGTH], HASH_LENGTH)?.to_owned())
}

#[cfg(test)]
mod tests {
    use near_sdk::serde_json;
    use super::*;

    const HEADER_WITH_EMPTY_EXTRA: &str = "f901a6a07285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7948888f1f195afa192cfee860698584c030f4c9db1a0ecc60e00b3fe5ce9f6e1a10e5469764daf51f1fe93c22ec3f9a7583a80357217a0d35d334d87c0cc0a202e3756bf81fae08b1575f286c7ee7a3f8df4f0f3afc55da056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001825208845c47775c80";

    const IST_EXTRA: &str = "0000000000000000000000000000000000000000000000000000000000000000f89af8549444add0ec310f115a0e603b2d7db9f067778eaf8a94294fc7e8f22b3bcdcf955dd7ff3ba2ed833f8212946beaaed781d2d2ab6350f5c4566a2c6eaac407a6948be76812f765c24641ec63dc2852b378aba2b440b8410000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0";

    #[test]
    fn encodes_header_to_rlp() {
        let bytes = hex::decode(&HEADER_WITH_EMPTY_EXTRA).unwrap();
        let header = Header::from_rlp(&bytes).unwrap();
        let encoded_bytes = header.to_rlp();

        assert_eq!(encoded_bytes, bytes);
    }

    #[test]
    fn decodes_header_from_rlp() {
        let expected = vec![Header {
            parent_hash: to_hash(
                "7285abd5b24742f184ad676e31f6054663b3529bc35ea2fcad8a3e0f642a46f7",
            ),
            coinbase: to_hash("8888f1f195afa192cfee860698584c030f4c9db1"),
            root: to_hash("ecc60e00b3fe5ce9f6e1a10e5469764daf51f1fe93c22ec3f9a7583a80357217"),
            tx_hash: to_hash("d35d334d87c0cc0a202e3756bf81fae08b1575f286c7ee7a3f8df4f0f3afc55d"),
            receipt_hash: to_hash(
                "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            ),
            bloom: Bloom::default(),
            number: Integer::from(1),
            gas_limit: Default::default(),
            gas_used: Integer::from(0x5208),
            time: Integer::from(0x5c47775c),
            extra: Vec::default(),
            min_digest: [0; HASH_LENGTH],
            nonce: [0; NONCE_LENGTH],
            base_fee: Default::default(),
        }];

        for (bytes, expected_ist) in vec![hex::decode(&HEADER_WITH_EMPTY_EXTRA).unwrap()]
            .iter()
            .zip(expected)
        {
            let parsed = Header::from_rlp(&bytes).unwrap();

            assert_eq!(parsed, expected_ist);
        }
    }

    #[test]
    fn serializes_and_deserializes_to_json() {
        for bytes in vec![hex::decode(&HEADER_WITH_EMPTY_EXTRA).unwrap()].iter() {
            let parsed = Header::from_rlp(&bytes).unwrap();
            let json_string = serde_json::to_string(&parsed).unwrap();
            let deserialized_from_json: Header = serde_json::from_str(&json_string).unwrap();

            assert_eq!(parsed, deserialized_from_json);
        }
    }

    #[test]
    fn generates_valid_header_hash() {
        for (extra_bytes, hash_str) in vec![(
            IST_EXTRA,
            "5c012c65d46edfbfca86a426da5111c51114b75577fec9b82161d3e05d83b723",
        )]
            .iter()
        {
            let expected_hash: Hash = Hash::from_bytes(&hex::decode(hash_str).unwrap())
                .unwrap()
                .to_owned();
            let mut header = Header::new();
            header.extra = hex::decode(&extra_bytes).unwrap();

            // for istanbul consensus
            assert_eq!(header.hash().unwrap(), expected_hash);

            // append useless information to extra-data
            header.extra.extend(vec![1, 2, 3]);

            assert_eq!(header.hash().unwrap(), rlp_hash(&header).unwrap());
        }
    }

    pub fn to_hash<T>(data: &str) -> T
        where
            T: FromBytes + Clone,
    {
        T::from_bytes(&hex::decode(data).unwrap())
            .unwrap()
            .to_owned()
    }
}
