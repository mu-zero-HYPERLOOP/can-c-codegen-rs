use crate::source_block::{SourceBlock, SourceBlockIdentifier};
use crate::{file_buffer::FileBuffer, options::Options};

use crate::errors::Result;

pub fn generate_rx_queue(
    _header: &mut FileBuffer,
    source: &mut FileBuffer,
    _options: &Options,
) -> Result<()> {
    
    let queue_def = 
"typedef struct {
  
} rx_queue;\n";
    source.add_block(SourceBlock::new(SourceBlockIdentifier::Definition("rx_queue".to_owned()),
        queue_def.to_owned(),
        vec![]))?;

    let enqueue_def = 
"void rx_queue_enqueue(rx_queue* self, can_frame* frame) {
  //TODO
}\n";
    source.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition("rx_queue_enqueue".to_owned()),
        enqueue_def.to_owned(),
        vec![
            SourceBlockIdentifier::Definition("rx_queue".to_owned()),
            SourceBlockIdentifier::Definition("can_frame".to_owned()),
        ],
    ))?;

    let dequeue_def = 
"int rx_queue_dequeue(rx_queue* self, can_frame* frame) {
  //TODO
  return 0;
}\n";
    source.add_block(SourceBlock::new(
        SourceBlockIdentifier::Definition("rx_queue_dequeue".to_owned()),
        dequeue_def.to_owned(),
        vec![
            SourceBlockIdentifier::Definition("rx_queue".to_owned()),
            SourceBlockIdentifier::Definition("can_frame".to_owned()),
        ],
    ))?;
    let queue_var_def = "static rx_queue can0_rx_queue;\n";

    source.add_block(SourceBlock::new(
            SourceBlockIdentifier::Definition("can0_rx_queue".to_owned()),
            queue_var_def.to_owned(),
            vec![SourceBlockIdentifier::Definition("rx_queue".to_owned())]))?;


    Ok(())
}
