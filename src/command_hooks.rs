use crate::{
    errors::Result,
    file_buffer::FileBuffer,
    options::Options,
    source_block::{SourceBlock, SourceBlockIdentifier},
};
use can_config_rs::config::{self, Type};

pub fn generate_command_hooks(
    commands: &Vec<config::CommandRef>,
    source: &mut FileBuffer,
    header: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }

    for command in commands {
        let command_name = command.name();
        let req_msg = command.tx_message();
        let Some(encoding) = req_msg.encoding() else {
            panic!("command request messages require a type encoding");
        };
        let mut attribute_list = String::new();
        let mut first = true;
        for attrib in encoding.attributes() {
            if first {
                first = false;
            } else {
                attribute_list += ", ";
            }
            fn ty_to_c_ty(ty: &Type) -> String {
                match ty as &Type {
                    config::Type::Primitive(prim) => match prim {
                        config::SignalType::UnsignedInt { size } => {
                            let s = (2 as u64).pow((*size as f64).log2().ceil().max(3.0) as u32);
                            format!("uint{s}_t")
                        }
                        config::SignalType::SignedInt { size } => {
                            let s = (2 as u64).pow((*size as f64).log2().ceil().max(3.0) as u32);
                            format!("int{s}_t")
                        }
                        config::SignalType::Decimal {
                            size,
                            offset: _,
                            scale: _,
                        } => {
                            let s = (2 as u64).pow((*size as f64).log2().ceil().max(3.0) as u32);
                            if s <= 32 {
                                "float".to_owned()
                            } else {
                                "double".to_owned()
                            }
                        }
                    },
                    config::Type::Struct {
                        name,
                        description: _,
                        attribs: _,
                        visibility: _,
                    } => name.clone(),
                    config::Type::Enum {
                        name,
                        description: _,
                        size: _,
                        entries: _,
                        visibility: _,
                    } => name.clone(),
                    config::Type::Array { len: _, ty: _ } => todo!(),
                }
            }
            let name = attrib.name();
            let ty = ty_to_c_ty(attrib.ty());
            attribute_list += &format!("{ty} {name}");
        }

        let hook_name = format!("{namespace}_{command_name}");
        let weak_hook_def = format!(
            "__attribute__((weak)) command_resp_erno {hook_name}({attribute_list}) {{
{indent}// OVERWRITE ME IN A DIFFERENT COMPILATION UNIT
{indent} return command_resp_erno_Error;
}}\n"
        );
        source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Definition(hook_name.clone()),
            weak_hook_def,
            vec![], // <-- not totally true =^)
        ))?;
    }

    Ok(())
}
