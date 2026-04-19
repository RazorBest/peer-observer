use std::{ptr, thread, time::Duration};

// These constants were copied from src/bpf/tracing.bpf.c
const MAX_PEER_ADDR_LENGTH: usize = 62 + 6;
const MAX_PEER_CONN_TYPE_LENGTH: usize = 20;
const MAX_MSG_TYPE_LENGTH: usize = 12;

#[usdt::provider(provider = "net")]
mod probes_net {
    fn inbound_message(
        id: u64,
        addr: *const u8,
        conn_type: *const u8,
        msg_type: *const u8,
        msg_size: u64,
        msg_payload: *const u8,
    ) {
    }
    fn outbound_message() {}
    fn inbound_connection() {}
    fn outbound_connection() {}
    fn closed_connection() {}
    fn evicted_inbound_connection() {}
    fn misbehaving_connection() {}
}

#[usdt::provider(provider = "mempool")]
mod probes_mempool {
    fn added(txid: *const u8, vsize: i32, fee: i64) {
        println!("Called mempool_added");
    }
    fn removed() {}
    fn replaced() {}
    fn rejected() {}
}

#[usdt::provider(provider = "addrman")]
mod probes_addrman {
    fn attempt_add() {}
    fn move_to_good() {}
}

#[usdt::provider(provider = "validation")]
mod probes_validation {
    fn block_connected() {}
}

fn main() {
    usdt::register_probes().expect("failed to register USDT probes");

    // Trigger the probes, so the compiler doesn't remove them
    probes_net::inbound_message!(|| (0, ptr::null(), ptr::null(), ptr::null(), 0, ptr::null()));
    probes_net::outbound_message!();
    probes_net::inbound_connection!();
    probes_net::outbound_connection!();
    probes_net::closed_connection!();
    probes_net::evicted_inbound_connection!();
    probes_net::misbehaving_connection!();

    // probes_mempool::added!();
    probes_mempool::removed!();
    probes_mempool::replaced!();
    probes_mempool::rejected!();

    probes_addrman::attempt_add!();
    probes_addrman::move_to_good!();

    probes_validation::block_connected!();

    let addr = [0u8; MAX_PEER_ADDR_LENGTH];
    let conn_type = [0u8; MAX_PEER_CONN_TYPE_LENGTH];
    let msg_type = [0u8; MAX_MSG_TYPE_LENGTH];

    // Same as MAX_HUGE_MSG_LENGTH from src/bpf/tracing.bpf.c
    // let payload_size = 4194304;
    let payload_size = 32;
    let mut id = 0u64;

    for _ in 0..100000000 {
        thread::sleep(Duration::from_millis(1000));
        println!("Calling inbound_message");
        probes_net::inbound_message!(|| (id, addr.as_ptr(), conn_type.as_ptr(), msg_type.as_ptr(), payload_size, ptr::null()));
        id += 1;

        probes_net::outbound_message!();
        probes_net::inbound_connection!();
        probes_net::outbound_connection!();
        probes_net::closed_connection!();
        probes_net::evicted_inbound_connection!();
        probes_net::misbehaving_connection!();

        const TXID_LENGTH: usize = 32;
        let txid = [0u8; TXID_LENGTH];

        probes_mempool::added!(|| (txid.as_ptr(), 0, 0));
        probes_mempool::removed!();
        probes_mempool::replaced!();
        probes_mempool::rejected!();

        probes_addrman::attempt_add!();
        probes_addrman::move_to_good!();

        probes_validation::block_connected!();
    }
}
