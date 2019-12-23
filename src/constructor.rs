use crate::config::ConstructorConfig;
use crate::utils::constants::MIN_OUTPUT_CAPACITY;
use crate::utils::{LiveCell, User};
use ckb_types::core::{TransactionBuilder, TransactionView};
use ckb_types::packed::{CellInput, CellOutput};
use ckb_types::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use std::thread::{spawn, JoinHandle};

pub struct Constructor {
    user: User,
    config: ConstructorConfig,
    receiver: Receiver<Vec<LiveCell>>,
    sender: Sender<TransactionView>,
}

impl Constructor {
    pub fn new(
        user: User,
        config: ConstructorConfig,
        receiver: Receiver<Vec<LiveCell>>,
        sender: Sender<TransactionView>,
    ) -> Self {
        Self {
            user,
            config,
            receiver,
            sender,
        }
    }

    pub fn serve(self) -> JoinHandle<()> {
        spawn(move || self.do_serve())
    }

    fn do_serve(self) {
        while let Ok(cells) = self.receiver.recv() {
            let input_capacities = cells
                .iter()
                .map(|cell| Unpack::<u64>::unpack(&cell.cell_output.capacity()))
                .sum::<u64>();
            let raw_transaction = self.construct_without_fee(input_capacities, cells);
            let raw_transaction = self.construct_with_fee(input_capacities, raw_transaction);
            println!("[Constructor] construct 1 transaction");
            self.sender.send(raw_transaction).unwrap();
        }
    }

    fn construct_without_fee(
        &self,
        input_capacities: u64,
        cells: Vec<LiveCell>,
    ) -> TransactionView {
        let inputs: Vec<_> = cells
            .into_iter()
            .map(|cell| CellInput::new(cell.out_point, 0))
            .collect();
        let outputs_count = input_capacities / MIN_OUTPUT_CAPACITY;
        let outputs: Vec<_> = (0..outputs_count)
            .map(|_| {
                CellOutput::new_builder()
                    .lock(self.user.lock_script())
                    .build()
            })
            .collect();
        let outputs_data: Vec<_> = (0..outputs_count).map(|_| Default::default()).collect();
        let cell_dep = self.user.cell_dep();
        TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data)
            .cell_dep(cell_dep)
            .build()
    }

    fn construct_with_fee(
        &self,
        input_capacities: u64,
        transaction: TransactionView,
    ) -> TransactionView {
        let mut outputs: Vec<_> = transaction.outputs().into_iter().collect();;
        let outputs_count = transaction.outputs().len() as u64;
        let size = transaction.data().serialized_size_in_block() as u64 + 100; // 100 is an estimate value of signed witness
        let fee = size * self.config.fee_rate / 1000; // 1000 = 1KB
        if input_capacities < outputs_count * MIN_OUTPUT_CAPACITY + fee {
            let mut outputs_data: Vec<_> = transaction.outputs_data().into_iter().collect();
            outputs.pop().unwrap();
            outputs_data.pop().unwrap();
            let transaction = transaction
                .as_advanced_builder()
                .set_outputs(outputs)
                .set_outputs_data(outputs_data)
                .build();
            return self.construct_with_fee(input_capacities, transaction);
        }

        let outputs_capacity = input_capacities - fee;
        let outputs: Vec<_> = outputs
            .into_iter()
            .enumerate()
            .map(|(i, output)| {
                output
                    .as_builder()
                    .capacity({
                        if (i as u64) < outputs_capacity % outputs_count {
                            (outputs_capacity / outputs_count + 1).pack()
                        } else {
                            (outputs_capacity / outputs_count).pack()
                        }
                    })
                    .build()
            })
            .collect();
        transaction
            .as_advanced_builder()
            .set_outputs(outputs)
            .build()
    }
}
