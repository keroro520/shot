use crate::utils::Secp;
use ckb_crypto::secp::{Privkey, Signature};
use ckb_hash::blake2b_256;
use ckb_types::{
    packed::{CellDep, Script},
    H160, H256,
};
use std::str::FromStr;

pub struct User {
    pk: String,
    privkey: Privkey,
    lock_args: H160,
    secp: Secp,
}

impl User {
    pub fn new(pk: String, secp: Secp) -> Self {
        let privkey = privkey_from(&pk);
        let lock_args = {
            let pubkey = privkey
                .pubkey()
                .expect("failed to generate pubkey from privkey");
            H160::from_slice(&blake2b_256(pubkey.serialize())[0..20])
                .expect("failed to generate hash(H160) from pubkey")
        };
        Self {
            pk,
            privkey,
            lock_args,
            secp,
        }
    }

    pub fn sign_recoverable(&self, message: H256) -> Signature {
        self.privkey.sign_recoverable(&message).unwrap()
    }

    pub fn lock_script(&self) -> Script {
        self.secp.lock_script(&self.lock_args)
    }

    pub fn cell_dep(&self) -> CellDep {
        self.secp.group_cell_dep()
    }
}

impl Clone for User {
    fn clone(&self) -> Self {
        Self::new(self.pk.clone(), self.secp.clone())
    }
}

fn privkey_from(pk: &str) -> Privkey {
    let privkey_str = if pk.starts_with("0x") || pk.starts_with("0X") {
        &pk[2..]
    } else {
        pk
    };
    Privkey::from_str(privkey_str.trim()).expect("parse private key")
}
