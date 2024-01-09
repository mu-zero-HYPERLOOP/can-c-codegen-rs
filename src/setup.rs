use crate::options::Options;

use crate::errors::Result;



pub fn generate_setup(source : &mut String, header : &mut String, options : &Options) -> Result<()>{
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    

    let init_name = format!("{namespace}_init");

    let init_decl = format!("void {init_name}();\n");
    header.push_str(&init_decl);

    let init_def = format!("void {init_name}() {{
{indent}scheduler_init();
{indent}schedule_heartbeat_job();
}}\n");
    source.push_str(&init_def);
    
    Ok(())
}
