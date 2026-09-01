#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionQuality {
    High,
    Medium,
    Low,
}

impl CompressionQuality {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            _ => None,
        }
    }
}
