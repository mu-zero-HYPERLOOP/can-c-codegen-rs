use can_config_rs::config::ObjectEntryRef;

use crate::errors::Result;
use crate::{options::Options, types::to_c_type_name};

pub fn generate_object_entries(
    object_entries: &Vec<ObjectEntryRef>,
    header: &mut String,
    source: &mut String,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }

    for object_entry in object_entries {
        let type_name = to_c_type_name(object_entry.ty());
        let oe_name = object_entry.name();
        let oe_var = format!("__oe_{oe_name}");

        let var_def = format!("{type_name} {oe_var};\n");
        source.push_str(&var_def);

        let getter_name = format!("{namespace}_get_{oe_name}");
        let mut getter_def = format!("static inline {type_name} {getter_name}() {{\n");
        getter_def.push_str(&format!("{indent}extern {type_name} {oe_var};\n"));
        getter_def.push_str(&format!("{indent}return {oe_var};\n"));
        getter_def.push_str("}\n");
        header.push_str(&getter_def);

        let setter_name = format!("{namespace}_set_{oe_name}");
        let mut setter_def = format!("static inline void {setter_name}({type_name} value){{\n");
        setter_def.push_str(&format!("{indent}extern {type_name} {oe_var};\n"));
        setter_def.push_str(&format!("{indent}{oe_var} = value;\n"));
        setter_def.push_str("}\n");
        header.push_str(&setter_def);
    }
    Ok(())
}
