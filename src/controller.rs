use crate::config::ControllerConfig;
use crate::utils::{Client, User};
use ckb_types::{bytes::Bytes, core::TransactionView, packed::WitnessArgs, prelude::*, H256};
use crossbeam_channel::Receiver;
use failure::_core::time::Duration;
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Instant;

pub struct Controller {
    user: User,
    client: Client,
    receiver: Receiver<TransactionView>,
    config: ControllerConfig,
}

impl Controller {
    pub fn new(
        user: User,
        rpc_urls: &[String],
        receiver: Receiver<TransactionView>,
        config: ControllerConfig,
    ) -> Self {
        let client = Client::init(rpc_urls);
        Self {
            user,
            client,
            receiver,
            config,
        }
    }

    pub fn serve(self) -> JoinHandle<()> {
        spawn(move || self.do_serve())
    }

    fn do_serve(self) {
        // TODO
        let tps = (self.config.tps * 1_000_000_000.0) as u32;
        let gap = Duration::from_secs(1_000_000_000) / tps;
        let mut last_sent = Instant::now();
        while let Ok(raw_transaction) = self.receiver.recv() {
            let transaction = self.sign_transaction(raw_transaction);
            self.send_transaction(transaction);

            let elapsed = last_sent.elapsed();
            last_sent = Instant::now();
            if elapsed < gap {
                sleep(gap - elapsed);
            }
        }
    }

    fn sign_transaction(&self, transaction: TransactionView) -> TransactionView {
        let tx_hash = transaction.hash();
        let signed_witness = {
            // message = blake2b([tx_hash, raw_witness_len, raw_witness])
            let raw_witness = WitnessArgs::new_builder()
                .lock(Some(Bytes::from(vec![0u8; 65])).pack())
                .build();
            let raw_witness_len = raw_witness.as_bytes().len() as u64;

            let mut blake2b = ckb_hash::new_blake2b();
            let mut message = [0u8; 32];
            blake2b.update(&tx_hash.raw_data());
            blake2b.update(&raw_witness_len.to_le_bytes());
            blake2b.update(&raw_witness.as_bytes());
            blake2b.finalize(&mut message);

            let signature = self.user.sign_recoverable(H256::from(message));
            WitnessArgs::new_builder()
                .lock(Some(Bytes::from(signature.serialize())).pack())
                .build()
                .as_bytes()
                .pack()
        };
        transaction
            .as_advanced_builder()
            .set_witnesses(vec![signed_witness])
            .build()
    }

    fn send_transaction(&self, transaction: TransactionView) {
        let index = transaction.hash().nth0().as_slice()[0] as usize % self.client.num_nodes();
        // TODO handle PoolTransactionDuplicated
        let _ = self
            .client
            .get(index)
            .send_transaction_result(transaction.data().into());
    }
}
