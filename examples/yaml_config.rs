extern crate can_cpp_codegen_rs;


fn main() {
    println!("Hello, World");

    // load config
    let network_config = can_yaml_config_rs::parse_yaml_config_from_file("test.yaml").unwrap();

    let options = can_cpp_codegen_rs::options::Options::default();

    can_cpp_codegen_rs::generate("secu", network_config, options).unwrap();
}
