use can_config_rs::config::encoding::PrimitiveSignalEncoding;
use can_config_rs::config::{self, MessageRef, SignalType, Type, TypeSignalEncoding};

use crate::errors::Result;
use crate::source_block::{SourceBlock, SourceBlockIdentifier};
use crate::types::to_c_type_name;
use crate::{file_buffer::FileBuffer, options::Options};

pub fn generate_messages(
    tx_messages: &Vec<MessageRef>,
    rx_messages: &Vec<MessageRef>,
    header: &mut FileBuffer,
    source: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }

    // create a unique set of all messages!
    let mut unique_set = tx_messages.clone();
    for rx_message in rx_messages {
        if !unique_set.iter().any(|m| m.name() == rx_message.name()) {
            unique_set.push(rx_message.clone());
        }
    }

    // create structs for all messages
    for message in &unique_set {
        let message_type_name = message.name();
        let mut dependencies = vec![];
        // type declartion of the message struct!
        let mut type_def = format!("typedef struct {{\n");
        match message.encoding() {
            Some(encoding) => {
                for attrib in encoding.attributes() {
                    let (attrib_type_name, dep) = to_c_type_name(attrib.ty());
                    match dep {
                        Some(dep) => {
                            if !dependencies.contains(&dep) {
                                dependencies.push(dep);
                            }
                        }
                        None => (),
                    };
                    let attrib_name = attrib.name();
                    type_def.push_str(&format!("{indent}{attrib_type_name} {attrib_name};\n"));
                }
            }
            None => {
                for signal in message.signals() {
                    let (signal_type_name, dep) = signal_type_to_c_type(signal.ty());
                    match dep {
                        Some(dep) => {
                            if !dependencies.contains(&dep) {
                                dependencies.push(dep);
                            }
                        }
                        None => (),
                    }
                    let signal_name = signal.name();
                    type_def.push_str(&format!("{indent}{signal_type_name} {signal_name};\n"));
                }
            }
        }

        type_def.push_str(&format!("}} {message_type_name};\n"));
        source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Definition(message_type_name.to_owned()),
            type_def,
            dependencies,
        ))?;

        if tx_messages.iter().any(|m| m.name() == message.name()) {
            // function to serialize the message struct into a can frame!
            let serialize_func_name = format!("serialize_{message_type_name}");
            let mut serialize_def = format!(
                "static void {serialize_func_name}({message_type_name}* msg, uint8_t* data) {{\n"
            );

            match message.encoding() {
                Some(encoding) => {
                    fn write_attribute_parse_code(
                        serialized_def: &mut String,
                        attrib: &TypeSignalEncoding,
                        indent: &str,
                        attribute_prefix: &str,
                    ) {
                        match attrib {
                            TypeSignalEncoding::Composite(composite) => {
                                let attrib_name = composite.name();
                                let attrib_prefix = format!("{attribute_prefix}{attrib_name}");
                                for attrib in composite.attributes() {
                                    write_attribute_parse_code(serialized_def, attrib, indent, &attrib_prefix);
                                }
                            }
                            TypeSignalEncoding::Primitive(primitive) => {
                                let attrib_name = primitive.name();
                                let signal = primitive.signal();
                                match attrib.ty() as &Type {
                                    config::Type::Primitive(signal_type) => {
                                        let var = match signal_type {
                                            SignalType::UnsignedInt { size: _ } => {
                                                format!("{attribute_prefix}{attrib_name}")
                                            }
                                            SignalType::SignedInt { size: _ } => {
                                                format!("{attribute_prefix}{attrib_name}")
                                            }
                                            SignalType::Decimal {
                                                size: _,
                                                offset,
                                                scale,
                                            } => {
                                                format!("({attribute_prefix}{attrib_name} * {scale} + {offset})")
                                            }
                                        };
                                        let bit_write_code = bit_write_code(
                                            signal.byte_offset(),
                                            signal_type.size() as usize,
                                            "data",
                                            &var,
                                        );

                                        serialized_def
                                            .push_str(&format!("{indent}{bit_write_code};\n"));
                                    }
                                    config::Type::Struct {
                                        name: _,
                                        description: _,
                                        attribs : _,
                                        visibility: _,
                                    } => panic!("structs are not primitive"),
                                    config::Type::Enum {
                                        name: _,
                                        description: _,
                                        size,
                                        entries: _,
                                        visibility: _,
                                    } => {
                                        let bit_write_code = bit_write_code(
                                            signal.byte_offset(),
                                            *size as usize,
                                            "data",
                                            &format!("({attribute_prefix}{attrib_name})"),
                                        );

                                        serialized_def
                                            .push_str(&format!("{indent}{bit_write_code};\n"));
                                    }
                                    config::Type::Array { len: _, ty: _ } => todo!(),
                                };
                            }
                        }
                    }

                    for attrib in encoding.attributes() {
                        write_attribute_parse_code(
                            &mut serialize_def,
                            attrib,
                            &indent,
                            "msg->",
                        );
                    }
                }
                None => {
                    for signal in message.signals() {
                        let signal_name = signal.name();
                        let var = match signal.ty() {
                            SignalType::UnsignedInt { size: _ } => {
                                format!("msg->{signal_name}")
                            }
                            SignalType::SignedInt { size: _ } => {
                                format!("msg->{signal_name}")
                            }
                            SignalType::Decimal {
                                size: _,
                                offset,
                                scale,
                            } => {
                                format!("(msg->{signal_name} * {scale} + {offset})")
                            }
                        };
                        let bit_write_code = bit_write_code(
                            signal.byte_offset(),
                            signal.size() as usize,
                            "data",
                            &var,
                        );

                        serialize_def.push_str(&format!("{indent}{bit_write_code};"));
                    }
                }
            };

            serialize_def.push_str("}\n");
            source.add_block(SourceBlock::new(
                SourceBlockIdentifier::Definition(serialize_func_name),
                serialize_def,
                vec![SourceBlockIdentifier::Definition(
                    message_type_name.to_owned(),
                )],
            ))?;
        }

        if rx_messages.iter().any(|m| m.name() == message.name()) {
            // function to serialize the message struct into a can frame!
            let deserialize_func_name = format!("deserialize_{message_type_name}");
            let mut deserialize_def = format!(
                "static void {deserialize_func_name}(uint8_t* data, {message_type_name}* msg) {{\n"
            );

            match message.encoding() {
                Some(encoding) => {
                    fn write_attribute_write_code(
                        deserialized_def: &mut String,
                        attrib: &TypeSignalEncoding,
                        indent: &str,
                        attribute_prefix: &str,
                    ) {
                        match attrib {
                            TypeSignalEncoding::Composite(composite) => {
                                let attrib_name = composite.name();
                                let attrib_prefix = format!("{attribute_prefix}{attrib_name}.");
                                for attrib in composite.attributes() {
                                    write_attribute_write_code(deserialized_def, attrib, indent, &attrib_prefix);
                                }
                            }
                            TypeSignalEncoding::Primitive(primitive) => {
                                let attrib_name = primitive.name();
                                let signal = primitive.signal();
                                match attrib.ty() as &Type {
                                    config::Type::Primitive(signal_type) => {
                                        let var = match signal_type {
                                            SignalType::UnsignedInt { size: _ } => {
                                                format!("{attribute_prefix}{attrib_name}")
                                            }
                                            SignalType::SignedInt { size: _ } => {
                                                format!("{attribute_prefix}{attrib_name}")
                                            }
                                            SignalType::Decimal {
                                                size: _,
                                                offset,
                                                scale,
                                            } => {
                                                format!("({attribute_prefix}{attrib_name} * {scale} + {offset})")
                                            }
                                        };
                                        let bit_write_code = bit_access_code(
                                            signal.byte_offset(),
                                            signal_type.size() as usize,
                                            "data",
                                        );

                                        deserialized_def
                                            .push_str(&format!("{indent}{var} = {bit_write_code};\n"));
                                    }
                                    config::Type::Struct {
                                        name: _,
                                        description: _,
                                        attribs : _,
                                        visibility: _,
                                    } => panic!("structs are not primitive"),
                                    config::Type::Enum {
                                        name: _,
                                        description: _,
                                        size,
                                        entries: _,
                                        visibility: _,
                                    } => {
                                        let bit_write_code = bit_access_code(
                                            signal.byte_offset(),
                                            *size as usize,
                                            "data"
                                        );

                                        deserialized_def
                                            .push_str(&format!("{indent}{attribute_prefix}{attrib_name} = {bit_write_code};\n"));
                                    }
                                    config::Type::Array { len: _, ty: _ } => todo!(),
                                };
                            }
                        }
                    }

                    for attrib in encoding.attributes() {
                        write_attribute_write_code(
                            &mut deserialize_def,
                            attrib,
                            &indent,
                            "msg->",
                        );
                    }
                }
                None => {
                    for signal in message.signals() {
                        let signal_name = signal.name();
                        let var = match signal.ty() {
                            SignalType::UnsignedInt { size: _ } => {
                                format!("msg->{signal_name}")
                            }
                            SignalType::SignedInt { size: _ } => {
                                format!("msg->{signal_name}")
                            }
                            SignalType::Decimal {
                                size: _,
                                offset,
                                scale,
                            } => {
                                format!("(msg->{signal_name} * {scale} + {offset})")
                            }
                        };
                        let bit_write_code = bit_access_code(
                            signal.byte_offset(),
                            signal.size() as usize,
                            "data",
                        );


                        deserialize_def.push_str(&format!("{indent}{var} = {bit_write_code};"));
                    }
                }
            };

            deserialize_def.push_str("}\n");
            source.add_block(SourceBlock::new(
                SourceBlockIdentifier::Definition(deserialize_func_name),
                deserialize_def,
                vec![SourceBlockIdentifier::Definition(
                    message_type_name.to_owned(),
                )],
            ))?;
        }
    }

    Ok(())
}

pub fn signal_type_to_c_type(signal_type: &SignalType) -> (&str, Option<SourceBlockIdentifier>) {
    match signal_type {
        config::SignalType::UnsignedInt { size } => {
            let bit_count = (*size as f64).log2().ceil() as usize;
            let type_name = if bit_count <= 8 {
                "uint8_t"
            } else if bit_count <= 16 {
                "uint16_t"
            } else if bit_count <= 32 {
                "uint32_t"
            } else if bit_count <= 64 {
                "uint64_t"
            } else {
                panic!()
            };
            (
                type_name,
                Some(SourceBlockIdentifier::Import("inttypes.h".to_owned())),
            )
        }
        config::SignalType::SignedInt { size } => {
            let bit_count = (*size as f64).log2().ceil() as usize;
            let type_name = if bit_count <= 8 {
                "int8_t"
            } else if bit_count <= 16 {
                "int16_t"
            } else if bit_count <= 32 {
                "int32_t"
            } else if bit_count <= 64 {
                "int64_t"
            } else {
                panic!()
            };
            (
                type_name,
                Some(SourceBlockIdentifier::Import("inttypes.h".to_owned())),
            )
        }
        config::SignalType::Decimal {
            size,
            offset: _,
            scale: _,
        } => {
            if *size <= 32 {
                ("float", None)
            } else {
                ("double", None)
            }
        }
    }
}

fn bit_access_code(bit_offset: usize, bit_size: usize, buffer_name: &str) -> String {
    if bit_size <= 32 && (bit_size + bit_offset % 32) <= 32 {
        //access half word access!
        let word_bit_offset = bit_offset % 32;
        let word_index = bit_offset / 32;
        let mask = (0xFFFFFFFF as u32).overflowing_shl(32 - bit_size as u32).0 >> word_bit_offset;
        let shift = word_bit_offset;
        format!("(((int32_t*){buffer_name})[{word_index}] & 0x{mask:X}) >> {shift}")
    } else {
        let mask = (0xFFFFFFFFFFFFFFFF as u64)
            .overflowing_shl(64 - bit_size as u32)
            .0
            >> bit_offset;
        let shift = bit_offset;
        format!("(((int64_t*){buffer_name}) & 0x{mask:X}) >> {shift}")
    }
}

fn bit_write_code(bit_offset: usize, bit_size: usize, buffer_name: &str, value: &str) -> String {
    if bit_size <= 32 && (bit_size + bit_offset % 32) <= 32 {
        let word_bit_offset = bit_offset % 32;
        let word_index = bit_offset / 8;
        format!("((uint32_t*){buffer_name})[{word_index}] |= {value} << {word_bit_offset}")
    } else {
        format!("*((uint64_t*){buffer_name}) |= {value} << {bit_offset}")
    }
}
