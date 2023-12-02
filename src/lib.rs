use can_config_rs::config;
use file_buffer::FileBuffer;
use options::Options;
use types::generate_types;
use errors::{Error, Result};

use crate::{object_entries::generate_object_entries, messages::generate_messages};

pub mod errors;
pub mod options;
mod file_buffer;
mod source_block;
mod types;
mod object_entries;
mod messages;


pub fn generate(node_name : &str, network_config : config::NetworkRef, options : Options) -> Result<()> {
    let Some(node_config) = network_config.nodes().iter().find(|n| n.name() == node_name) else {
        return Err(Error::InvalidNodeName);
    };
    
    let mut src = FileBuffer::new(options.source_file_path());
    let mut header = FileBuffer::new(options.header_file_path());

    generate_types(node_config, &mut header, &options)?;
    generate_object_entries(node_config.object_entries(), &mut header, &mut src, &options)?;
    generate_messages(node_config.tx_messages(), node_config.rx_messages(), &mut header, &mut src, &options)?;


    println!("HEADER:");
    println!("{header:?}");
    println!("SOURCE:");
    println!("{src:?}");

    Ok(())
}
