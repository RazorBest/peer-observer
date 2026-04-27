use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Generate Rust types for the protobuf's
    if let Err(e) = prost_build::Config::new()
        .compile_well_known_types()
        .type_attribute("Transaction", "use crate::proptest::prelude::*; use crate::proptest::strategy::Strategy;")
        .type_attribute("log", "use crate::proptest::prelude::*; use crate::proptest::strategy::Strategy;")
        .type_attribute("p2p", "use crate::proptest::prelude::*; use crate::proptest::strategy::Strategy;")
        .type_attribute("rpc", "use crate::proptest::prelude::*; use crate::proptest::strategy::Strategy;")
        .type_attribute("ebpf", "use crate::proptest::prelude::*; use crate::proptest::strategy::Strategy;")
        .type_attribute("ebpf_extractor.addrman.AddrmanEvent", "use crate::proptest::prelude::*; use crate::proptest::strategy::Strategy;")
        .type_attribute("ebpf_extractor.connection.ConnectionEvent", "use crate::proptest::prelude::*; use crate::proptest::strategy::Strategy;")
        .type_attribute("ebpf_extractor.mempool.MempoolEvent", "use crate::proptest::prelude::*; use crate::proptest::strategy::Strategy;")
        .type_attribute("ebpf_extractor.message.Metadata", "use crate::proptest::prelude::*; use crate::proptest::strategy::Strategy;")
        .type_attribute(".", "#[derive(proptest_derive::Arbitrary, serde::Serialize, serde::Deserialize)]")
        .field_attribute(
            "Metadata.addr",
            "#[proptest(strategy = \"(0u8..=255, 0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(a,b,c,d)| format!(\\\"{}.{}.{}.{}\\\", a,b,c,d))\")]",
        )
        .field_attribute(
            "Connection.addr",
            "#[proptest(strategy = \"(0u8..=255, 0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(a,b,c,d)| format!(\\\"{}.{}.{}.{}\\\", a,b,c,d))\")]",
        )
        .field_attribute(
            "ebpf.ebpf_event",
            "#[proptest(strategy = \"any::<ebpf::EbpfEvent>().prop_map(Some)\")]",
        )
        .field_attribute(
            "rpc.rpc_event",
            "#[proptest(strategy = \"any::<rpc::RpcEvent>().prop_map(Some)\")]",
        )
        .field_attribute(
            "ChainTxStats.window_tx_count",
            "#[proptest(strategy = \"any::<u32>().prop_map(Some)\")]",
        )
        .field_attribute(
            "ChainTxStats.window_interval",
            "#[proptest(strategy = \"any::<u32>().prop_map(Some)\")]",
        )
        .field_attribute(
            "ChainTxStats.tx_rate",
            "#[proptest(strategy = \"any::<f64>().prop_map(Some)\")]",
        )
        .field_attribute(
            "AddrmanEntry.mapped_as",
            "#[proptest(strategy = \"any::<u32>().prop_map(Some)\")]",
        )
        .field_attribute(
            "AddrmanEntry.source_mapped_as",
            "#[proptest(strategy = \"any::<u32>().prop_map(Some)\")]",
        )
        .field_attribute(
            "p2p.p2p_event",
            "#[proptest(strategy = \"any::<p2p::P2pEvent>().prop_map(Some)\")]",
        )
        .field_attribute(
            "log.log_event",
            "#[proptest(strategy = \"any::<log::LogEvent>().prop_map(Some)\")]",
        )
        .field_attribute(
            "MempoolEvent.event",
            "#[proptest(strategy = \"any::<mempool_event::Event>().prop_map(Some)\")]",
        )
        .field_attribute(
            "MessageEvent.msg",
            "#[proptest(strategy = \"any::<message_event::Msg>().prop_map(Some)\")]",
        )
        .field_attribute(
            "AddrmanEvent.event",
            "#[proptest(strategy = \"any::<addrman_event::Event>().prop_map(Some)\")]",
        )
        .field_attribute(
            "ConnectionEvent.event",
            "#[proptest(strategy = \"any::<connection_event::Event>().prop_map(Some)\")]",
        )
        .field_attribute(
            "bitcoin_primitives.Address.address",
            "#[proptest(strategy = \"any::<address::Address>().prop_map(Some)\")]",
        )
        .field_attribute(
            "bitcoin_primitives.InvetoryItem.item",
            "#[proptest(strategy = \"any::<InvetoryItem::Item>().prop_map(Some)\")]",
        )
        .compile_protos(&["../protobuf/event.proto"], &["../protobuf/"])
    {
        println!("Error while compiling protos: {}", e);
        panic!("Failed to code-gen the Rust structs from the Protobuf definitions");
    }
    println!("cargo:rerun-if-changed=../protobuf/");

    // Generate check functions for IP addresses
    gen_ip_match_fn(
        "ip-lists/torbulkexitlist",
        "torexitlist.rs",
        "is_tor_exit_node",
    );
    gen_ip_match_fn(
        "ip-lists/gmaxbanlist.txt",
        "gmaxbanlist.rs",
        "is_on_gmax_banlist",
    );
    gen_ip_match_fn(
        "ip-lists/monerobanlist.txt",
        "monerobanlist.rs",
        "is_on_monero_banlist",
    );
    gen_ip_match_fn(
        "ip-lists/bitprojects-io.txt",
        "bitprojects-list.rs",
        "belongs_to_bitprojects",
    );

    println!("cargo:rerun-if-changed=build.rs");
}

fn generate_ip_match(addr: &str) -> String {
    format!(" \"{}\" ", addr)
}

// generates a 'rs_file' with a 'match_fn_name' IP match function from a 'input' list
// of IP addresses
fn gen_ip_match_fn(input: &str, rs_file: &str, match_fn_name: &str) {
    let input_list = fs::read_to_string(input).expect("a valid input file path");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join(rs_file);

    let match_statement = input_list
        .lines()
        .map(generate_ip_match)
        .collect::<Vec<String>>()
        .join("|");

    fs::write(
        &out_path,
        format!(
            "\
// DON'T CHANGE THIS FILE MANUALLY. IT WILL BE OVERWRITTEN.
// This file is generated in `build.rs`.

#[allow(dead_code)]
pub fn {}(ip: &str) -> bool {{
    matches!(ip, {})
}}

",
            match_fn_name, match_statement
        ),
    )
    .expect("can write to file");
    println!("cargo:rerun-if-changed={}", input);
}
