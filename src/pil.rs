use crate::{
    errors::Result,
    options::Options,
};

pub fn generate_pil(
    source: &mut String,
    header: &mut String,
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
    header.push_str(&can_frame_type_def);

    let can_frame_id_bits_name = format!("{namespace}_frame_id_bits");
    let can_frame_id_bits_def = format!(
        "typedef enum : uint32_t {{
{indent}{}_FRAME_IDE_BIT = 0x40000000, // 1 << 30
{indent}{}_FRAME_RTR_BIT = 0x80000000, // 1 << 31
}} can_frame_id_bits;\n",
        namespace.to_uppercase(),
        namespace.to_uppercase()
    );
    header.push_str(&can_frame_id_bits_def);


    // ============= CAN filter definitions ================
    let can_filter_name = format!("{namespace}_can_filter");
    let can_filter_def = format!(
        "typedef struct {{
{indent}uint32_t mask;
{indent}uint32_t id;
}} {can_filter_name};\n"
    );
    header.push_str(&can_filter_def);

    Ok(())
}
