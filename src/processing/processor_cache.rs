use std::collections::BTreeMap;
use dash_spv_models::common::LLMQType;
use dash_spv_models::llmq::{LLMQIndexedHash, LLMQSnapshot};
use dash_spv_models::masternode::{MasternodeEntry, MasternodeList};
use dash_spv_primitives::crypto::UInt256;

#[derive(Clone, Debug)]
pub struct MasternodeProcessorCache {
    pub llmq_members: BTreeMap<LLMQType, BTreeMap<UInt256, Vec<MasternodeEntry>>>,
    pub llmq_indexed_members: BTreeMap<LLMQType, BTreeMap<LLMQIndexedHash, Vec<MasternodeEntry>>>,
    pub mn_lists: BTreeMap<UInt256, MasternodeList>,
    pub llmq_snapshots: BTreeMap<UInt256, LLMQSnapshot>,
}
impl Default for MasternodeProcessorCache {
    fn default() -> Self {
        MasternodeProcessorCache {
            llmq_members: BTreeMap::new(),
            llmq_indexed_members: BTreeMap::new(),
            llmq_snapshots: BTreeMap::new(),
            mn_lists: BTreeMap::new(),
        }
    }
}

impl MasternodeProcessorCache {
    pub fn add_masternode_list(&mut self, block_hash: UInt256, list: MasternodeList) {
        self.mn_lists.insert(block_hash, list);
    }
    pub fn get_quorum_members_of_type(&mut self, r#type: LLMQType) -> Option<&mut BTreeMap<UInt256, Vec<MasternodeEntry>>> {
        self.llmq_members.get_mut(&r#type)
    }

    pub fn get_indexed_quorum_members_of_type(&mut self, r#type: LLMQType) -> Option<&mut BTreeMap<LLMQIndexedHash, Vec<MasternodeEntry>>> {
        self.llmq_indexed_members.get_mut(&r#type)
    }

    pub fn get_quorum_members(&mut self, r#type: LLMQType, block_hash: UInt256) -> Option<Vec<MasternodeEntry>> {
        let map_by_type_opt = self.get_quorum_members_of_type(r#type);
        if map_by_type_opt.is_some() {
            if let Some(members) = map_by_type_opt.as_ref().unwrap().get(&block_hash) {
                return Some(members.clone());
            }
        }
        None
    }
}