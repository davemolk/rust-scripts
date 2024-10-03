use super::{
    HashMap,
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub name: String,
    pub value: String,
}

impl Item {
    pub fn new(name: String, value: String) -> Self {
        Item { name, value }
    }
    pub fn short_name(&self) -> String {
        if self.name.len() > 15 { 
            format!("{}...", &self.name[..15])
        } else { 
            self.name.clone()
        }
    }
    pub fn url(&self) -> String {
        self.value.split_whitespace()
            .find(|v| v.starts_with("http://") || v.starts_with("https://"))
            .unwrap_or(&self.value)
            .to_string()
    }
    pub fn to_hash(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(self.name.clone(), self.value.clone());
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_creation() {
        let item = Item::new("foo".to_string(), "bar".to_string());
        assert_eq!(item.name, "foo".to_string());
        assert_eq!(item.value, "bar".to_string());
    }
    #[test]
    fn test_item_short_name() {
        let item = Item::new("this is a very long item name".to_string(), "value".to_string());
        assert_eq!(item.short_name(), "this is a very ...");
        let short_item = Item::new("short".to_string(), "value".to_string());
        assert_eq!(short_item.short_name(), "short");
    }
    #[test]
    fn test_item_to_hash() {
        let item = Item::new("foo".to_string(), "bar".to_string());
        let expected = HashMap::from([("foo".to_string(), "bar".to_string())]);
        assert_eq!(expected, item.to_hash());
    }
}