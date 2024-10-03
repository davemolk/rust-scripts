use crate::item::Item;
use super::{
    HashMap,
    Serialize,
    Deserialize,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct List {
    pub name: String,
    pub items: Vec<Item>,
}

impl List {
    pub fn new(name: String) -> Self {
        List {
            name,
            items: Vec::new(),
        }
    }
    pub fn add_item(&mut self, item: Item) {
        self.delete_item(&item.name);
        self.items.push(item)
    }
    pub fn find_item(&self, name: &str) -> Option<&Item> {
        self.items.iter().find(|i| {
            i.name == name || i.short_name().replace("...", "") == name
        })
    }
    pub fn delete_item(&mut self, name: &str) {
        self.items.retain(|i| i.name != name)
    }
    pub fn to_hash(&self) -> HashMap<String, Vec<HashMap<String, String>>> {
        let mut map = HashMap::new();
        map.insert(self.name.clone(), self.items.iter().map(|item| item.to_hash()).collect());
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_creation() {
        let list = List::new("foo".to_string());
        assert!(list.items.is_empty());
        assert_eq!(list.name, "foo".to_string());
    }
    #[test]
    fn test_add() {
        let mut list = List::new("foo".to_string());
        let item = Item::new("github".to_string(), "github.com/foo".to_string());
        list.add_item(item);
        assert_eq!(list.items.len(), 1);
        assert_eq!("github".to_string(), list.items[0].name);
    }
    #[test]
    fn test_add_overwrite() {
        let mut list = List::new("foo".to_string());
        let item = Item::new("github".to_string(), "github.com/foo".to_string());
        list.add_item(item);
        let item2 = Item::new("github".to_string(), "github.com/bar".to_string());
        list.add_item(item2);
        assert_eq!(list.items.len(), 1);
        assert_eq!("github".to_string(), list.items[0].name);
        assert_eq!("github.com/bar".to_string(), list.items[0].value);
    }
    #[test]
    fn test_delete() {
        let mut list = List::new("foo".to_string());
        let item = Item::new("github".to_string(), "github.com/foo".to_string());
        list.add_item(item);
        list.delete_item("github");
        assert!(list.items.is_empty());
        assert!(list.find_item("github").is_none());
    }
    #[test]
    fn test_find() {
        let mut list = List::new("foo".to_string());
        let item = Item::new("github".to_string(), "github.com/foo".to_string());
        list.add_item(item);
        assert!(list.find_item("github").is_some());
        assert!(list.find_item("blah").is_none());
    }
    #[test]
    fn test_find_short_name() {
        let mut list = List::new("foo".to_string());
        let item = Item::new("this is a very long item name".to_string(), "github.com/foo".to_string());
        list.add_item(item);
        assert!(list.find_item("this is a very ").is_some());
        assert!(list.find_item("blah").is_none());
    }
    #[test]
    fn test_to_hash() {
        let mut list = List::new("foo".to_string());
        let item = Item::new("key".to_string(), "value".to_string());
        list.add_item(item);
        let expected = HashMap::from([
            ("foo".to_string(), vec![HashMap::from([("key".to_string(), "value".to_string())])])
        ]);
        assert_eq!(list.to_hash(), expected);
    }
}