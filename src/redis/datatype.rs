#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DataType {
    #[default]
    None,
    String,
    Hash,
    List,
    Set,
    Zset,
}

impl DataType {
    pub fn as_str(&self) -> &str {
        match self {
            DataType::String => "string",
            DataType::Hash => "hash",
            DataType::List => "list",
            DataType::Set => "set",
            DataType::Zset => "zset",
            DataType::None => "",
        }
    }
}

impl From<char> for DataType {
    fn from(c: char) -> Self {
        // Type selection - keep it simple: s: string, h: hash, l: list, e: set, z: zset

        match c {
            's' => DataType::String,
            'h' => DataType::Hash,
            'l' => DataType::List,
            'e' => DataType::Set,
            'z' => DataType::Zset,
            _ => DataType::None,
        }
    }
}

impl From<&str> for DataType {
    fn from(c: &str) -> Self {
        // Type selection - keep it simple: s: string, h: hash, l: list, e: set, z: zset

        match c.to_lowercase().as_str() {
            "string" => DataType::String,
            "hash" => DataType::Hash,
            "list" => DataType::List,
            "set" => DataType::Set,
            "zset" => DataType::Zset,
            _ => DataType::None,
        }
    }
}
