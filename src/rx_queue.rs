use crate::source_block::{SourceBlock, SourceBlockIdentifier};
use crate::{file_buffer::FileBuffer, options::Options};

use crate::errors::Result;

pub fn generate_rx_queue(
    _header: &mut FileBuffer,
    source: &mut FileBuffer,
    options: &Options,
) -> Result<()> {
    let namespace = options.namespace();
    
    let queue_def = 
"typedef struct {
  
} rx_queue;\n";
    source.add_block(SourceBlock::new(SourceBlockIdentifier::Definition("rx_queue".to_owned()),
        queue_def.to_owned(),
        vec![]))?;

    let can_frame_type = format!("{namespace}_frame");
    let enqueue_def = format!(
"void rx_queue_enqueue(rx_queue* self, {can_frame_type}* frame) {{
  //TODO
}}\n");
    source.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition("rx_queue_enqueue".to_owned()),
        enqueue_def,
        vec![
            SourceBlockIdentifier::Definition("rx_queue".to_owned()),
            SourceBlockIdentifier::Definition(can_frame_type.clone()),
        ],
    ))?;

    let dequeue_def = format!(
"int rx_queue_dequeue(rx_queue* self, {can_frame_type}* frame) {{
  //TODO
  return 0;
}}\n");
    source.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition("rx_queue_dequeue".to_owned()),
        dequeue_def,
        vec![
            SourceBlockIdentifier::Definition("rx_queue".to_owned()),
            SourceBlockIdentifier::Definition(can_frame_type.clone()),
        ],
    ))?;
    let queue_var_def = "static rx_queue can0_rx_queue;\n";

    source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Definition("can0_rx_queue".to_owned()),
            queue_var_def.to_owned(),
            vec![SourceBlockIdentifier::Definition("rx_queue".to_owned())]))?;


    Ok(())
}
