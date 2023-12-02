
pub enum Platform {
    Linux,
}

pub struct Options {
    source_file_path : String,
    header_file_path : String,
    platform : Platform,
    ident : usize,
    namespace : String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            source_file_path : "canzero.c".to_owned(),
            header_file_path : "canzero.h".to_owned(),
            platform : Platform::Linux,
            ident : 2,
            namespace : "can".to_owned(),
        }
    }
}

impl Options {
    pub fn source_file_path(&self) -> &str {
        &self.source_file_path
    }
    pub fn header_file_path(&self) -> &str {
        &self.header_file_path
    }
    pub fn platform(&self) -> &Platform {
        &self.platform
    }
    pub fn indent(&self) -> usize {
        self.ident
    }
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}
