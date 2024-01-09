use can_config_rs::config;
use can_config_rs::config::bus::BusRef;

use crate::source_block::{SourceBlock, SourceBlockIdentifier};
use crate::{file_buffer::FileBuffer, options::Options};

use crate::errors::Result;

pub fn generate_poll(
    node_config: &config::NodeRef,
    buses: &Vec<BusRef>,
    header: &mut FileBuffer,
    source: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    let indent2 = format!("{indent}{indent}");
    let indent3 = format!("{indent2}{indent}");
    let indent4 = format!("{indent2}{indent2}");

    for bus in buses {
        let bus_id = bus.id();
        let poll_func_name = format!("{namespace}_can{bus_id}_poll");
        let poll_func_decl = format!("void {poll_func_name}();\n");

        header.add_block(SourceBlock::new(
            SourceBlockIdentifier::Declartion(poll_func_name.clone()),
            poll_func_decl,
            vec![],
        ))?;

        let dependencies = vec![SourceBlockIdentifier::Declartion(poll_func_name.clone())];

        let mut poll_func_def = format!("void {poll_func_name}() {{\n");
        poll_func_def.push_str(&format!("{indent}{namespace}_frame frame;\n"));
        poll_func_def.push_str(&format!(
            "{indent}while ({namespace}_can{bus_id}_recv(&frame)) {{\n"
        ));
        poll_func_def.push_str(&format!(
            "{indent2}switch ({namespace}_frame_get_id(&frame)) {{\n"
        ));
        for message in node_config.rx_messages() {
            if message.bus().id() != bus_id {
                continue;
            }
            let message_name = message.name();
            // first bit stands for ide bit
            let key = match message.id() {
                config::MessageId::StandardId(id) => id << 1 | 0,
                config::MessageId::ExtendedId(id) => id << 1 | 1,
            };
            poll_func_def.push_str(&format!("{indent3}case 0x{key:X}: \n"));
            poll_func_def.push_str(&format!("{indent3}{{\n"));
            poll_func_def += &format!("{indent3}{namespace}_handle_{message_name}(&frame);\n");
            // poll_func_def.push_str(&format!("{indent4}{message_name} msg;\n"));
            // dependencies.push(SourceBlockIdentifier::Definition(message_name.to_owned()));
            // let deserialize_func_name = format!("deserialize_{message_name}");
            // dependencies.push(SourceBlockIdentifier::Definition(
            //     deserialize_func_name.clone(),
            // ));
            // poll_func_def.push_str(&format!(
            //     "{indent4}{deserialize_func_name}({namespace}_frame_get_data(&frame), &msg);\n"
            // ));
            // poll_func_def.push_str(&format!("{indent4}//TODO handling of frame!\n"));
            //
            poll_func_def.push_str(&format!("{indent4}break;\n"));
            poll_func_def.push_str(&format!("{indent3}}}\n"));
        }
        poll_func_def.push_str(&format!("{indent2}}}\n"));
        poll_func_def.push_str(&format!("{indent}}}\n"));

        poll_func_def.push_str("}\n");
        source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Definition(poll_func_name.clone()),
            poll_func_def,
            dependencies,
        ))?;
    }

    Ok(())
}
