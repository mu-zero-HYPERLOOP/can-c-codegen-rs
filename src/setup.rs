use can_config_rs::config;

use crate::options::Options;

use crate::errors::Result;



pub fn generate_setup(network_config : &config::NetworkRef, source : &mut String, header : &mut String, options : &Options) -> Result<()>{
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    

    let init_name = format!("{namespace}_init");

    let init_decl = format!("void {init_name}();\n");
    header.push_str(&init_decl);

    let mut setup_cans = String::new();
    for bus in network_config.buses() {
        let bus_id = bus.id();
        let baudrate = bus.baudrate();
        setup_cans.push_str(&format!("{indent}{namespace}_can{bus_id}_setup({baudrate}, NULL, 0);\n"));
    }

    let init_def = format!("void {init_name}() {{
{setup_cans}
{indent}scheduler_init();
{indent}schedule_heartbeat_job();
}}\n");
    source.push_str(&init_def);
    
    Ok(())
}
