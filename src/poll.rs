use can_config_rs::config;

use crate::source_block::{SourceBlock, SourceBlockIdentifier};
use crate::{file_buffer::FileBuffer, options::Options};

use crate::errors::Result;

pub fn generate_poll(
    node_config: &config::NodeRef,
    header: &mut FileBuffer,
    source: &mut FileBuffer,
    options: &Options,
) -> Result<()>{
    let namespace = options.namespace();
    let mut indent = String::new();
    for _ in 0..options.indent() {
        indent.push(' ');
    }
    let indent2 = format!("{indent}{indent}");
    let indent3 = format!("{indent2}{indent}");
    
    let poll_func_name = format!("{namespace}_poll");
    let poll_func_decl = format!("void {poll_func_name}();\n");

    header.add_block(SourceBlock::new(SourceBlockIdentifier::Declartion(poll_func_name.clone()), 
                                      poll_func_decl, vec![]))?;

    let mut poll_func_def = format!("void {poll_func_name}() {{\n");
    poll_func_def.push_str(&format!("{indent}can_frame frame;\n"));
    poll_func_def.push_str(&format!("{indent}while (rx_queue_dequeue(&rx_queue, &frame) {{\n"));
    poll_func_def.push_str(&format!("{indent}switch (frame.header) {{\n"));
    for message in node_config.tx_messages() {
        let message_name = message.name();
        // first bit stands for ide bit
        let key = match message.id()  {
            config::MessageId::StandardId(id) => {
                id << 1 | 0
            },
            config::MessageId::ExtendedId(id) =>{
                id << 1 | 1
            }
        };
        poll_func_def.push_str(&format!("{indent2}case 0x{key:X}: \n"));
        let deserialize_func_name = format!("deserialize_{message_name}");
        poll_func_def.push_str(&format!("{indent3}{message_name} msg;\n"));
        poll_func_def.push_str(&format!("{indent3}{deserialize_func_name}(frame.data, &msg);\n"));
        poll_func_def.push_str(&format!("{indent3}//TODO handling of frame!\n"));


        poll_func_def.push_str(&format!("{indent3}break;\n"));
        
    }
    poll_func_def.push_str(&format!("{indent}}}\n"));

    poll_func_def.push_str("}\n");
    source.add_block(SourceBlock::new(SourceBlockIdentifier::Definition(poll_func_name.clone()),
            poll_func_def, vec![SourceBlockIdentifier::Declartion(poll_func_name.clone())]))?;
    


    Ok(())
}
