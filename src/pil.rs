use crate::{
    errors::Result,
    file_buffer::FileBuffer,
    options::Options,
    source_block::{SourceBlock, SourceBlockIdentifier},
};

pub fn generate_pil(
    source: &mut FileBuffer,
    header: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    let indent2 = format!("{indent}{indent}");

    // =====================can_frame definition===================
    let can_frame_name = format!("{namespace}_frame");
    let can_frame_type_def = format!(
        "typedef struct {{
{indent}uint32_t id;
{indent}uint8_t dlc;
{indent}uint8_t data[8];
}} {can_frame_name};\n"
    );

    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_name.clone()),
        can_frame_type_def,
        vec![SourceBlockIdentifier::Import("inttypes.h".to_owned())],
    ))?;

    let can_frame_id_bits_name = format!("{namespace}_frame_id_bits");
    let can_frame_id_bits_def = format!(
        "typedef enum : uint32_t {{
{indent}{}_FRAME_IDE_BIT = 0x40000000, // 1 << 30
{indent}{}_FRAME_RTR_BIT = 0x80000000, // 1 << 31
}} can_frame_id_bits;\n",
        namespace.to_uppercase(),
        namespace.to_uppercase()
    );
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_id_bits_name.clone()),
        can_frame_id_bits_def,
        vec![SourceBlockIdentifier::Import("inttypes.h".to_owned())],
    ))?;

    let can_frame_new_name = format!("{namespace}_frame_new");
    let can_frame_new_inline_def =
        format!("static inline {can_frame_name} {can_frame_new_name}(uint32_t id,  uint8_t* data, int dlc) {{
{indent}{can_frame_name} frame;
{indent}frame.id = id;
{indent}frame.dlc = dlc;
{indent}for (uint8_t i = 0; i < dlc; ++i) {{
{indent2}frame.data[i] = data[i];
{indent}}}
{indent}return frame; //RVO
}}\n");
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_new_name.clone()),
        can_frame_new_inline_def,
        vec![
            SourceBlockIdentifier::Import("inttypes.h".to_owned()),
            SourceBlockIdentifier::Definition(can_frame_name.clone()),
        ],
    ))?;

    let can_frame_is_ext_name = format!("{namespace}_frame_is_ext");
    let can_frame_is_ext_def = format!(
        "static inline int {can_frame_is_ext_name}({can_frame_name} *frame) {{
{indent}return (frame->id & {}_FRAME_IDE_BIT) != 0;
}}\n",
        namespace.to_uppercase()
    );
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_is_ext_name.clone()),
        can_frame_is_ext_def,
        vec![
            SourceBlockIdentifier::Definition(can_frame_id_bits_name.clone()),
            SourceBlockIdentifier::Definition(can_frame_name.clone()),
        ],
    ))?;

    let can_frame_is_std_name = format!("{namespace}_frame_is_std");
    let can_frame_is_std_def = format!(
        "static inline int {can_frame_is_std_name}({can_frame_name} *frame) {{
{indent}return !{can_frame_is_ext_name}(frame);
}}\n"
    );
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_is_std_name.clone()),
        can_frame_is_std_def,
        vec![
            SourceBlockIdentifier::Definition(can_frame_is_ext_name.clone()),
            SourceBlockIdentifier::Definition(can_frame_name.clone()),
        ],
    ))?;

    let can_frame_is_rtr_name = format!("{namespace}_frame_is_rtr");
    let can_frame_is_rtr_def = format!(
        "static inline int {can_frame_is_rtr_name}({can_frame_name} *frame) {{
{indent}return (frame->id & {}_FRAME_RTR_BIT) != 0;
}}\n",
        namespace.to_uppercase()
    );
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_is_rtr_name.clone()),
        can_frame_is_rtr_def,
        vec![
            SourceBlockIdentifier::Definition(can_frame_id_bits_name.clone()),
            SourceBlockIdentifier::Definition(can_frame_name.clone()),
        ],
    ))?;

    let can_frame_get_data_name = format!("{namespace}_frame_get_data");
    let can_frame_get_data_def = format!(
        "static inline uint8_t* {can_frame_get_data_name}({can_frame_name} *frame) {{
{indent}return frame->data;
}}\n"
    );
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_get_data_name.clone()),
        can_frame_get_data_def,
        vec![SourceBlockIdentifier::Definition(can_frame_name.clone())],
    ))?;

    let can_frame_get_dlc_name = format!("{namespace}_frame_get_dlc");
    let can_frame_get_dlc_def = format!(
        "static inline uint8_t {can_frame_get_dlc_name}({can_frame_name} *frame) {{
{indent}return frame->dlc;
}}\n"
    );
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_get_dlc_name.clone()),
        can_frame_get_dlc_def,
        vec![
            SourceBlockIdentifier::Definition(can_frame_name.clone()),
            SourceBlockIdentifier::Import("inttypes.h".to_owned()),
        ],
    ))?;

    let can_frame_get_id_name = format!("{namespace}_frame_get_id");
    let can_frame_get_id_def = format!(
        "static inline uint32_t {can_frame_get_id_name}({can_frame_name} *frame) {{
{indent}return frame->id;
}}\n"
    );
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_frame_get_id_name.clone()),
        can_frame_get_id_def,
        vec![
            SourceBlockIdentifier::Definition(can_frame_name.clone()),
            SourceBlockIdentifier::Import("inttypes.h".to_owned()),
        ],
    ))?;

    // ============= CAN filter definitions ================
    let can_filter_name = format!("{namespace}_can_filter");
    let can_filter_def = format!(
        "typedef struct {{
{indent}uint32_t mask;
{indent}uint32_t id;
}} {can_filter_name};\n"
    );
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_filter_name.clone()),
        can_filter_def,
        vec![SourceBlockIdentifier::Import("inttypes.h".to_owned())],
    ))?;

    Ok(())
}
