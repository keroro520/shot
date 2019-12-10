use crate::utils::{Client, LiveCell, Unspent, User};
use ckb_types::core::{BlockNumber, BlockView};
use ckb_types::packed::OutPoint;
use crossbeam_channel::Sender;
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

pub struct Collector {
    user: User,
    client: Client,
    sender: Sender<LiveCell>,
}

impl Collector {
    pub fn new(user: User, rpc_urls: &[String], sender: Sender<LiveCell>) -> Self {
        let client = Client::init(rpc_urls);
        Self {
            client,
            sender,
            user,
        }
    }

    // Block until synchrozing to the tip
    pub fn serve(self) -> JoinHandle<()> {
        let tip = self.client.get_safe_tip();
        let mut unspent = Unspent::new();
        for block_number in 1..=tip {
            // FIXME 0..=tip
            let block = self
                .client
                .get_block_by_number(block_number)
                .unwrap()
                .into();
            let dead_out_points = self.dead_out_points(&block);
            let live_cells = self.live_cells(&block);
            unspent.update(&dead_out_points, live_cells);

            if block_number % 1000 == 0 {
                println!("#{}", block_number);
            }
        }
        for (_, cell) in unspent.into_iter() {
            self.sender.send(cell).unwrap();
        }

        spawn(move || self.do_serve(tip))
    }

    fn do_serve(self, from: BlockNumber) {
        let mut block_number = from + 1;
        loop {
            let tip: u64 = self.client.get_tip_block_number();
            if block_number + 3 < tip {
                sleep(Duration::from_secs(1));
                continue;
            }

            let block = self.client.get_safe_block(block_number);
            let live_cells = self.live_cells(&block);
            for cell in live_cells.into_iter() {
                self.sender.send(cell).unwrap();
            }
            block_number += 1;
        }
    }

    fn live_cells(&self, block: &BlockView) -> Vec<LiveCell> {
        let block_number = block.number();
        let lock_hash = self.user.lock_script().calc_script_hash();
        let mut lives = Vec::new();
        for (tx_index, transaction) in block.transactions().into_iter().enumerate() {
            for (index, cell_output) in transaction.outputs().into_iter().enumerate() {
                if lock_hash != cell_output.lock().calc_script_hash() {
                    continue;
                }
                let out_point = OutPoint::new(transaction.hash(), index as u32);
                let live_cell = LiveCell {
                    cell_output,
                    out_point,
                    tx_index,
                    block_number,
                };
                lives.push(live_cell);
            }
        }
        lives
    }

    fn dead_out_points(&self, block: &BlockView) -> Vec<OutPoint> {
        let mut deads = Vec::new();
        for transaction in block.transactions().iter().skip(1) {
            for input in transaction.input_pts_iter() {
                deads.push(input.clone());
            }
        }
        deads
    }
}
