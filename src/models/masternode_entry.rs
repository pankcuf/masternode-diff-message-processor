use byte::{BytesExt, TryRead};
use std::collections::BTreeMap;
use crate::common::{Block, SocketAddress};
use crate::common::masternode_type::MasternodeType;
use crate::consensus::Encodable;
use crate::crypto::byte_util::Zeroable;
use crate::crypto::data_ops::short_hex_string_from;
use crate::crypto::{UInt128, UInt160, UInt256, UInt384};
use crate::hashes::{sha256, sha256d, Hash};
use crate::models::OperatorPublicKey;

// (block_height, protocol_version, bls_version)
#[derive(Clone, Copy)]
pub struct MasternodeReadContext(pub u32, pub u32, pub u16);

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct MasternodeEntry {
    pub provider_registration_transaction_hash: UInt256,
    pub confirmed_hash: UInt256,
    pub confirmed_hash_hashed_with_provider_registration_transaction_hash: Option<UInt256>,
    pub socket_address: SocketAddress,
    pub operator_public_key: OperatorPublicKey,
    pub previous_operator_public_keys: BTreeMap<Block, OperatorPublicKey>,
    pub previous_entry_hashes: BTreeMap<Block, UInt256>,
    pub previous_validity: BTreeMap<Block, bool>,
    pub known_confirmed_at_height: Option<u32>,
    pub update_height: u32,
    pub key_id_voting: UInt160,
    pub is_valid: bool,
    pub mn_type: MasternodeType,
    pub platform_http_port: u16,
    pub platform_node_id: UInt160,
    pub entry_hash: UInt256,
}
impl std::fmt::Debug for MasternodeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MasternodeEntry")
            .field("provider_registration_transaction_hash", &self.provider_registration_transaction_hash)
            .field("confirmed_hash", &self.confirmed_hash)
            .field("confirmed_hash_hashed_with_provider_registration_transaction_hash", &self.confirmed_hash_hashed_with_provider_registration_transaction_hash.unwrap_or(UInt256::MIN))
            .field("socket_address", &self.socket_address)
            .field("operator_public_key", &self.operator_public_key)
            .field("previous_operator_public_keys", &self.previous_operator_public_keys)
            .field("previous_entry_hashes", &self.previous_entry_hashes)
            .field("previous_validity", &self.previous_validity)
            .field("known_confirmed_at_height", &self.known_confirmed_at_height.unwrap_or(0))
            .field("update_height", &self.update_height)
            .field("key_id_voting", &self.key_id_voting)
            .field("is_valid", &self.is_valid)
            .field("mn_type", &self.mn_type)
            .field("platform_http_port", &self.platform_http_port)
            .field("platform_node_id", &self.platform_node_id)
            .field("entry_hash", &self.entry_hash)
            .finish()
    }
}

impl<'a> TryRead<'a, MasternodeReadContext> for MasternodeEntry {
    fn try_read(bytes: &'a [u8], context: MasternodeReadContext) -> byte::Result<(Self, usize)> {
        // println!("MasternodeEntry:read: {:?}", bytes.to_hex());
        let MasternodeReadContext (block_height, protocol_version, bls_version) = context;
        let offset = &mut 0;
        let provider_registration_transaction_hash =
            bytes.read_with::<UInt256>(offset, byte::LE)?;
        println!("MasternodeEntry:provider_registration_transaction_hash: {:?}", provider_registration_transaction_hash);
        let confirmed_hash = bytes.read_with::<UInt256>(offset, byte::LE)?;
        println!("MasternodeEntry:confirmed_hash: {:?}", confirmed_hash);
        let ip_address = bytes.read_with::<UInt128>(offset, byte::LE)?;
        println!("MasternodeEntry:ip_address: {:?}", ip_address);
        let port = bytes.read_with::<u16>(offset, byte::BE)?;
        println!("MasternodeEntry:port: {:?}", port);
        let socket_address = SocketAddress { ip_address, port };
        let operator_public_key = bytes.read_with::<UInt384>(offset, byte::LE)?;
        println!("MasternodeEntry:operator_public_key: {:?}", operator_public_key);
        let key_id_voting = bytes.read_with::<UInt160>(offset, byte::LE)?;
        println!("MasternodeEntry:key_id_voting: {:?}", key_id_voting);
        let is_valid = bytes.read_with::<u8>(offset, byte::LE)
            .unwrap_or(0);
        println!("MasternodeEntry:is_valid: {:?}", is_valid);
        let mn_type = if protocol_version >= 70227 {
            // MasternodeType::Regular
            bytes.read_with::<MasternodeType>(offset, byte::LE)?
        } else {
            MasternodeType::Regular
        };
        println!("MasternodeEntry:mn_type: {:?}", mn_type);
        let (platform_http_port, platform_node_id) = if mn_type == MasternodeType::HighPerformance {
            (bytes.read_with::<u16>(offset, byte::LE)?.swap_bytes(),
             bytes.read_with::<UInt160>(offset, byte::LE)?)
        } else {
            (0u16, UInt160::MIN)
        };
        println!("MasternodeEntry:platform_http_port: {:?}", platform_http_port);
        println!("MasternodeEntry:platform_node_id: {:?}", platform_node_id);
        let mut entry = Self::new(
            provider_registration_transaction_hash,
            confirmed_hash,
            socket_address,
            key_id_voting,
            OperatorPublicKey { data: operator_public_key, version: bls_version },
            is_valid,
            mn_type,
            platform_http_port,
            platform_node_id,
            block_height
        );
        if !entry.confirmed_hash.is_zero() && block_height != u32::MAX {
            entry.known_confirmed_at_height = Some(block_height);
        }
        Ok((entry, *offset))
    }
}

impl MasternodeEntry {
    pub fn new(
        provider_registration_transaction_hash: UInt256,
        confirmed_hash: UInt256,
        socket_address: SocketAddress,
        key_id_voting: UInt160,
        operator_public_key: OperatorPublicKey,
        is_valid: u8,
        mn_type: MasternodeType,
        platform_http_port: u16,
        platform_node_id: UInt160,
        update_height: u32,
    ) -> Self {
        let entry_hash = MasternodeEntry::calculate_entry_hash(
            provider_registration_transaction_hash,
            confirmed_hash,
            socket_address,
            operator_public_key,
            key_id_voting,
            is_valid,
            mn_type,
            platform_http_port,
            platform_node_id
        );
        println!("MasternodeEntry:entry_hash: {:?}", entry_hash);
        Self {
            provider_registration_transaction_hash,
            confirmed_hash,
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                Self::hash_confirmed_hash(confirmed_hash, provider_registration_transaction_hash),
            ),
            socket_address,
            operator_public_key,
            previous_operator_public_keys: Default::default(),
            previous_entry_hashes: Default::default(),
            previous_validity: Default::default(),
            known_confirmed_at_height: None,
            update_height,
            key_id_voting,
            is_valid: is_valid != 0,
            mn_type,
            platform_http_port,
            platform_node_id,
            entry_hash,
        }
    }

    fn calculate_entry_hash(
        provider_registration_transaction_hash: UInt256,
        confirmed_hash: UInt256,
        socket_address: SocketAddress,
        operator_public_key: OperatorPublicKey,
        key_id_voting: UInt160,
        is_valid: u8,
        mn_type: MasternodeType,
        platform_http_port: u16,
        platform_node_id: UInt160,
    ) -> UInt256 {
        const HASH_IMPORTANT_DATA_LENGTH: usize = 32 + 32 + 16 + 2 + 48 + 20 + 1;
        let mut buffer: Vec<u8> = Vec::with_capacity(HASH_IMPORTANT_DATA_LENGTH);
        provider_registration_transaction_hash.enc(&mut buffer);
        confirmed_hash.enc(&mut buffer);
        socket_address.ip_address.enc(&mut buffer);
        socket_address.port.swap_bytes().enc(&mut buffer);
        operator_public_key.data.enc(&mut buffer);
        key_id_voting.enc(&mut buffer);
        is_valid.enc(&mut buffer);
        if operator_public_key.version == 2 /*Basic BLS*/ {
            let mn_type_u16: u16 = mn_type.into();
            mn_type_u16.enc(&mut buffer);
            if mn_type == MasternodeType::HighPerformance {
                // swap?
                platform_http_port.enc(&mut buffer);
                platform_node_id.enc(&mut buffer);
            }
        }
        UInt256(sha256d::Hash::hash(&buffer).into_inner())
    }

    pub fn confirmed_hash_at(&self, block_height: u32) -> Option<UInt256> {
        self.known_confirmed_at_height.and_then(|h| (h <= block_height)
            .then_some(self.confirmed_hash))
    }

    pub fn update_confirmed_hash(&mut self, hash: UInt256) {
        self.confirmed_hash = hash;
        if !self.provider_registration_transaction_hash.is_zero() {
            self.update_confirmed_hash_hashed_with_provider_registration_transaction_hash();
        }
    }

    pub fn update_confirmed_hash_hashed_with_provider_registration_transaction_hash(&mut self) {
        let hash = Self::hash_confirmed_hash(
            self.confirmed_hash,
            self.provider_registration_transaction_hash,
        );
        self.confirmed_hash_hashed_with_provider_registration_transaction_hash = Some(hash)
    }

    pub fn confirmed_hash_hashed_with_provider_registration_transaction_hash_at(
        &self,
        block_height: u32,
    ) -> Option<UInt256> {
        if self.known_confirmed_at_height.is_none()
            || self.known_confirmed_at_height? <= block_height
        {
            self.confirmed_hash_hashed_with_provider_registration_transaction_hash
        } else {
            Some(Self::hash_confirmed_hash(
                UInt256::default(),
                self.provider_registration_transaction_hash,
            ))
        }
    }

    pub fn host(&self) -> String {
        let ip = self.socket_address.ip_address;
        let port = self.socket_address.port;
        format!("{}:{}", ip.to_string(), port.to_string())
    }

    pub fn payload_data(&self) -> UInt256 {
        Self::calculate_entry_hash(
            self.provider_registration_transaction_hash,
            self.confirmed_hash,
            self.socket_address,
            self.operator_public_key,
            self.key_id_voting,
            u8::from(self.is_valid),
            self.mn_type,
            self.platform_http_port,
            self.platform_node_id
        )
    }

    pub fn hash_confirmed_hash(confirmed_hash: UInt256, pro_reg_tx_hash: UInt256) -> UInt256 {
        UInt256(sha256::Hash::hash(&[pro_reg_tx_hash.0, confirmed_hash.0].concat()).into_inner())
    }

    pub fn is_valid_at(&self, block_height: u32) -> bool {
        if self.previous_validity.is_empty() || block_height == u32::MAX {
            return self.is_valid;
        }
        let mut min_distance = u32::MAX;
        let mut is_valid = self.is_valid;
        for (&Block { height, .. }, &validity) in &self.previous_validity {
            if height <= block_height {
                continue;
            }
            let distance = height - block_height;
            if distance < min_distance {
                min_distance = distance;
                is_valid = validity;
            }
        }
        is_valid
    }

    pub fn operator_public_key_at(&self, block_height: u32) -> OperatorPublicKey {
        if self.previous_operator_public_keys.is_empty() {
            return self.operator_public_key;
        }
        let mut min_distance = u32::MAX;
        let mut used_previous_operator_public_key_at_block_hash = self.operator_public_key;
        for (&Block { height, .. }, &key) in &self.previous_operator_public_keys {
            if height <= block_height {
                continue;
            }
            let distance = height - block_height;
            if distance < min_distance {
                min_distance = distance;
                used_previous_operator_public_key_at_block_hash = key;
            }
        }
        used_previous_operator_public_key_at_block_hash
    }

    pub fn entry_hash_at(&self, block_height: u32) -> UInt256 {
        if self.previous_entry_hashes.is_empty() || block_height == u32::MAX {
            return self.entry_hash;
        }
        let mut min_distance = u32::MAX;
        let mut used_hash = self.entry_hash;
        for (&Block { height, .. }, &hash) in &self.previous_entry_hashes {
            if height <= block_height {
                continue;
            }
            let distance = height - block_height;
            if distance < min_distance {
                min_distance = distance;
                println!("SME Hash for proTxHash {:?} : Using {:?} instead of {:?} for list at block height {}", self.provider_registration_transaction_hash, hash, used_hash, block_height);
                used_hash = hash;
            }
        }
        used_hash
    }

    pub fn unique_id(&self) -> String {
        short_hex_string_from(&self.provider_registration_transaction_hash.0)
    }

    pub fn update_with_previous_entry(&mut self, entry: &mut MasternodeEntry, block: Block) {
        self.previous_validity = entry
            .previous_validity
            .clone()
            .into_iter()
            .filter(|(block, _)| block.height < self.update_height)
            .collect();
        if entry.is_valid_at(self.update_height) != self.is_valid {
            self.previous_validity.insert(block, entry.is_valid);
        }
        self.previous_operator_public_keys = entry
            .previous_operator_public_keys
            .clone()
            .into_iter()
            .filter(|(block, _)| block.height < self.update_height)
            .collect();
        if entry.operator_public_key_at(self.update_height) != self.operator_public_key {
            self.previous_operator_public_keys.insert(block, entry.operator_public_key);
        }
        let old_prev_mn_entry_hashes = entry
            .previous_entry_hashes
            .clone()
            .into_iter()
            .filter(|(block, _)| block.height < self.update_height)
            .collect();
        self.previous_entry_hashes = old_prev_mn_entry_hashes;
        if entry.entry_hash_at(self.update_height) != self.entry_hash {
            self.previous_entry_hashes.insert(block, entry.entry_hash);
        }
    }
}
