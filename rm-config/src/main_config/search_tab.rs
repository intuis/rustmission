use magnetease::WhichProvider;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchTab {
    #[serde(default = "default_providers")]
    pub providers: Vec<WhichProvider>,
}

impl Default for SearchTab {
    fn default() -> Self {
        Self {
            providers: default_providers(),
        }
    }
}

fn default_providers() -> Vec<WhichProvider> {
    vec![WhichProvider::Knaben, WhichProvider::Nyaa]
}
