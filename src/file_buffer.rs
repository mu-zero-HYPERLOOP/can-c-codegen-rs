
use crate::source_block::SourceBlock;
use crate::source_block::SourceBlockIdentifier;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;

use crate::errors::Result;
use crate::errors::Error;


#[derive(Clone)]
pub struct FileBuffer {
    blocks : Rc<RefCell<Vec<SourceBlock>>>,
    path : String,
    includes : Rc<RefCell<Vec<FileBuffer>>>,
}

impl FileBuffer {

    pub fn new(path : &str) -> Self {
        Self {
            blocks : Rc::new(RefCell::new(vec![])),
            path : path.to_owned(),
            includes : Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn add_block(&self, block : SourceBlock) -> Result<()> {
        let allow_dup = match block.id() {
            crate::source_block::SourceBlockIdentifier::Import(_) => true,
            crate::source_block::SourceBlockIdentifier::Declartion(_) => false,
            crate::source_block::SourceBlockIdentifier::Definition(_) => false,
        };
        if self.blocks.borrow().iter().any(|b| b.id() == block.id()) {
            if !allow_dup {
                return Err(Error::DuplicatedBlockIdentifier);
            }else {
                Ok(())
            }
        }else {
            self.blocks.borrow_mut().push(block);
            Ok(())
        }
    }

    pub fn include_file_buffer(&self, buffer: &FileBuffer) {
        self.includes.borrow_mut().push(buffer.clone());
    }

    pub fn write(&self, header_guard : Option<String>) -> Result<()> {
        let mut content = String::new();
        let mut defined_blocks = HashSet::new();
        fn collect_defined_blocks(defined_blocks : &mut HashSet<SourceBlockIdentifier>, include : &FileBuffer, included_files : &mut Vec<String>) {
            included_files.push(include.path.clone());
            for inc in include.includes.borrow().iter() {
                if !included_files.contains(&inc.path) {
                    collect_defined_blocks(defined_blocks, inc, included_files);
                }
            }
            for block in include.blocks.borrow().iter() {
                defined_blocks.insert(block.id().clone());
            }
        }
        // find all required imports!
        collect_defined_blocks(&mut defined_blocks, self, &mut vec![]);
        for inc in self.includes.borrow().iter() {
            content.push_str(&format!("#include \"{}\"\n", inc.path));
        }
        let mut includes = HashSet::new();
        for block in self.blocks.borrow().iter() {
            for dep in block.dependencies() {
                match dep {
                    SourceBlockIdentifier::Import(path) => {
                        includes.insert(path.clone());
                    },
                    _ => (),
                }
            }
        }
        let header_guard = if header_guard.is_some() {
            let guard = header_guard.unwrap();
            content += &format!(
"#ifndef {}
#define {}
#ifdef __cplusplus
extern \"C\" {{
#endif\n", guard, guard);
            true
        }else {false};

        for inc in includes {
            content.push_str(&format!("#include \"{}\"\n", inc));
        }

        

        // TODO sort blocks topological by dependencies!
        

        // TODO check that all dependencies actually exist

        for block in self.blocks.borrow().iter() {
            content.push_str(block.content());
        }

        if header_guard {
            content += &format!(
"
#ifdef __cplusplus 
}}
#endif
#endif
");
        }
        
        std::fs::write(&self.path, &content)?;
        Ok(())
    }
}

impl Debug for FileBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("FILE = {}\n", self.path))?;
        for block in self.blocks.borrow().iter() {
            f.write_str(&format!("//{:?}\n", block.id()))?;
            f.write_str(block.content())?;
        }
        Ok(())
    }
}
