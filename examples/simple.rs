use std::time::Duration;

use can_c_codegen_rs::options::Options;



fn main() {
    let network_builder = can_config_rs::builder::NetworkBuilder::new();
    let secu = network_builder.create_node("secu");
    secu.create_object_entry("bar", "u32");
    secu.create_object_entry("state", "u8");
    let foo = secu.create_stream("something");
    foo.add_entry("bar");


    let state_stream = secu.create_stream("states");
    state_stream.add_entry("state");
    state_stream.set_interval(Duration::from_millis(5), Duration::from_millis(1000));


    let config = network_builder.build().unwrap();

    let mut options = Options::default();
    options.set_source_file_path("./examples/simple.c");
    options.set_header_file_path("./examples/simple.h");

    can_c_codegen_rs::generate("secu", config, options).unwrap();


}
