
use crate::source_block::SourceBlock;
use std::fmt::Debug;

use crate::errors::Result;
use crate::errors::Error;

pub struct FileBuffer {
    blocks : Vec<SourceBlock>,
    path : String,
}

impl FileBuffer {

    pub fn new(path : &str) -> Self {
        Self {
            blocks : vec![],
            path : path.to_owned(),
        }
    }

    pub fn add_block(&mut self, block : SourceBlock) -> Result<()> {
        let allow_dup = match block.id() {
            crate::source_block::SourceBlockIdentifier::Import(_) => true,
            crate::source_block::SourceBlockIdentifier::Declartion(_) => false,
            crate::source_block::SourceBlockIdentifier::Definition(_) => false,
        };
        if self.blocks.iter().any(|b| b.id() == block.id()) {
            if !allow_dup {
                return Err(Error::DuplicatedBlockIdentifier);
            }else {
                Ok(())
            }
        }else {
            self.blocks.push(block);
            Ok(())
        }
    }

    pub fn write(&self) -> Result<()> {

        // TODO sort blocks topological by dependencies!
        // TODO check that all dependencies actually exist

        let mut content = String::new();
        for block in &self.blocks {
            content.push_str(block.content());
        }
    
        
        std::fs::write(&self.path, &content)?;
        Ok(())
    }
}

impl Debug for FileBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("FILE = {}\n", self.path))?;
        for block in &self.blocks {
            f.write_str(&format!("//{:?}\n", block.id()))?;
            f.write_str(block.content())?;
        }
        Ok(())
    }
}
