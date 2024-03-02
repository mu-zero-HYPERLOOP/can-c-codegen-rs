use can_config_rs::config;

use crate::options::Options;

use crate::errors::Result;



pub fn generate_setup(node_config : &config::NodeRef, network_config : &config::NetworkRef, source : &mut String, header : &mut String, options : &Options) -> Result<()>{
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
        let bus_name = bus.name();
        let baudrate = bus.baudrate();
        setup_cans.push_str(&format!("{indent}{namespace}_{bus_name}_setup({baudrate}, NULL, 0);\n"));
    }

    let mut schedule_stream_jobs_logic = String::new();
    for tx_stream in node_config.tx_streams() {
        let stream_name = tx_stream.name();
        schedule_stream_jobs_logic.push_str(&format!("{indent}schedule_{stream_name}_interval_job();\n"));
    }

    let init_def = format!("void {init_name}() {{
{setup_cans}
{indent}job_pool_allocator_init();
{indent}scheduler.size = 0;
{indent}schedule_heartbeat_job();
{schedule_stream_jobs_logic}
}}\n");
    source.push_str(&init_def);
    
    Ok(())
}
