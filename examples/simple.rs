use std::time::Duration;

use can_c_codegen_rs::options::Options;



fn main() {
    let network_builder = can_config_rs::builder::NetworkBuilder::new();
    let secu = network_builder.create_node("secu");
    secu.create_object_entry("bar", "u32");
    secu.create_object_entry("state", "u64");
    let foo = secu.create_stream("something");
    foo.set_interval(Duration::from_millis(1000), Duration::from_millis(1000));
    foo.add_entry("bar");


    let state_stream = secu.create_stream("states");
    state_stream.add_entry("state");
    state_stream.set_interval(Duration::from_millis(5), Duration::from_millis(1000));


    let master = network_builder.create_node("master");
    master.create_object_entry("secu_control", "u64");
    let tx_control = master.create_stream("control");
    tx_control.add_entry("secu_control");


    let rx_stream = secu.receive_stream("master", "control");
    rx_stream.map("secu_control", "state");


    let config = network_builder.build().unwrap();

    let mut options = Options::default();
    options.set_source_file_path("./examples/simple.c");
    options.set_header_file_path("./examples/simple.h");

    can_c_codegen_rs::generate("secu", config, options).unwrap();


}
