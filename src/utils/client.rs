use ckb_types::core::HeaderView;
use ckb_types::core::{BlockNumber, BlockView};
use rpc_client::Jsonrpc;
use std::ops::Deref;
use std::thread::sleep;
use std::time::Duration;

pub struct Client {
    ckb_nodes: Vec<Jsonrpc>,
}

impl Deref for Client {
    type Target = Jsonrpc;
    fn deref(&self) -> &Self::Target {
        &self.ckb_nodes[0]
    }
}

impl Client {
    pub fn init(urls: &[String]) -> Self {
        let ckb_nodes: Result<Vec<Jsonrpc>, _> = urls
            .iter()
            .map(|url| Jsonrpc::connect(url.as_str()))
            .collect();
        Self {
            ckb_nodes: ckb_nodes.expect("init Client failed"),
        }
    }

    pub fn get_safe_block(&self, block_number: BlockNumber) -> BlockView {
        loop {
            if let Some(header) = self.get_safe_header(block_number) {
                return self.ckb_nodes[0].get_block(header.hash()).unwrap().into();
            } else {
                sleep(Duration::from_secs(1));
            }
        }
    }

    pub fn get_safe_tip(&self) -> BlockNumber {
        let tip = self.ckb_nodes[0].get_tip_block_number();
        for block_number in (0..=tip).rev() {
            if self.get_safe_header(block_number).is_some() {
                return block_number;
            }
        }
        unreachable!()
    }

    fn get_safe_header(&self, block_number: BlockNumber) -> Option<HeaderView> {
        let safe = if let Some(header) = self.ckb_nodes[0].get_header_by_number(block_number) {
            Into::<HeaderView>::into(header)
        } else {
            return None;
        };
        for jsonrpc in self.ckb_nodes.iter().skip(1) {
            if jsonrpc
                .get_header_by_number(block_number)
                .map(Into::<HeaderView>::into)
                .map(|header: HeaderView| header.hash() == safe.hash())
                .unwrap_or(false)
            {
                return None;
            }
        }
        Some(safe)
    }
}
