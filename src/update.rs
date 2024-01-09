use crate::{
    errors::Result,
    file_buffer::FileBuffer,
    options::Options,
    source_block::{SourceBlock, SourceBlockIdentifier},
};

pub fn generate_update(
    source: &mut FileBuffer,
    header: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }

    let can_update_continue_name = format!("{namespace}_update_continue");

    let can_update_continue_decl =
        format!("uint32_t {can_update_continue_name}(uint32_t delta_time);\n");
    header.add_block(SourceBlock::new(
        SourceBlockIdentifier::Declartion(can_update_continue_name.clone()),
        can_update_continue_decl,
        vec![SourceBlockIdentifier::Import("inttypes.h".to_owned())],
    ))?;

    // fragmentation buffer data!

    let can_update_continue_def = format!(
        "uint32_t {can_update_continue_name}(uint32_t delta_time){{
{indent}return 100;
}}"
    );
    source.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition(can_update_continue_name.clone()),
        can_update_continue_def,
        vec![SourceBlockIdentifier::Declartion(
            can_update_continue_name.clone(),
        )],
    ))?;

    Ok(())
}
