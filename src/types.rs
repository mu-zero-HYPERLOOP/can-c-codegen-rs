use can_config_rs::config::{self, Type, TypeRef};

use crate::errors::Result;
use crate::messages::signal_type_to_c_type;
use crate::source_block::{SourceBlock, SourceBlockIdentifier};
use crate::{file_buffer::FileBuffer, options::Options};

pub fn generate_types(
    node_config: &config::NodeRef,
    header: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }

    // in theory they should already be sorted topological, but we should still add proper
    // dependencies!
    for ty in node_config.types() {
        match ty as &config::Type {
            config::Type::Struct {
                name,
                description: _,
                attribs,
                visibility: _,
            } => {
                let mut def = format!("typedef struct {{\n");
                let mut dependencies = vec![];
                for (attrib_name, attrib_type) in attribs {
                    let attrib_type_name = attrib_type.name();
                    let (ctype, deps) = to_c_type_name(attrib_type);
                    def.push_str(&format!("{indent}{ctype} {attrib_name};\n"));
                    
                    match deps {
                        Some(dep) => {
                            if !dependencies.contains(&dep) {
                                dependencies.push(dep);
                            }
                        }
                        None => (),
                    }

                    let dep = SourceBlockIdentifier::Declartion(attrib_type_name);
                    if !dependencies.contains(&dep) {
                        dependencies.push(dep);
                    }
                }
                def.push_str(&format!("}} {name};\n"));
                header.add_block(SourceBlock::new(
                    SourceBlockIdentifier::Declartion(name.clone()),
                    def,
                    dependencies,
                ))?;
            }
            config::Type::Enum {
                name,
                description: _,
                size: _,
                entries,
                visibility: _,
            } => {
                let mut def = format!("typedef enum {{\n");
                for (entry_name, entry_value) in entries {
                    def.push_str(&format!("{indent}{name}_{entry_name} = {entry_value},\n"));
                }
                def.push_str(&format!("}} {name};\n"));
                header.add_block(SourceBlock::new(
                    SourceBlockIdentifier::Declartion(name.clone()),
                    def,
                    vec![],
                ))?;
            }
            config::Type::Array { len: _, ty: _ } => todo!(),
            config::Type::Primitive(_) => {
                panic!("primitives should not be explicitly defined as node types")
            }
        }
    }
    Ok(())
}

pub fn to_c_type_name(ty: &Type) -> (&str, Option<SourceBlockIdentifier>) {
    match ty {
        config::Type::Primitive(signal_type) => signal_type_to_c_type(signal_type),
        config::Type::Struct {
            name,
            description: _,
            attribs: _,
            visibility: _,
        } => (name, Some(SourceBlockIdentifier::Definition(name.clone()))),
        config::Type::Enum {
            name,
            description: _,
            size: _,
            entries: _,
            visibility: _,
        } => (name, Some(SourceBlockIdentifier::Definition(name.clone()))),
        config::Type::Array { len: _, ty: _ } => todo!(),
    }
}
