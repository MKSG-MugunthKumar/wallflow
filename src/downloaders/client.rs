use crate::config::AdvancedConfig;

pub struct WallflowClient {
  client: reqwest::Client,
}

impl WallflowClient {
  /// Create a new Wallflow HTTP client
  pub fn from(config: &AdvancedConfig) -> Self {
    Self {
      client: reqwest::Client::builder()
        .user_agent(config.user_agent.clone())
        .build()
        .expect("Failed to build HTTP client"),
    }
  }

  pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
    self.client.get(url)
  }
}
