use can_config_rs::config::{self, message, stream::StreamRef, Type, TypeSignalEncoding};

use crate::{
    errors::Result,
    file_buffer::FileBuffer,
    options::Options,
    source_block::{SourceBlock, SourceBlockIdentifier},
};

pub fn generate_rx_handlers(
    network_config: &config::NetworkRef,
    node_config: &config::NodeRef,
    source: &mut FileBuffer,
    _header: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    let indent2 = format!("{indent}{indent}");
    let indent3 = format!("{indent2}{indent}");

    let frame_type_name = format!("{namespace}_frame");
    for message in node_config.rx_messages() {
        let msg_name = message.name();
        let handler_name = format!("{namespace}_handle_{msg_name}");

        let (logic, weak) = match message.usage() {
            message::MessageUsage::Stream(stream) => {
                let Some(encoding) = message.encoding() else {
                    panic!("stream message requires a type encoding");
                };
                let mut logic = String::new();
                for (encoding, object_entry_mapping) in
                    std::iter::zip(encoding.attributes().iter(), stream.mapping().iter())
                {
                    let Some(object_entry_mapping) = object_entry_mapping else {
                        continue;
                    };
                    let object_entry_name = object_entry_mapping.name();
                    let object_entry_var = format!("__oe_{object_entry_name}");

                    let msg_attribute = encoding.name();

                    logic += &format!("{indent}{object_entry_var} = msg.{msg_attribute};\n");
                }
                (logic, false)
            }
            message::MessageUsage::CommandReq(command) => {
                let req_msg = command.tx_message();
                let Some(encoding) = req_msg.encoding() else {
                    panic!("command request messgages require a type format");
                };
                let mut attribute_list = String::new();
                let mut first = true;
                for attrib in encoding.attributes() {
                    if first {
                        first = false;
                    } else {
                        attribute_list += ", ";
                    }
                    let name = attrib.name();
                    attribute_list += &format!("msg.{name}");
                }
                let resp_msg = command.rx_message();
                let resp_msg_name = resp_msg.name();
                let resp_msg_dlc = resp_msg.dlc();
                let resp_msg_id = match resp_msg.id() {
                    config::MessageId::StandardId(id) => format!("{id}"),
                    config::MessageId::ExtendedId(id) => {
                        format!("({id} | {namespace}_FRAME_IDE_BIT)")
                    }
                };
                let resp_bus_id = resp_msg.bus().id();

                let command_name = command.name();
                (
                    format!(
                        "{indent}{resp_msg_name} resp;
{indent}resp.erno = {namespace}_{command_name}({attribute_list});
{indent}{frame_type_name} resp_frame;
{indent}serialize_{resp_msg_name}(&resp, resp_frame.data);
{indent}resp_frame.dlc = {resp_msg_dlc};
{indent}resp_frame.id = {resp_msg_id};
{indent}{namespace}_can{resp_bus_id}_send(&resp_frame);
"
                    ),
                    false,
                )
            }
            message::MessageUsage::CommandResp(_) => todo!(),
            message::MessageUsage::GetResp => ("".to_owned(), false),
            message::MessageUsage::GetReq => {
                let mut logic = String::new();
                let resp = network_config.get_resp_message();
                let resp_dlc = resp.dlc();
                let resp_id = match resp.id() {
                    config::MessageId::StandardId(id) => format!("{id}"),
                    config::MessageId::ExtendedId(id) => {
                        format!("({id} | {namespace}_FRAME_IDE_BIT)")
                    }
                };
                let resp_bus_id = resp.bus().id();
                let mut case_logic = format!("{indent}switch (msg.header.od_index) {{\n");
                for object_entry in node_config.object_entries() {
                    let name = object_entry.name();
                    let var = format!("__oe_{name}");
                    let size = ty_size(object_entry.ty());
                    let id = object_entry.id();
                    if size <= 32 {
                        let oe_value: String = match object_entry.ty() as &Type {
                            Type::Primitive(signal_type) => match signal_type {
                                config::SignalType::UnsignedInt { size } => {
                                    if *size <= 8 {
                                        format!("{indent2}resp.data = (uint32_t)({var} & (0xFF >> (8 - {size})));\n")
                                    } else if *size <= 16 {
                                        format!("{indent2}resp.data = (uint32_t)({var} & (0xFFFF >> (16 - {size});\n")
                                    } else if *size <= 32 {
                                        format!("{indent2}resp.data = {var} & (0xFFFFFFFF >> (32 - {size};\n")
                                    } else if *size <= 64 {
                                        panic!("values larger than 32 should be send in fragmented mode")
                                    } else {
                                        panic!("unsigned integer larger than 64 are not supported");
                                    }
                                }
                                config::SignalType::SignedInt { size } => {
                                    if *size <= 8 {
                                        format!("{indent2}resp.data = (uint32_t)(((uint8_t){var}) & (0xFF >> (8 - {size})));\n")
                                    } else if *size <= 16 {
                                        format!("{indent2}resp.data = (uint32_t)(((uint16_t){var}) & (0xFFFF >> (16 - {size});\n")
                                    } else if *size <= 32 {
                                        format!("{indent2}resp.data = (uint32_t)(((uint32_t){var}) & (0xFFFFFFFF >> (32 - {size});\n")
                                    } else if *size <= 64 {
                                        panic!("values larger than 32 should be send in fragmented mode")
                                    } else {
                                        panic!("signed integer larger than 64 are not supported");
                                    }
                                }
                                config::SignalType::Decimal {
                                    size,
                                    offset,
                                    scale,
                                } => {
                                    if *size <= 32 {
                                        format!("{indent2}resp.data = (uint32_t)(({var} - ({offset})) / {scale});\n")
                                    } else if *size <= 64 {
                                        panic!("values larger than 32 should be send in fragmented mode")
                                    } else {
                                        panic!("decimals larger than 64 are not supported");
                                    }
                                }
                            },
                            Type::Struct {
                                name: _,
                                description: _,
                                attribs: _,
                                visibility: _,
                            } => {
                                todo!()
                            }
                            Type::Enum {
                                name: _,
                                description: _,
                                size: _,
                                entries: _,
                                visibility: _,
                            } => {
                                if size <= 8 {
                                    format!("{indent2}resp.data = (uint32_t)(((uint8_t){var}) & (0xFF >> (8 - {size})));\n")
                                } else if size <= 16 {
                                    format!("{indent2}resp.data = (uint32_t)(((uint16_t){var}) & (0xFFFF >> (16 - {size});\n")
                                } else if size <= 32 {
                                    format!("{indent2}resp.data = ((uint32_t){var}) & (0xFFFFFFFF >> (32 - {size};\n")
                                } else if size <= 64 {
                                    panic!(
                                        "values larger than 32 should be send in fragmented mode"
                                    )
                                } else {
                                    panic!("unsigned integer larger than 64 are not supported");
                                }
                            }
                            Type::Array { len: _, ty: _ } => todo!(),
                        };

                        case_logic += &format!(
                            "{indent}case {id}: {{
{oe_value}{indent2}resp.header.sof = 1;
{indent2}resp.header.eof = 1;
{indent2}resp.header.toggle = 1;
{indent2}break;
{indent}}}\n"
                        );
                    } else {
                        let buffer_name = format!("{var}_rx_fragmentation_buffer");
                        let buffer_offset = format!("{var}_rx_fragmentation_offset");
                        let buffer_def = format!(
                            "static uint32_t {buffer_name}[{}];
static uint32_t {buffer_offset} = 0;\n",
                            size.div_ceil(32)
                        );
                        source.add_block(SourceBlock::new(
                            SourceBlockIdentifier::Definition(buffer_name.clone()),
                            buffer_def,
                            vec![SourceBlockIdentifier::Import("inttypes.h".to_owned())],
                        ))?;

                        let mut fragmentation_logic = String::new();
                        fn generate_fragmentation_logic(
                            logic: &mut String,
                            ty: &Type,
                            var: &str,
                            buffer: &str,
                            bit_offset: &mut usize,
                            indent2 : &str,
                            indent3 : &str,
                        ) {
                            match ty {
                                Type::Primitive(signal_type) => {
                                    let val = match signal_type {
                                        config::SignalType::UnsignedInt { size: _ } => {
                                            var.to_owned()
                                        }
                                        config::SignalType::SignedInt { size } => {
                                            if *size <= 8 {
                                                format!("((uint8_t){var})")
                                            } else if *size <= 16 {
                                                format!("((uint16_t){var})")
                                            } else if *size <= 32 {
                                                format!("((uint32_t){var})")
                                            } else if *size <= 64 {
                                                format!("((uint64_t){var})")
                                            } else {
                                                panic!("singed integer larger than 64 are not supported");
                                            }
                                        }
                                        config::SignalType::Decimal {
                                            size,
                                            offset,
                                            scale,
                                        } => {
                                            if *size <= 8 {
                                                format!("((uint8_t)(({var} - ((float){offset})) / (float){scale}))")
                                            } else if *size <= 16 {
                                                format!("((uint16_t)(({var} - ((float){offset})) / (float){scale}))")
                                            } else if *size <= 32 {
                                                format!("((uint32_t)(({var} - ((float){offset})) / (float){scale}))")
                                            } else if *size <= 64 {
                                                format!("((uint64_t)(({var} - ((double){offset})) / (double){scale}))")
                                            } else {
                                                panic!("singed integer larger than 64 are not supported");
                                            }
                                        }
                                    };
                                    let size = signal_type.size() as usize;
                                    let val = if size <= 8 {
                                        format!("({val} & (0xFF >> (8 - {size})))")
                                    } else if size <= 16 {
                                        format!("({val} & (0xFFFF >> (16 - {size})))")
                                    } else if size <= 32 {
                                        format!("({val} & (0xFFFFFFFF >> (32 - {size})))")
                                    } else if size <= 64 {
                                        format!("({val} & (0xFFFFFFFFFFFFFFFF >> (64 - {size})))")
                                    } else {
                                        panic!(
                                            "primitive data types larger than 64 are not supported"
                                        )
                                    };
                                    if size <= 32 {
                                        if *bit_offset % 32 == 0 {
                                            let word_offset = *bit_offset / 32;
                                            logic.push_str(&format!(
                                                "{indent2}{buffer}[{word_offset}] = {val};\n"
                                            ));
                                        } else if (*bit_offset % 32) + size >= 32 {
                                            let word_offset = *bit_offset / 32;
                                            let shift = (*bit_offset + size) % 32;
                                            logic.push_str(&format!(
                                                "{indent2}{buffer}[{word_offset}] |= ({val} << {shift});\n"
                                            ));
                                        } else {
                                            let lower_word_offset = *bit_offset / 32;
                                            let lower_shift = *bit_offset % 32;
                                            logic.push_str(&format!("{indent2}{buffer}[{lower_word_offset}] |= ({val} << {lower_shift});\n"));
                                            let upper_word_offset = lower_word_offset + 1;
                                            let upper_shift = 32 - (*bit_offset + size) % 32;
                                            logic.push_str(&format!("{indent2}{buffer}[{upper_word_offset}] = ({val} >> {upper_shift});\n"));
                                        }
                                    } else if size <= 64 {
                                        logic.push_str(&format!("{indent2}{{\n"));
                                        logic.push_str(&format!("{indent3}uint64_t masked = {val};\n"));
                                        if *bit_offset % 32 == 0 {
                                            let lower_word_offset = *bit_offset / 32;
                                            let upper_word_offset = lower_word_offset + 1;
                                            logic.push_str(&format!("{indent3}{buffer}[{lower_word_offset}] = ((uint32_t*)&masked)[0];\n"));
                                            logic.push_str(&format!("{indent3}{buffer}[{upper_word_offset}] = ((uint32_t*)&masked)[1];\n"));
                                        } else if (*bit_offset % 32) + size >= 64 {
                                            let lower_word_offset = *bit_offset / 32;
                                            let lower_shift_left = *bit_offset % 32;
                                            logic.push_str(&format!("{indent3}{buffer}[{lower_word_offset}] |= ((uint32_t*)&masked)[0] << {lower_shift_left});\n"));
                                            let upper_word_offset = lower_word_offset + 1;
                                            let lower_shift_right = 32 - *bit_offset % 32;
                                            logic.push_str(&format!("{indent3}{buffer}[{upper_word_offset}] = ((uint32_t*)&masked)[0] >> {lower_shift_right});\n"));
                                            let upper_shift_left = lower_shift_right;
                                            logic.push_str(&format!("{indent3}{buffer}[{upper_word_offset}] |= ((uint32_t*)&masked)[1] << {upper_shift_left});\n"));
                                        } else {
                                            let lower_word_offset = *bit_offset / 32;
                                            let lower_shift_left = *bit_offset % 32;
                                            logic.push_str(&format!("{indent3}{buffer}[{lower_word_offset}] |= ((uint32_t*)&masked)[0] << {lower_shift_left});\n"));
                                            let middle_word_offset = lower_word_offset + 1;
                                            let lower_shift_right = 32 - *bit_offset % 32;
                                            logic.push_str(&format!("{indent3}{buffer}[{middle_word_offset}] = ((uint32_t*)&masked)[0] >> {lower_shift_right});\n"));
                                            let upper_shift_left = lower_shift_right;
                                            logic.push_str(&format!("{indent3}{buffer}[{middle_word_offset}] |= ((uint32_t*)&masked)[1] << {upper_shift_left});\n"));
                                            let upper_word_offset = middle_word_offset + 1;
                                            let upper_shift_right =
                                                32 - (*bit_offset % 32 + size) % 64;
                                            logic.push_str(&format!("{indent3}{buffer}[{upper_word_offset}] = ((uint32_t*)&masked)[1] >> {upper_shift_right});\n"));
                                        }
                                        logic.push_str(&format!("{indent2}}}"));
                                        *bit_offset += size;
                                    } else {
                                        panic!(
                                            "primitive data types larger than 64 are not supported"
                                        );
                                    }
                                }
                                Type::Enum {
                                    name: _,
                                    description: _,
                                    size,
                                    entries: _,
                                    visibility: _,
                                } => {
                                    let size = *size as usize;
                                    let val = if size <= 8 {
                                        format!("({var} & (0xFF >> (8 - {size})))")
                                    } else if size <= 16 {
                                        format!("({var} & (0xFFFF >> (16 - {size})))")
                                    } else if size <= 32 {
                                        format!("({var} & (0xFFFFFFFF >> (32 - {size})))")
                                    } else if size <= 64 {
                                        format!("({var} & (0xFFFFFFFFFFFFFFFF >> (64 - {size})))")
                                    } else {
                                        panic!("enum data types larger than 64 are not supported")
                                    };
                                    if size <= 32 {
                                        if *bit_offset % 32 == 0 {
                                            let word_offset = *bit_offset / 32;
                                            logic.push_str(&format!(
                                                "{buffer}[{word_offset}] = {val};\n"
                                            ));
                                        } else if (*bit_offset % 32) + size >= 32 {
                                            let word_offset = *bit_offset / 32;
                                            let shift = (*bit_offset + size) % 32;
                                            logic.push_str(&format!(
                                                "{buffer}[{word_offset}] |= ({val} << {shift});\n"
                                            ));
                                        } else {
                                            let lower_word_offset = *bit_offset / 32;
                                            let lower_shift = *bit_offset % 32;
                                            logic.push_str(&format!("{buffer}[{lower_word_offset}] |= ({val} << {lower_shift});\n"));
                                            let upper_word_offset = lower_word_offset + 1;
                                            let upper_shift = 32 - (*bit_offset + size) % 32;
                                            logic.push_str(&format!("{buffer}[{upper_word_offset}] = ({val} >> {upper_shift});\n"));
                                        }
                                    } else if size <= 64 {
                                        logic.push_str(&format!("{{"));
                                        logic.push_str(&format!("uint64_t masked = {val}"));
                                        if *bit_offset % 32 == 0 {
                                            let lower_word_offset = *bit_offset / 32;
                                            let upper_word_offset = lower_word_offset + 1;
                                            logic.push_str(&format!("{buffer}[{lower_word_offset}] = ((uint32_t*)&masked)[0];\n"));
                                            logic.push_str(&format!("{buffer}[{upper_word_offset}] = ((uint32_t*)&masked)[1];\n"));
                                        } else if (*bit_offset % 32) + size >= 64 {
                                            let lower_word_offset = *bit_offset / 32;
                                            let lower_shift_left = *bit_offset % 32;
                                            logic.push_str(&format!("{buffer}[{lower_word_offset}] |= ((uint32_t*)&masked)[0] << {lower_shift_left});\n"));
                                            let upper_word_offset = lower_word_offset + 1;
                                            let lower_shift_right = 32 - *bit_offset % 32;
                                            logic.push_str(&format!("{buffer}[{upper_word_offset}] = ((uint32_t*)&masked)[0] >> {lower_shift_right});\n"));
                                            let upper_shift_left = lower_shift_right;
                                            logic.push_str(&format!("{buffer}[{upper_word_offset}] |= ((uint32_t*)&masked)[1] << {upper_shift_left});\n"));
                                        } else {
                                            let lower_word_offset = *bit_offset / 32;
                                            let lower_shift_left = *bit_offset % 32;
                                            logic.push_str(&format!("{buffer}[{lower_word_offset}] |= ((uint32_t*)&masked)[0] << {lower_shift_left});\n"));
                                            let middle_word_offset = lower_word_offset + 1;
                                            let lower_shift_right = 32 - *bit_offset % 32;
                                            logic.push_str(&format!("{buffer}[{middle_word_offset}] = ((uint32_t*)&masked)[0] >> {lower_shift_right});\n"));
                                            let upper_shift_left = lower_shift_right;
                                            logic.push_str(&format!("{buffer}[{middle_word_offset}] |= ((uint32_t*)&masked)[1] << {upper_shift_left});\n"));
                                            let upper_word_offset = middle_word_offset + 1;
                                            let upper_shift_right =
                                                32 - (*bit_offset % 32 + size) % 64;
                                            logic.push_str(&format!("{buffer}[{upper_word_offset}] = ((uint32_t*)&masked)[1] >> {upper_shift_right});\n"));
                                        }
                                        logic.push_str(&format!("}}"));
                                        *bit_offset += size;
                                    } else {
                                        panic!(
                                            "primitive data types larger than 64 are not supported"
                                        );
                                    }
                                }
                                Type::Struct {
                                    name : _,
                                    description : _,
                                    attribs,
                                    visibility : _,
                                } => {
                                    for (attrib_name, attrib_ty) in attribs {
                                        generate_fragmentation_logic(logic, &attrib_ty, &format!("{var}.{attrib_name}"), buffer, bit_offset, indent2, indent3);
                                    }
                                }
                                Type::Array { len : _, ty : _ } => todo!(),
                            }
                        }
                        generate_fragmentation_logic(
                            &mut fragmentation_logic,
                            object_entry.ty(),
                            &var,
                            &buffer_name,
                            &mut 0,
                            &indent2,
                            &indent3
                        );

                        case_logic += &format!(
                            "{indent}case {id}: {{
{fragmentation_logic}
{indent2}{buffer_offset} = 1;
{indent2}resp.data = {buffer_name}[0];
{indent2}resp.header.sof = 1;
{indent2}resp.header.eof = 0;
{indent2}resp.header.toggle = 1;
{indent2}{namespace}_request_update(10);
{indent2}break;
{indent}}}\n"
                        );
                    }
                }
                case_logic += &format!("{indent}}}\n");
                logic += &format!(
                    "{indent}get_resp resp;
{case_logic}{indent}resp.header.od_index = msg.header.od_index;
{indent}resp.header.client_id = msg.header.client_id;
{indent}resp.header.server_id = msg.header.server_id;
{indent}{frame_type_name} resp_frame;
{indent}serialize_get_resp(&resp, resp_frame.data);
{indent}resp_frame.dlc = {resp_dlc};
{indent}resp_frame.id = {resp_id};
{indent}{namespace}_can{resp_bus_id}_send(&resp_frame);
"
                );
                (logic, false)
            }
            message::MessageUsage::SetResp => ("".to_owned(), false),
            message::MessageUsage::SetReq => ("".to_owned(), false),
            message::MessageUsage::External { interval: _ } => ("".to_owned(), true),
        };

        let attributes = if weak {
            "__attribute__((weak))"
        } else {
            "static"
        };

        let handler_def = format!(
            "{attributes} void {handler_name}({frame_type_name}* frame) {{
{indent}{msg_name} msg;
{indent}deserialize_{msg_name}(frame->data, &msg);
{logic}}}\n"
        );
        source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Definition(handler_name.clone()),
            handler_def,
            vec![SourceBlockIdentifier::Definition(frame_type_name.clone())],
        ))?;
    }

    Ok(())
}

fn ty_size(ty: &Type) -> usize {
    match ty {
        Type::Primitive(signal_type) => signal_type.size() as usize,
        Type::Struct {
            name: _,
            description: _,
            attribs,
            visibility: _,
        } => {
            let mut sum = 0;
            for (_, attrib) in attribs {
                sum += ty_size(attrib as &Type);
            }
            sum
        }
        Type::Enum {
            name: _,
            description: _,
            size,
            entries: _,
            visibility: _,
        } => *size as usize,
        Type::Array { len, ty } => *len * ty_size(ty),
    }
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
