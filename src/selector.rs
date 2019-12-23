use crate::utils::constants::{CELLBASE_MATURITY, MIN_INPUT_CAPACITY};
use crate::utils::LiveCell;
use ckb_types::prelude::Unpack;
use crossbeam_channel::{Receiver, Sender};
use std::thread::{spawn, JoinHandle};

pub struct Selector {
    receiver: Receiver<LiveCell>,
    sender: Sender<Vec<LiveCell>>,
}

impl Selector {
    pub fn new(receiver: Receiver<LiveCell>, sender: Sender<Vec<LiveCell>>) -> Self {
        Self { receiver, sender }
    }

    pub fn serve(self) -> JoinHandle<()> {
        spawn(move || self.do_serve())
    }

    fn do_serve(self) {
        let mut anchor = 0;
        let mut immatures: Vec<LiveCell> = Vec::new();
        let mut pending = None;

        while let Ok(cell) = self.receiver.recv() {
            if anchor + CELLBASE_MATURITY <= cell.block_number {
                println!("[Selector] matured trigger #{}", cell.block_number);
                let mut truncate = immatures.len();
                for (i, immature) in immatures.iter().enumerate() {
                    if immature.block_number + CELLBASE_MATURITY <= cell.block_number {
                        self.maybe_send(&mut pending, immature.clone());
                    } else {
                        truncate = i;
                        break;
                    }
                }

                immatures = immatures[truncate..].to_vec();
                if let Some(first) = immatures.first() {
                    anchor = first.block_number;
                }
            }

            if cell.tx_index == 0 {
                println!(
                    "[Selector] received cellbase cell from #{}",
                    cell.block_number
                );
                immatures.push(cell);
                if let Some(first) = immatures.first() {
                    anchor = first.block_number;
                }
            } else {
                self.maybe_send(&mut pending, cell);
            }
        }
    }

    fn maybe_send(&self, pending: &mut Option<LiveCell>, cell: LiveCell) {
        if let Some(p) = pending.take() {
            let total_capacity = Unpack::<u64>::unpack(&p.cell_output.capacity())
                + Unpack::<u64>::unpack(&cell.cell_output.capacity());
            assert!(total_capacity >= MIN_INPUT_CAPACITY);
            println!(
                "[Selector] selectd 2 cells, total_capacity {}",
                total_capacity
            );
            self.sender.send(vec![p, cell]).unwrap();
        } else {
            let total_capacity = Unpack::<u64>::unpack(&cell.cell_output.capacity());
            if total_capacity < MIN_INPUT_CAPACITY {
                println!("[Selector] selected failed, pending = {}", total_capacity);
                *pending = Some(cell);
            } else {
                println!(
                    "[Selector] selectd 1 cell, total_capacity {}",
                    total_capacity
                );
                self.sender.send(vec![cell]).unwrap();
            }
        }
    }
}
