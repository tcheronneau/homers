use reqwest::Client;
pub struct Plex {
    name: String,
    address: String,
    port: u16,
    token: String,
}
impl Plex {
    pub fn new(name: String, address: String, port: u16, token: String) -> Plex {
        Plex {
            name,
            address,
            port,
            token,
        }
    }
    pub async fn get(&self, path: &str) -> String {
        let url = format!("http://{}:{}/{}", self.address, self.port, path);
        let res = Client::new()
            .get(&url)
            .header("Accept", "application/json")
            .header("X-Plex-Token", &self.token)
            .send()
            .await;
                
        match res {
            Ok(response) => {
                let text = response.text().await.unwrap();
                text
            }
            Err(e) => {
                format!("Error: {}", e)
            }
        }
    }
}
