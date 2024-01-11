use can_c_codegen_rs::options::Platform;

fn main() {
    let command = clap::Command::new("cli")
        .arg(clap::Arg::new("config").required(true))
        .arg(clap::Arg::new("node_name").required(true))
        .arg(clap::Arg::new("src_dir").required(true));

    let matches = command.get_matches();
    let config_path: &String = matches.get_one("config").unwrap();
    let node_name : &String = matches.get_one("node_name").unwrap();
    let path : &String = matches.get_one("src_dir").unwrap();

    let mut options = can_c_codegen_rs::options::Options::default();
    options.set_source_file_path(&format!("{path}/canzero.c"));
    options.set_header_file_path(&format!("{path}/canzero.h"));
    options.set_platform(Platform::Linux);
    options.set_namespace("canzero");

    let network_config = can_yaml_config_rs::parse_yaml_config_from_file(config_path)
        .expect("failed to parse network_config");

    can_c_codegen_rs::generate(node_name, network_config, options).expect("failed to generate code");
}
