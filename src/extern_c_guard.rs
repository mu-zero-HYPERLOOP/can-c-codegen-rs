use crate::errors::Result;

pub fn generate_extern_guard_top(header : &mut String) -> Result<()> {
    header.push_str("#ifdef __cplusplus
extern \"C\" {
#endif\n");
    Ok(())
}

pub fn generate_extern_guard_bottom(header : &mut String) -> Result<()> {
    header.push_str("#ifdef __cplusplus
}
#endif\n");
    Ok(())
}
