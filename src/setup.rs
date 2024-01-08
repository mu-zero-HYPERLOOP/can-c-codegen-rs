use crate::source_block::{SourceBlockIdentifier, SourceBlock};
use crate::{file_buffer::FileBuffer, options::Options};

use crate::errors::Result;



pub fn generate_setup(header : &mut FileBuffer, source : &mut FileBuffer, options : &Options) -> Result<()>{
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    

    let setup_name = format!("{namespace}_setup");

    let setup_decl = format!("void {setup_name}();");

    header.add_block(SourceBlock::new(
            SourceBlockIdentifier::Declartion(setup_name.clone()),
            setup_decl,
            vec![]))?;

    let mut setup_def = format!("void {setup_name}() {{\n");
    setup_def.push_str(&format!("{indent}//TODO\n"));
    setup_def.push_str("}\n");

    source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Definition(setup_name.clone()),
            setup_def,
            vec![SourceBlockIdentifier::Declartion(setup_name.clone())]))?;

    
    Ok(())
}
