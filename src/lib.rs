use can_config_rs::config;
use can_module_hooks::generate_hooks;
use command_hooks::generate_command_hooks;
use file_buffer::FileBuffer;
use options::Options;
use pil::generate_pil;
use rx_handlers::generate_rx_handlers;
use types::generate_types;
use errors::{Error, Result};
use update::generate_update;

use crate::{object_entries::generate_object_entries, messages::generate_messages, poll::generate_poll, rx_queue::generate_rx_queue};

pub mod errors;
pub mod options;
mod pil;
mod file_buffer;
mod source_block;
mod types;
mod object_entries;
mod messages;
mod rx_queue;
mod poll;
mod can_module_hooks;
mod rx_handlers;
mod command_hooks;
mod update;


pub fn generate(node_name : &str, network_config : config::NetworkRef, options : Options) -> Result<()> {
    let Some(node_config) = network_config.nodes().iter().find(|n| n.name() == node_name) else {
        return Err(Error::InvalidNodeName);
    };

    // TODO setup paths relativ to the workspace directory!
    
    let mut src = FileBuffer::new(options.source_file_path());
    let mut header = FileBuffer::new(options.header_file_path());


    generate_pil(&mut src, &mut header, &options)?;
    generate_hooks(network_config.buses(), &mut src, &mut header, &options)?;
    generate_types(node_config, &mut header, &options)?;
    generate_command_hooks(node_config.commands(), &mut src, &mut header, &options)?;
    generate_object_entries(node_config.object_entries(), &mut header, &mut src, &options)?;
    generate_messages(node_config.tx_messages(), node_config.rx_messages(), &mut header, &mut src, &options)?;
    // generate_rx_queue(&mut header, &mut src, &options)?;
    generate_rx_handlers(&network_config, node_config, &mut src, &mut header, &options)?;
    generate_poll(node_config, network_config.buses(), &mut header, &mut src, &options)?;
    generate_update(&mut src, &mut header, &options)?;


    src.include_file_buffer(&header);

    header.write(Some("CANZERO_H".to_owned())).unwrap();
    src.write(None).unwrap();
    // println!("HEADER:");
    // println!("{header:?}");
    // println!("SOURCE:");
    // println!("{src:?}");

    Ok(())
}
