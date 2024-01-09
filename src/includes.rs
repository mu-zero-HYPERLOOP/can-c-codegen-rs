use std::str::FromStr;

use crate::{errors::Result, options::Options};

pub fn generate_includes(
    source: &mut String,
    header: &mut String,
    options: &Options,
) -> Result<()> {
    let header_path = options.header_file_path();
    let header_name = std::path::PathBuf::from_str(header_path)
        .expect(&format!("{header} is not a file"))
        .file_name()
        .expect(&format!("{header} is not a file"))
        .to_str()
        .unwrap()
        .to_owned();
    source.push_str(&format!("#include \"{header_name}\"\n"));
    
    header.push_str("#include \"inttypes.h\"\n");
    header.push_str("#include \"stddef.h\"\n");

    Ok(())
}
