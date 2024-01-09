use can_config_rs::config::bus::BusRef;

use crate::{
    errors::Result,
    file_buffer::FileBuffer,
    options::Options,
    source_block::{SourceBlock, SourceBlockIdentifier},
};

pub fn generate_hooks(
    buses: &Vec<BusRef>,
    source: &mut FileBuffer,
    _header: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    for bus in buses {
        let bus_id = bus.id();
        let can_setup_name = format!("{namespace}_can{bus_id}_setup");
        let can_setup_decl = format!("extern void {can_setup_name}(uint32_t baudrate, {namespace}_can_filter* filters, int filter_count);\n");
        source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Declartion(can_setup_name.clone()),
            can_setup_decl,
            vec![SourceBlockIdentifier::Import("inttypes.h".to_owned())],
        ))?;

        let can_send_name = format!("{namespace}_can{bus_id}_send");
        let can_send_decl = format!("extern void {can_send_name}({namespace}_frame* frame);\n");
        source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Declartion(can_send_name.clone()),
            can_send_decl,
            vec![SourceBlockIdentifier::Definition(format!(
                "{namespace}_frame"
            ))],
        ))?;

        let can_recv_name = format!("{namespace}_can{bus_id}_recv");
        let can_recv_decl = format!("extern int {can_recv_name}({namespace}_frame* frame);\n");
        source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Declartion(can_recv_name.clone()),
            can_recv_decl,
            vec![SourceBlockIdentifier::Definition(format!(
                "{namespace}_frame"
            ))],
        ))?;
    }

    let can_request_update_name = format!("{namespace}_request_update");
    let can_request_update_decl = format!("extern void {can_request_update_name}(uint32_t time);\n");
    source.add_block(SourceBlock::new(
        SourceBlockIdentifier::Declartion(can_request_update_name.clone()),
        can_request_update_decl,
        vec![SourceBlockIdentifier::Import("inttypes.h".to_owned())],
    ))?;

    Ok(())
}
