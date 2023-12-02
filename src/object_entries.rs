use can_config_rs::config::ObjectEntryRef;

use crate::{file_buffer::FileBuffer, source_block::{SourceBlockIdentifier, SourceBlock}, types::to_c_type_name, options::Options};
use crate::errors::Result;


pub fn generate_object_entries(object_entries : &Vec<ObjectEntryRef>, header : &mut FileBuffer, source : &mut FileBuffer, options : &Options) -> Result<()>{
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    
    for object_entry in object_entries {
        let (type_name, dep) = to_c_type_name(object_entry.ty());
        let var_dependencies = match dep {
            Some(dep) => vec![dep],
            None => vec![],
        };
        let oe_name = object_entry.name();
        let oe_var = format!("__oe_{oe_name}");

        let var_def = format!("{type_name} {oe_var};\n");
        source.add_block(SourceBlock::new(SourceBlockIdentifier::Definition(oe_var.clone()), 
                                          var_def, var_dependencies.clone()))?;
        
        let getter_name = format!("{namespace}_get_{oe_name}");
        let mut getter_def = format!("inline {type_name} {getter_name}() {{\n");
        getter_def.push_str(&format!("{indent}extern {type_name} {oe_var};\n"));
        getter_def.push_str(&format!("{indent}return {oe_var};\n"));
        getter_def.push_str("}\n");


        header.add_block(SourceBlock::new(SourceBlockIdentifier::Definition(getter_name.clone()), 
                                          getter_def, var_dependencies.clone()))?;

        let setter_name = format!("{namespace}_set_{oe_name}");
        let mut setter_def = format!("inline void {setter_name}({type_name} value){{\n");
        setter_def.push_str(&format!("{indent}extern {type_name} {oe_var};\n"));
        setter_def.push_str(&format!("{indent}{oe_var} = value;\n"));
        //TODO set dirty flag for this object entry
        setter_def.push_str("}\n");
        header.add_block(SourceBlock::new(SourceBlockIdentifier::Definition(setter_name.clone()),
            setter_def, var_dependencies))?;

    }
    Ok(())
}
