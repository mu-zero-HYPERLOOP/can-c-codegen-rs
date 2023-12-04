use crate::{options::Options, file_buffer::FileBuffer};

use crate::errors::Result;


pub fn generate_rx_queue(header : &mut FileBuffer, source : &mut FileBuffer, options : &Options) -> Result<()>{
       
    match options.platform() {
        crate::options::Platform::Linux => {
            
        }
    };

    Ok(())
}
