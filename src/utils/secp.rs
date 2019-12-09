use crate::utils::Client;
use ckb_chain_spec::{build_genesis_type_id_script, OUTPUT_INDEX_SECP256K1_BLAKE160_SIGHASH_ALL};
use ckb_types::core::DepType;
use ckb_types::{
    bytes::Bytes,
    core::{BlockView, ScriptHashType},
    packed::{Byte32, CellDep, OutPoint, Script},
    prelude::*,
    H160,
};

#[derive(Clone)]
pub struct Secp {
    group_cell_dep: CellDep,
    type_hash: Byte32,
}

impl Secp {
    pub fn group_cell_dep(&self) -> CellDep {
        self.group_cell_dep.clone()
    }

    pub fn lock_script(&self, address: &H160) -> Script {
        Script::new_builder()
            .args(Bytes::from(address.as_bytes()).pack())
            .code_hash(self.type_hash.clone())
            .hash_type(ScriptHashType::Type.into())
            .build()
    }

    pub fn init(rpc_urls: &[String]) -> Self {
        let client = Client::init(rpc_urls);
        let block: BlockView = client.get_safe_block(0);
        let group_transaction = &block.transactions()[1];

        let group_cell_dep = CellDep::new_builder()
            .out_point(OutPoint::new(
                group_transaction.hash(),
                0, // INDEX_OF_SECP_DEP_GROUPS = 0
            ))
            .dep_type(DepType::DepGroup.into())
            .build();
        let type_hash = build_genesis_type_id_script(OUTPUT_INDEX_SECP256K1_BLAKE160_SIGHASH_ALL)
            .calc_script_hash();
        Self {
            group_cell_dep,
            type_hash,
        }
    }
}
