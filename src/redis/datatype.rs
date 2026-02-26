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

    pub fn from_char(c: char) -> Self {
        match c {
            's' => DataType::String,
            'h' => DataType::Hash,
            'l' => DataType::List,
            'e' => DataType::Set,
            'z' => DataType::Zset,
            _ => DataType::None,
        }
    }

    pub fn from_str(c: &str) -> Self {
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
