use crate::utils::constants::{MIN_FEE, MIN_OUTPUT_CAPACITY};
use crate::utils::{LiveCell, User};
use ckb_types::core::{TransactionBuilder, TransactionView};
use ckb_types::packed::{CellInput, CellOutput};
use ckb_types::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use std::thread::{spawn, JoinHandle};

pub struct Constructor {
    user: User,
    receiver: Receiver<Vec<LiveCell>>,
    sender: Sender<TransactionView>,
}

impl Constructor {
    pub fn new(
        user: User,
        receiver: Receiver<Vec<LiveCell>>,
        sender: Sender<TransactionView>,
    ) -> Self {
        Self {
            user,
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
            let inputs: Vec<_> = cells
                .into_iter()
                .map(|cell| CellInput::new(cell.out_point, 0))
                .collect();

            let output_capacities = input_capacities - MIN_FEE;
            let output_count = output_capacities / MIN_OUTPUT_CAPACITY;
            let outputs: Vec<_> = (0..output_count)
                .map(|i| {
                    CellOutput::new_builder()
                        .lock(self.user.lock_script())
                        .capacity({
                            if i > output_capacities % MIN_OUTPUT_CAPACITY {
                                (MIN_OUTPUT_CAPACITY + 1).pack()
                            } else {
                                MIN_OUTPUT_CAPACITY.pack()
                            }
                        })
                        .build()
                })
                .collect();

            let outputs_data: Vec<_> = (0..output_count).map(|_| Default::default()).collect();
            let cell_dep = self.user.cell_dep();
            let raw_transaction = TransactionBuilder::default()
                .inputs(inputs)
                .outputs(outputs)
                .outputs_data(outputs_data)
                .cell_dep(cell_dep)
                .build();

            let _ = self.sender.send(raw_transaction);
        }
    }
}
