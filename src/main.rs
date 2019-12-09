use crate::config::setup;
use crate::utils::{Secp, User};
use crossbeam_channel::bounded;
use std::thread::sleep;
use std::time::Duration;

mod collector;
mod config;
mod constructor;
mod controller;
mod selector;
mod utils;

fn main() {
    let config = attempt(setup(), "setup");
    // let _ = ckb_logger::init(config.logger.clone()).unwrap();
    println!("Init");

    let secp = Secp::init(&config.chain.rpc_urls);
    let alice = User::new(config.alice.clone(), secp);
    let (cell_sender, cell_receiver) = bounded(10000);
    let (inputs_sender, inputs_receiver) = bounded(10000);
    let (raw_tx_sender, raw_tx_receiver) = bounded(10000);

    let collector = collector::Collector::new(alice.clone(), &config.chain.rpc_urls, cell_sender);
    let selector = selector::Selector::new(cell_receiver, inputs_sender);
    let constructor = constructor::Constructor::new(alice.clone(), inputs_receiver, raw_tx_sender);
    let controller = controller::Controller::new(
        alice,
        &config.chain.rpc_urls,
        raw_tx_receiver,
        config.controller.clone(),
    );
    let _ = selector.serve();
    let _ = constructor.serve();
    let _ = controller.serve();
    let _ = collector.serve();
    loop {
        sleep(Duration::from_secs(10));
    }
}

fn attempt<T>(r: Result<T, String>, msg: &str) -> T {
    match r {
        Err(err) => {
            eprintln!("{} {:?}", msg, err);
            exit();
            unreachable!()
        }
        Ok(t) => t,
    }
}

fn exit() {
    ckb_logger::flush();
    std::process::exit(1);
}
