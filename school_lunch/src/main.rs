mod client;

use crate::{
    client::LunchClient,
    client::FoodApi,
};

use anyhow::Result;

fn main() -> Result<()> {
    let client = LunchClient::new();
    let url = client.get_url()?;
    let menu_options = client.get_lunch(url)?; 
    for option in menu_options.entree {
        println!("{}", option.menu_item_description);
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::LunchResponse;
    #[test]
    fn get_lunch() {
        struct ClientMock{}
        impl FoodApi for ClientMock {
            fn get_lunch(&self, _url: String) -> Result<LunchResponse> {
                let data = std::fs::read_to_string("menu_response.json").expect("failed to read test data");
                let resp: LunchResponse = serde_json::from_str(&data)?;

                Ok(resp)
            }
        }
        let client = ClientMock {};
        let resp = client.get_lunch("blah".to_string()).expect("get lunch failed");
        assert_eq!(resp.entree[0].menu_item_description, "Yogurt Basket with Fresh Baked Blueberry Muffin");
    }
}
