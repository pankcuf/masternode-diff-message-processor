use std::convert::Into;
use byte::{BytesExt, LE};
use byte::ctx::Bytes;
use hashes::{Hash, sha256d};
use crate::common::llmq_type::LLMQType;
use crate::consensus::{Decodable, Encodable, WriteExt};
use crate::consensus::encode::{consensus_encode_with_size, VarInt};
use crate::crypto::byte_util::{Data, UInt256, UInt384, UInt768};

// #[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct QuorumEntry<'a> {
    pub version: u16,
    pub quorum_hash: UInt256,
    pub quorum_public_key: UInt384,
    pub quorum_threshold_signature: UInt768,
    pub quorum_verification_vector_hash: UInt256,
    pub all_commitment_aggregated_signature: UInt768,
    pub signers_count: VarInt,
    pub llmq_type: LLMQType,
    pub valid_members_count: VarInt,
    pub signers_bitset: &'a [u8],
    pub valid_members_bitset: &'a [u8],
    pub length: usize,
    pub quorum_entry_hash: UInt256,
    pub verified: bool,
    pub saved: bool,
    pub commitment_hash: Option<UInt256>,
}

impl<'a> QuorumEntry<'a> {
    pub fn new(message: &'a [u8], data_offset: usize) -> Option<Self> {
        let length = message.len();
        let offset = &mut data_offset.clone();

        let version = match message.read_with::<u16>(offset, LE) {
            Ok(data) => data,
            _ => { return None; }
        };
        let llmq_type = match message.read_with::<u8>(offset, LE) {
            Ok(data) => data,
            _ => { return None; }
        };
        let quorum_hash = match message.read_with::<UInt256>(offset, LE) {
            Ok(data) => data,
            _ => { return None; }
        };

        let signers_count = match VarInt::consensus_decode(&message[*offset..]) {
            Ok(data) => data,
            Err(_err) => { return None; }
        };
        *offset += signers_count.len();

        let signers_buffer_length: usize = ((signers_count.0 as usize) + 7) / 8;
        if length - *offset < signers_buffer_length { return None; }
        let signers_bitset: &[u8] = message.read_with(offset, Bytes::Len(signers_buffer_length)).unwrap();

        let valid_members_count = match VarInt::consensus_decode(&message[*offset..]) {
            Ok(data) => data,
            Err(_err) => { return None; }
        };
        *offset += valid_members_count.len();

        let valid_members_count_buffer_length: usize = ((valid_members_count.0 as usize) + 7) / 8;
        if length - *offset < valid_members_count_buffer_length { return None; }
        let valid_members_bitset: &[u8] = message.read_with(offset, Bytes::Len(valid_members_count_buffer_length)).unwrap();

        let quorum_public_key = match message.read_with::<UInt384>(offset, LE) {
            Ok(data) => data,
            Err(_err) => { return None; }
        };
        let quorum_verification_vector_hash = match message.read_with::<UInt256>(offset, LE) {
            Ok(data) => data,
            Err(_err) => { return None; }
        };
        let quorum_threshold_signature = match message.read_with::<UInt768>(offset, LE) {
            Ok(data) => data,
            Err(_err) => { return None; }
        };
        let all_commitment_aggregated_signature = match message.read_with::<UInt768>(offset, LE) {
            Ok(data) => data,
            Err(_err) => { return None; }
        };


        let llmq_type: LLMQType = llmq_type.into();
        let q_data = &QuorumEntry::generate_data(
            version, llmq_type, quorum_hash,
            signers_count.clone(), &signers_bitset,
            valid_members_count.clone(), &valid_members_bitset,
            quorum_public_key, quorum_verification_vector_hash, quorum_threshold_signature,
            all_commitment_aggregated_signature);
        let quorum_entry_hash = UInt256(sha256d::Hash::hash(q_data).into_inner());
        let length = *offset - data_offset;
        //LLMQType::try_from(llmq_type)
        Some(QuorumEntry {
            version,
            quorum_hash,
            quorum_public_key,
            quorum_threshold_signature,
            quorum_verification_vector_hash,
            all_commitment_aggregated_signature,
            signers_count: signers_count.clone(),
            llmq_type,
            valid_members_count: valid_members_count.clone(),
            signers_bitset,
            valid_members_bitset,
            length,
            quorum_entry_hash,
            verified: false,
            saved: false,
            commitment_hash: None
        })
    }

    pub fn generate_data(
        version: u16,
        llmq_type: LLMQType,
        quorum_hash: UInt256,
        signers_count: VarInt,
        signers_bitset: &[u8],
        valid_members_count: VarInt,
        valid_members_bitset: &[u8],
        quorum_public_key: UInt384,
        quorum_verification_vector_hash: UInt256,
        quorum_threshold_signature: UInt768,
        all_commitment_aggregated_signature: UInt768
    ) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let offset: &mut usize = &mut 0;
        let llmq_u8: u8 = llmq_type.into();
        *offset += version.consensus_encode(&mut buffer).unwrap();
        *offset += llmq_u8.consensus_encode(&mut buffer).unwrap();
        *offset += quorum_hash.consensus_encode(&mut buffer).unwrap();
        *offset += signers_count.consensus_encode(&mut buffer).unwrap();
        buffer.emit_slice(&signers_bitset).unwrap();
        *offset += signers_bitset.len();

        *offset += valid_members_count.consensus_encode(&mut buffer).unwrap();
        buffer.emit_slice(&valid_members_bitset).unwrap();
        *offset += valid_members_bitset.len();

        *offset += quorum_public_key.consensus_encode(&mut buffer).unwrap();

        *offset += quorum_verification_vector_hash.consensus_encode(&mut buffer).unwrap();
        *offset += quorum_threshold_signature.consensus_encode(&mut buffer).unwrap();
        *offset += all_commitment_aggregated_signature.consensus_encode(&mut buffer).unwrap();
        buffer
    }

    pub fn to_data(&self) -> Vec<u8> {
        QuorumEntry::generate_data(
            self.version, self.llmq_type, self.quorum_hash,
            self.signers_count, self.signers_bitset,
            self.valid_members_count, self.valid_members_bitset,
            self.quorum_public_key, self.quorum_verification_vector_hash,
            self.quorum_threshold_signature, self.all_commitment_aggregated_signature)
    }

    pub fn llmq_quorum_hash(&self) -> UInt256 {
        let mut buffer: Vec<u8> = Vec::with_capacity(33);
        let offset: &mut usize = &mut 0;
        let llmq_u8: u8 = self.llmq_type.into();
        *offset += llmq_u8.consensus_encode(&mut buffer).unwrap();
        *offset += self.quorum_hash.consensus_encode(&mut buffer).unwrap();
        UInt256(sha256d::Hash::hash(&buffer).into_inner())
    }

    pub fn commitment_data(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let offset: &mut usize = &mut 0;
        let llmq_u8: u8 = self.llmq_type.into();
        *offset += llmq_u8.consensus_encode(&mut buffer).unwrap();
        *offset += self.quorum_hash.consensus_encode(&mut buffer).unwrap();
        *offset += self.valid_members_count.consensus_encode(&mut buffer).unwrap();
        *offset += consensus_encode_with_size(self.valid_members_bitset, &mut buffer).unwrap();
        *offset += self.quorum_public_key.consensus_encode(&mut buffer).unwrap();
        *offset += self.quorum_verification_vector_hash.consensus_encode(&mut buffer).unwrap();
        buffer
    }

    pub fn commitment_hash(&mut self) -> UInt256 {
        if self.commitment_hash.is_none() {
            self.commitment_hash = Some(UInt256(sha256d::Hash::hash(&self.commitment_data()).into_inner()));
        }
        self.commitment_hash.unwrap()
    }

    pub fn validate_payload(&mut self) -> bool {
        // The quorumHash must match the current DKG session
        // todo
        // The byte size of the signers and validMembers bitvectors must match “(quorumSize + 7) / 8”
        if self.signers_bitset.len() != (self.signers_count.0 as usize + 7) / 8 {
            println!("Error: The byte size of the signers bitvectors ({}) must match “(quorumSize + 7) / 8 ({})“", self.signers_bitset.len(), (self.signers_count.0 + 7) / 8);
            return false;
        }
        if self.valid_members_bitset.len() != (self.valid_members_count.0 as usize + 7) / 8 {
            println!("Error: The byte size of the validMembers bitvectors ({}) must match “(quorumSize + 7) / 8 ({})", self.valid_members_bitset.len(), (self.valid_members_count.0 + 7) / 8);
            return false;
        }

        // No out-of-range bits should be set in byte representation of the signers and validMembers bitvectors
        let mut signers_offset: usize = (self.signers_count.0 as usize) / 8;
        let signers_last_byte = match self.signers_bitset.read_with::<u8>(&mut signers_offset, LE) {
            Ok(data) => data,
            Err(_err) => 0
        };
        let signers_mask = u8::MAX >> (8 - signers_offset) << (8 - signers_offset);
        if signers_last_byte & signers_mask != 0 {
            println!("Error: No out-of-range bits should be set in byte representation of the signers bitvector");
            return false;
        }

        let mut valid_members_offset = self.valid_members_count.0 as usize / 8;
        let valid_members_last_byte: u8 = self.valid_members_bitset.read_with::<u8>(&mut valid_members_offset, LE).unwrap();
        let valid_members_mask = u8::MAX >> (8 - valid_members_offset) << (8 - valid_members_offset);
        if valid_members_last_byte & valid_members_mask != 0 {
            println!("Error: No out-of-range bits should be set in byte representation of the validMembers bitvector");
            return false;
        }
        let quorum_threshold = self.llmq_type.quorum_threshold();
        // The number of set bits in the signers and validMembers bitvectors must be at least >= quorumThreshold
        if self.signers_bitset.true_bits_count() < quorum_threshold as u64 {
            println!("Error: The number of set bits in the signers bitvector {} must be at least >= quorumThreshold {}", self.signers_bitset.true_bits_count(), quorum_threshold);
            return false;
        }
        if self.valid_members_bitset.true_bits_count() < quorum_threshold as u64 {
            println!("Error: The number of set bits in the validMembers bitvector {} must be at least >= quorumThreshold {}", self.valid_members_bitset.true_bits_count(), quorum_threshold);
            return false;
        }
        true
    }

    // This validation moved into external lib with bls
    // The quorumSig must validate against the quorumPublicKey and the commitmentHash. As this is a recovered threshold signature, normal signature verification can be performed, without the need of the full quorum verification vector. The commitmentHash is calculated in the same way as in the commitment phase.
    /*pub fn validate_signatures(&mut self) -> bool {
        let all_commitment_aggregated_signature_validated = BLSKey::verify_secure_aggregated(self.commitment_hash(), self.all_commitment_aggregated_signature, public_keys);
        if !all_commitment_aggregated_signature_validated {
            return false;
        }
        // The sig must validate against the commitmentHash and all public keys determined by the
        // signers bitvector. This is an aggregated BLS signature verification.
        if BLSKey::verify(self.commitment_hash(), self.quorum_threshold_signature, self.quorum_public_key) {
            self.verified = true;
            true
        } else {
            println!("Issue with quorumSignatureValidated");
            false
        }
    }*/
}
