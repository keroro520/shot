use ckb_types::core::BlockNumber;
use ckb_types::packed::{CellOutput, OutPoint};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct LiveCell {
    pub cell_output: CellOutput,
    pub out_point: OutPoint,
    pub tx_index: usize,
    pub block_number: BlockNumber,
}

#[derive(Clone, Debug)]
pub struct Unspent {
    inner: HashMap<OutPoint, LiveCell>,
}

impl Unspent {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn update(&mut self, dead_out_points: &[OutPoint], live_cells: Vec<LiveCell>) {
        for dead in dead_out_points.iter() {
            self.inner.remove(dead);
        }
        for live in live_cells.into_iter() {
            self.inner.insert(live.out_point.clone(), live);
        }
    }

    pub fn into_iter(self) -> impl IntoIterator<Item = (OutPoint, LiveCell)> {
        let mut vec = self.inner.into_iter().collect::<Vec<_>>();
        vec.sort_by(|a, b| a.1.block_number.cmp(&b.1.block_number));
        vec.into_iter()
    }
}
