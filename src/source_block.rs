

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceBlockIdentifier {
    Import(String),
    Declartion(String),
    Definition(String),
}

#[derive(Clone)]
pub struct SourceBlock {
    id : SourceBlockIdentifier,
    content : String,
    dependencies : Vec<SourceBlockIdentifier>,
}

impl SourceBlock { 

    pub fn new(id : SourceBlockIdentifier, content : String, 
           dependencies : Vec<SourceBlockIdentifier>) -> Self {
        Self {
            id,
            content,
            dependencies,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn dependencies(&self) -> &Vec<SourceBlockIdentifier> {
        &self.dependencies
    }
    pub fn id(&self) -> &SourceBlockIdentifier {
        &self.id
    }
}
