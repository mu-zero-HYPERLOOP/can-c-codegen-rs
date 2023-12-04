use crate::{
    errors::Result,
    file_buffer::FileBuffer,
    options::Options,
    source_block::{SourceBlock, SourceBlockIdentifier},
};

pub fn generate_can_frame(
    header: &mut FileBuffer,
    source: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }

    let can_frame_name = &format!("{namespace}_frame");
    let mut can_frame_def = format!("
typedef struct {{
  uint32_t _id;
  uint8_t _dlc;
  uint8_t _data[8];
}} {can_frame_name};

// expects data to point to a 8 byte array
inline {can_frame_name} {can_frame_name}_new(uint32_t id, 
                                      int ide, 
                                      int rtr, 
                                      uint8_t dlc,
                                      uint8_t* data) {{
{indent}{can_frame_name} frame;
{indent}frame._id = id << 2 | (!!rtr) << 1 | (!!ide);
{indent}frame._dlc = dlc;
{indent}*((uint64_t*)frame._data) = *((uint64_t*)data);
{indent}return frame;
}}
inline uint32_t {can_frame_name}_get_id({can_frame_name}* self) {{
{indent}return self->_id >> 2;
}}
inline int {can_frame_name}_get_ide({can_frame_name}* self) {{
{indent}return self->_id & 0x1;
}}
inline int {can_frame_name}_get_rtr({can_frame_name}* self) {{
{indent}return self->_id & 0x2;
}}
inline uint8_t {can_frame_name}_get_dlc({can_frame_name}* self) {{
{indent}return self->_dlc;
}}
inline uint8_t* {can_frame_name}_get_data({can_frame_name}* self) {{
{indent}return self->_data;
}}");
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_name.clone()),
        can_frame_def,
        vec![SourceBlockIdentifier::Import("inttypes.h".to_owned())],
    ))?;

    Ok(())
}
