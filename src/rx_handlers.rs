use can_config_rs::config::{self, message, Type, TypeRef};

use crate::{errors::Result, options::Options, types::to_c_type_name};

pub fn generate_rx_handlers(
    network_config: &config::NetworkRef,
    node_config: &config::NodeRef,
    source: &mut String,
    _header: &mut String,
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

    let frame_type_name = format!("{namespace}_frame");
    for message in node_config.rx_messages() {
        let msg_name = message.name();
        let handler_name = format!("{namespace}_handle_{msg_name}");

        let (logic, weak) = match message.usage() {
            message::MessageUsage::Stream(stream) => {
                // NOTE this is the instance of the tx_stream NOT the rx_stream !!!
                // pretty bad that there is no difference. Already lead to multiple bugs that
                // took a while to find. BAD sign =^(.
                let rx_stream = node_config.rx_streams().iter().find(|rx_stream| rx_stream.message().name() == stream.message().name())
                    .expect("If a node receives a stream message it should define a corresponding rx_stream. This is not the case here!");

                let Some(encoding) = message.encoding() else {
                    panic!("stream message requires a type encoding");
                };
                let mut logic = String::new();
                for (encoding, object_entry_mapping) in
                    std::iter::zip(encoding.attributes().iter(), rx_stream.mapping().iter())
                {
                    let Some(object_entry_mapping) = object_entry_mapping else {
                        continue;
                    };
                    let object_entry_name = object_entry_mapping.name();
                    // let object_entry_var = format!("__oe_{object_entry_name}");

                    let msg_attribute = encoding.name();

                    logic += &format!(
                        "{indent}{namespace}_set_{object_entry_name}(msg.{msg_attribute});\n"
                    );
                    // logic += &format!("{indent}{object_entry_var} = msg.{msg_attribute};\n");
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
                let resp_bus_id = resp_msg.bus().id();

                let command_name = command.name();
                (
                    format!(
                        "{indent}{namespace}_message_{resp_msg_name} resp{{}};
{indent}resp.erno = {namespace}_{command_name}({attribute_list});
{indent}{frame_type_name} resp_frame;
{indent}{namespace}_serialize_{namespace}_message_{resp_msg_name}(&resp, &resp_frame);
{indent}{namespace}_can{resp_bus_id}_send(&resp_frame);
"
                    ),
                    false,
                )
            }
            message::MessageUsage::CommandResp(_) => ("".to_owned(), false),
            message::MessageUsage::GetResp => panic!(),
            message::MessageUsage::GetReq => {
                let mut logic = String::new();
                let resp = network_config.get_resp_message();
                let resp_bus_id = resp.bus().id();
                let mut case_logic = format!("{indent}switch (msg.m_header.m_od_index) {{\n");
                for object_entry in node_config.object_entries() {
                    let name = object_entry.name();
                    let var = format!("__oe_{name}");
                    let size = ty_size(object_entry.ty());
                    let id = object_entry.id();
                    if size <= 32 {
                        fn generate_parse_logic(
                            parse_code: &mut String,
                            oe_name: &str,
                            oe_type: &Type,
                            bit_offset: &mut u8,
                            indent2: &str,
                        ) {
                            match oe_type {
                                Type::Primitive(signal_type) => {
                                    match signal_type {
                                        config::SignalType::UnsignedInt { size } => {
                                            let parse_uint = if *size <= 8 {
                                                format!(
                                                    "{indent2}resp.m_data |= ((uint32_t)({oe_name} \
                                                    & (0xFF >> (8 - {size})))) << {bit_offset};\n"
                                                )
                                            } else if *size <= 16 {
                                                format!(
                                                    "{indent2}resp.m_data |= ((uint32_t)({oe_name} \
                                                    & (0xFFFF >> (16 - {size})))) << {bit_offset};\n")
                                            } else if *size <= 32 {
                                                format!(
                                                    "{indent2}resp.m_data |= ({oe_name} & \
                                                    (0xFFFFFFFF >> (32 - {size}))) << {bit_offset};\n")
                                            } else if *size <= 64 {
                                                panic!("values larger than 32 should be send in fragmented mode")
                                            } else {
                                                panic!("unsigned integer larger than 64 are not supported");
                                            };
                                            parse_code.push_str(&parse_uint);
                                            *bit_offset += size;
                                        }
                                        config::SignalType::SignedInt { size } => {
                                            let parse_int = if *size <= 8 {
                                                format!("{indent2}resp.m_data |= ((uint32_t)(((uint8_t){oe_name}) \
                                                    & (0xFF >> (8 - {size})))) << {bit_offset};\n")
                                            } else if *size <= 16 {
                                                format!("{indent2}resp.m_data |= ((uint32_t)(((uint16_t){oe_name}) \
                                                    & (0xFFFF >> (16 - {size})))) << {bit_offset};\n")
                                            } else if *size <= 32 {
                                                format!("{indent2}resp.m_data |= ((uint32_t)(((uint32_t){oe_name}) \
                                                    & (0xFFFFFFFF >> (32 - {size})))) << {bit_offset};\n")
                                            } else if *size <= 64 {
                                                panic!("values larger than 32 should be send in fragmented mode")
                                            } else {
                                                panic!("signed integer larger than 64 are not supported");
                                            };
                                            parse_code.push_str(&parse_int);
                                            *bit_offset += size;
                                        }
                                        config::SignalType::Decimal {
                                            size,
                                            offset,
                                            scale,
                                        } => {
                                            let parse_dec = if *size <= 32 {
                                                format!(
                                                    "{indent2}resp.m_data |= ((uint32_t)(({oe_name} \
                                                    - ({offset})) / {scale})) << {bit_offset};\n"
                                                )
                                            } else if *size <= 64 {
                                                panic!("values larger than 32 should be send in fragmented mode")
                                            } else {
                                                panic!("decimals larger than 64 are not supported");
                                            };
                                            parse_code.push_str(&parse_dec);
                                            *bit_offset += size;
                                        }
                                    };
                                }
                                Type::Struct {
                                    name: _,
                                    description: _,
                                    attribs,
                                    visibility: _,
                                } => {
                                    for (attr_name, attr_type) in attribs {
                                        let oe_name = oe_name.to_owned() + ".m_" + attr_name;
                                        generate_parse_logic(
                                            parse_code, &oe_name, attr_type, bit_offset, indent2,
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
                                    let parse_enum = if *size <= 8 {
                                        format!(
                                            "{indent2}resp.m_data |= \
                                                ((uint32_t)(((uint8_t){oe_name}) \
                                                & (0xFF >> (8 - {size})))) << {bit_offset};\n"
                                        )
                                    } else if *size <= 16 {
                                        format!(
                                            "{indent2}resp.m_data |= \
                                                ((uint32_t)(((uint16_t){oe_name}) \
                                                & (0xFFFF >> (16 - {size})))) << {bit_offset};\n"
                                        )
                                    } else if *size <= 32 {
                                        format!("{indent2}resp.m_data |= \
                                                (((uint32_t){oe_name}) \
                                                 & (0xFFFFFFFF >> (32 - {size}))) << {bit_offset};\n")
                                    } else if *size <= 64 {
                                        panic!(
                                        "values larger than 32 should be send in fragmented mode"
                                    )
                                    } else {
                                        panic!("unsigned integer larger than 64 are not supported");
                                    };
                                    parse_code.push_str(&parse_enum);
                                    *bit_offset += size;
                                }
                                Type::Array { len: _, ty: _ } => todo!(),
                            };
                        }
                        let mut parse_code = String::new();
                        let mut bit_offset = 0;
                        let oe_type = object_entry.ty();
                        generate_parse_logic(
                            &mut parse_code,
                            &var,
                            oe_type,
                            &mut bit_offset,
                            &indent2,
                        );

                        case_logic += &format!(
                            "{indent}case {id}: {{\n\
                                {parse_code}{indent2}resp.m_header.m_sof = 1;\n\
                                {indent2}resp.m_header.m_eof = 1;\n\
                                {indent2}resp.m_header.m_toggle = 0;\n\
                                {indent2}break;\n\
                                {indent}}}\n"
                        );
                    } else {
                        let buffer_name = format!("{var}_rx_fragmentation_buffer");
                        let buffer_def =
                            format!("static uint32_t {buffer_name}[{}];\n", size.div_ceil(32));
                        source.push_str(&buffer_def);

                        let mut fragmentation_logic = String::new();
                        fn generate_fragmentation_logic(
                            logic: &mut String,
                            ty: &Type,
                            var: &str,
                            buffer: &str,
                            bit_offset: &mut usize,
                            indent2: &str,
                            indent3: &str,
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
                                        logic.push_str(&format!(
                                            "{indent3}uint64_t masked = {val};\n"
                                        ));
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
                                    name: _,
                                    description: _,
                                    attribs,
                                    visibility: _,
                                } => {
                                    for (attrib_name, attrib_ty) in attribs {
                                        generate_fragmentation_logic(
                                            logic,
                                            &attrib_ty,
                                            &format!("{var}.m_{attrib_name}"),
                                            buffer,
                                            bit_offset,
                                            indent2,
                                            indent3,
                                        );
                                    }
                                }
                                Type::Array { len: _, ty: _ } => todo!(),
                            }
                        }
                        generate_fragmentation_logic(
                            &mut fragmentation_logic,
                            object_entry.ty(),
                            &var,
                            &buffer_name,
                            &mut 0,
                            &indent2,
                            &indent3,
                        );

                        let buffer_size = size.div_ceil(32);
                        let od_index = object_entry.id();
                        case_logic += &format!(
                            "{indent}case {id}: {{
{fragmentation_logic}
{indent2}resp.m_data = {buffer_name}[0];
{indent2}resp.m_header.m_sof = 1;
{indent2}resp.m_header.m_eof = 0;
{indent2}resp.m_header.m_toggle = 0;
{indent2}schedule_get_resp_fragmentation_job({buffer_name}, {buffer_size}, {od_index}, msg.m_header.m_client_id);
{indent2}break;
{indent}}}\n"
                        );
                    }
                }
                case_logic += &format!("{indent}}}\n");
                let node_id = node_config.id();
                let resp_bus_name = network_config.get_resp_message().bus().name();
                logic += &format!(
                    "{indent}if (msg.m_header.m_server_id != {node_id}) {{
{indent2}return;
{indent}}}
{indent}{namespace}_message_get_resp resp{{}};
{case_logic}{indent}resp.m_header.m_od_index = msg.m_header.m_od_index;
{indent}resp.m_header.m_client_id = msg.m_header.m_client_id;
{indent}resp.m_header.m_server_id = msg.m_header.m_server_id;
{indent}{frame_type_name} resp_frame;
{indent}{namespace}_serialize_{namespace}_message_get_resp(&resp, &resp_frame);
{indent}{namespace}_{resp_bus_name}_send(&resp_frame);
"
                );
                (logic, false)
            }
            message::MessageUsage::SetResp => panic!(),
            message::MessageUsage::SetReq => {
                let node_id = node_config.id();
                let mut case_logic = format!("{indent}switch (msg.m_header.m_od_index) {{\n");
                for object_entry in node_config.object_entries() {
                    let od_index = object_entry.id();
                    let size = ty_size(object_entry.ty());
                    let oe_name = object_entry.name();
                    let ty = to_c_type_name(object_entry.ty());
                    let oe_tmp_var = format!("{ty} {oe_name}_tmp;\n");
                    let mut parse_logic = format!("{oe_tmp_var}");
                    let oe_var = format!("{oe_name}_tmp");
                    if size <= 32 {
                        fn generate_parse_logic(
                            parse_logic: &mut String,
                            ty: &Type,
                            var: &str,
                            attrib_offset: &mut usize,
                        ) {
                            match ty {
                                Type::Primitive(signal_type) => {
                                    let size = signal_type.size() as usize;

                                    let masked_val =
                                        format!("(msg.m_data & (0xFFFFFFFF >> (32 - {size})))");

                                    let parsed_val = match signal_type {
                                        config::SignalType::UnsignedInt { size } => {
                                            if *size <= 8 {
                                                format!("(uint8_t){masked_val}")
                                            } else if *size <= 16 {
                                                format!("(uint16_t){masked_val}")
                                            } else if *size <= 32 {
                                                format!("(uint32_t){masked_val}")
                                            } else if *size <= 64 {
                                                format!("(uint64_t){masked_val}")
                                            } else {
                                                panic!("unsigned integers larger than 64 bit are not supported");
                                            }
                                        }
                                        config::SignalType::SignedInt { size } => {
                                            if *size <= 8 {
                                                format!("(int8_t){masked_val}")
                                            } else if *size <= 16 {
                                                format!("(int16_t){masked_val}")
                                            } else if *size <= 32 {
                                                format!("(int32_t){masked_val}")
                                            } else if *size <= 64 {
                                                format!("(int64_t){masked_val}")
                                            } else {
                                                panic!("unsigned integers larger than 64 bit are not supported");
                                            }
                                        }
                                        config::SignalType::Decimal {
                                            size,
                                            offset,
                                            scale,
                                        } => {
                                            if *size <= 32 {
                                                format!(
                                                    "(float)({masked_val} * {scale} + {offset})"
                                                )
                                            } else if *size <= 64 {
                                                format!(
                                                    "(double)({masked_val} * {scale} + {offset})"
                                                )
                                            } else {
                                                panic!("decimal data types larger than 64 bit are not supported");
                                            }
                                        }
                                    };
                                    parse_logic.push_str(&format!("{var} = {parsed_val};\n"));
                                    *attrib_offset += size as usize;
                                }
                                Type::Struct {
                                    name: _,
                                    description: _,
                                    attribs,
                                    visibility: _,
                                } => {
                                    for (attrib_name, attrib_ty) in attribs {
                                        generate_parse_logic(
                                            parse_logic,
                                            attrib_ty,
                                            &format!("{var}.m_{attrib_name}"),
                                            attrib_offset,
                                        );
                                    }
                                }
                                Type::Enum {
                                    name,
                                    description: _,
                                    size,
                                    entries: _,
                                    visibility: _,
                                } => {
                                    let size = *size as usize;

                                    let masked_val =
                                        format!("(msg.m_data & (0xFFFFFFFF >> (32 - {size})))");

                                    let parsed_val = format!("({name})({masked_val})");
                                    parse_logic.push_str(&format!("{var} = {parsed_val};\n"));
                                    *attrib_offset += size as usize;
                                }
                                Type::Array { len: _, ty: _ } => todo!(),
                            }
                        }
                        generate_parse_logic(
                            &mut parse_logic,
                            object_entry.ty(),
                            &format!("{indent2}{oe_var}"),
                            &mut 0,
                        );
                        case_logic.push_str(&format!(
                            "{indent}case {od_index} : {{
{indent2}if (msg.m_header.m_sof != 1 || msg.m_header.m_toggle != 0 || msg.m_header.m_eof != 1) {{
{indent3}return;
{indent2}}}
{indent2}{parse_logic}{indent2}{namespace}_set_{oe_name}({oe_var});
{indent2}break;
{indent}}}
"
                        ));
                    } else {
                        let word_size = size.div_ceil(32);
                        let buffer_name = format!("{oe_var}_tx_fragmentation_buffer");
                        let buffer_offset = format!("{oe_var}_tx_fragmentation_offset");
                        source.push_str(&format!("static uint32_t {buffer_name}[{word_size}];\n"));
                        source.push_str(&format!("static uint32_t {buffer_offset} = 0;\n"));

                        let mut write_logic = String::new();
                        fn generate_write_logic(
                            write_logic: &mut String,
                            ty: &Type,
                            bit_offset: &mut usize,
                            buffer_name: &str,
                            var: &str,
                            indent: &str,
                        ) {
                            match ty {
                                Type::Primitive(signal_type) => {
                                    let size = signal_type.size() as usize;
                                    let bit_word_offset = *bit_offset % 32;
                                    let word_offset = *bit_offset / 32;
                                    let val_bits = if bit_word_offset == 0 && size <= 32 {
                                        format!("({buffer_name}[{word_offset}] & (0xFFFFFFFF >> (32 - {size})))")
                                    } else if bit_word_offset == 0 && size > 32 {
                                        assert!(size <= 64);
                                        let upper_word_offset = word_offset + 1;
                                        let upper_word_bit_offset = (bit_word_offset + size) - 32;
                                        format!("(uint64_t){buffer_name}[{word_offset}] | (((uint64_t)({buffer_name}[{upper_word_offset}] & (0xFFFFFFFF >> (32 - {upper_word_bit_offset})))) << 32)")
                                    } else if bit_word_offset + size <= 32 {
                                        format!("({buffer_name}[{word_offset}] << {bit_word_offset}) & (0xFFFFFFFF >> (32 - {size}))")
                                    } else if bit_word_offset + size < 32 {
                                        let upper_word_offset = word_offset + 1;
                                        let upper_word_bit_offset = (bit_word_offset + size) - 32;
                                        format!("(uint64_t)({buffer_name}[{word_offset}] << {bit_word_offset}) | ((uint64_t)({buffer_name}[{upper_word_offset}] & (0xFFFFFFFF >> (32 - {upper_word_bit_offset}))) << 32")
                                    } else {
                                        panic!();
                                    };
                                    let val = match signal_type {
                                        config::SignalType::UnsignedInt { size: _ } => {
                                            format!("{val_bits}")
                                        }
                                        config::SignalType::SignedInt { size: _ } => {
                                            format!("{val_bits}")
                                        }
                                        config::SignalType::Decimal {
                                            size: _,
                                            offset,
                                            scale,
                                        } => format!("({val_bits}) * {scale} + {offset}"),
                                    };
                                    write_logic.push_str(&format!("{indent}{var} = {val};\n"));
                                }
                                Type::Struct {
                                    name: _,
                                    description: _,
                                    attribs,
                                    visibility: _,
                                } => {
                                    for (attrib_name, attrib_ty) in attribs {
                                        generate_write_logic(
                                            write_logic,
                                            attrib_ty,
                                            bit_offset,
                                            buffer_name,
                                            &format!("{var}.m_{attrib_name}"),
                                            indent,
                                        )
                                    }
                                }
                                Type::Enum {
                                    name,
                                    description: _,
                                    size,
                                    entries: _,
                                    visibility: _,
                                } => {
                                    let size = *size as usize;
                                    let bit_word_offset = *bit_offset % 32;
                                    let word_offset = *bit_offset / 32;
                                    let val_bits = if bit_word_offset == 0 && size <= 32 {
                                        format!("({buffer_name}[{word_offset}] & (0xFFFFFFFF >> (32 - {size})))")
                                    } else if bit_word_offset == 0 && size > 32 {
                                        assert!(size <= 64);
                                        let upper_word_offset = word_offset + 1;
                                        let upper_word_bit_offset = (bit_word_offset + size) % 32;
                                        format!("(uint64_t){buffer_name}[{word_offset}] | (((uint64_t)({buffer_name}[{upper_word_offset}] & (0xFFFFFFFF >> (32 - {upper_word_bit_offset})))) << 32)")
                                    } else if bit_word_offset + size <= 32 {
                                        format!("({buffer_name}[{word_offset}] << {bit_word_offset}) & (0xFFFFFFFF >> (32 - {size}))")
                                    } else if bit_word_offset + size < 32 {
                                        let upper_word_offset = word_offset + 1;
                                        let upper_word_bit_offset = (bit_word_offset + size) % 32;
                                        format!("(uint64_t)({buffer_name}[{word_offset}] << {bit_word_offset}) | ((uint64_t)({buffer_name}[{upper_word_offset}] & (0xFFFFFFFF >> (32 - {upper_word_bit_offset}))) << 32)")
                                    } else {
                                        panic!();
                                    };
                                    let val = format!("(({name}){val_bits})");
                                    write_logic.push_str(&format!("{indent}{var} = {val};\n"));
                                }
                                Type::Array { len: _, ty: _ } => todo!(),
                            }
                        }
                        generate_write_logic(
                            &mut write_logic,
                            object_entry.ty(),
                            &mut 0,
                            &buffer_name,
                            &oe_var,
                            &indent2,
                        );
                        let oe_ty = to_c_type_name(object_entry.ty());

                        case_logic.push_str(&format!(
                            "{indent}case {od_index} : {{
{indent2}if (msg.m_header.m_sof == 1) {{
{indent3}if (msg.m_header.m_toggle == 0 || msg.m_header.m_eof != 0) {{
{indent4}return;
{indent3}}}
{indent3}{buffer_offset} = 0;
{indent2}}}else {{
{indent3}{buffer_offset} += 1;
{indent3}if ({buffer_offset} >= {word_size}) {{
{indent4}return;
{indent3}}}
{indent2}}}
{indent2}{buffer_name}[{buffer_offset}] = msg.m_data;
{indent2}if (msg.m_header.m_eof == 0) {{
{indent3}return;
{indent2}}}
{indent2}{oe_ty} {oe_var};
{write_logic}{indent2}{namespace}_set_{oe_name}({oe_var});
{indent2}break;
{indent}}}
"
                        ));
                    }
                }
                case_logic.push_str(&format!("{indent}default:\n{indent2}return;\n{indent}}}"));
                let resp_bus_name = network_config.set_resp_message().bus().name();
                let logic = format!(
                    "{indent}if (msg.m_header.m_server_id != {node_id}) {{
{indent2}return;
{indent}}}
{indent}{namespace}_message_set_resp resp{{}};
{case_logic}
{indent}resp.m_header.m_od_index = msg.m_header.m_od_index;
{indent}resp.m_header.m_client_id = msg.m_header.m_client_id;
{indent}resp.m_header.m_server_id = msg.m_header.m_server_id;
{indent}resp.m_header.m_erno = set_resp_erno_Success;
{indent}canzero_frame resp_frame;
{indent}{namespace}_serialize_{namespace}_message_set_resp(&resp, &resp_frame);
{indent}{namespace}_{resp_bus_name}_send(&resp_frame);\n
"
                );

                (logic, false)
            }
            message::MessageUsage::Heartbeat => ("".to_owned(), false),
            message::MessageUsage::External { interval: _ } => ("".to_owned(), true),
        };

        let attributes = if weak {
            "__attribute__((weak))"
        } else {
            "static"
        };

        let handler_def = format!(
            "{attributes} void {handler_name}({frame_type_name}* frame) {{
{indent}{namespace}_message_{msg_name} msg;
{indent}{namespace}_deserialize_{namespace}_message_{msg_name}(frame, &msg);
{logic}}}\n"
        );
        source.push_str(&handler_def);
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
