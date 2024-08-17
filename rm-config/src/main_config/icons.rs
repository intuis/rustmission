use serde::Deserialize;

#[derive(Deserialize)]
pub struct Icons {
    #[serde(default = "default_upload")]
    pub upload: String,
    #[serde(default = "default_download")]
    pub download: String,
    #[serde(default = "default_arrow_left")]
    pub arrow_left: String,
    #[serde(default = "default_arrow_right")]
    pub arrow_right: String,
    #[serde(default = "default_arrow_up")]
    pub arrow_up: String,
    #[serde(default = "default_arrow_down")]
    pub arrow_down: String,
    #[serde(default = "default_triangle_right")]
    pub triangle_right: String,
    #[serde(default = "default_triangle_down")]
    pub triangle_down: String,
    #[serde(default = "default_file")]
    pub file: String,
    #[serde(default = "default_disk")]
    pub disk: String,
    #[serde(default = "default_help")]
    pub help: String,
    #[serde(default = "default_success")]
    pub success: String,
    #[serde(default = "default_failure")]
    pub failure: String,
    #[serde(default = "default_searching")]
    pub searching: String,
    #[serde(default = "default_verifying")]
    pub verifying: String,
    #[serde(default = "default_loading")]
    pub loading: String,
    #[serde(default = "default_pause")]
    pub pause: String,
    #[serde(default = "default_idle")]
    pub idle: String,
    #[serde(default = "default_magnifying_glass")]
    pub magnifying_glass: String,
    #[serde(default = "default_provider_disabled")]
    pub provider_disabled: String,
    #[serde(default = "default_provider_category_general")]
    pub provider_category_general: String,
    #[serde(default = "default_provider_category_anime")]
    pub provider_category_anime: String,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            upload: default_upload(),
            download: default_download(),
            arrow_left: default_arrow_left(),
            arrow_right: default_arrow_right(),
            arrow_up: default_arrow_up(),
            arrow_down: default_arrow_down(),
            triangle_right: default_triangle_right(),
            triangle_down: default_triangle_down(),
            file: default_file(),
            disk: default_disk(),
            help: default_help(),
            success: default_success(),
            failure: default_failure(),
            searching: default_searching(),
            verifying: default_verifying(),
            loading: default_loading(),
            pause: default_pause(),
            idle: default_idle(),
            magnifying_glass: default_magnifying_glass(),
            provider_disabled: default_provider_disabled(),
            provider_category_general: default_provider_category_general(),
            provider_category_anime: default_provider_category_anime(),
        }
    }
}
fn default_upload() -> String {
    "".into()
}

fn default_download() -> String {
    "".into()
}

fn default_arrow_left() -> String {
    "".into()
}

fn default_arrow_right() -> String {
    "".into()
}

fn default_arrow_up() -> String {
    "".into()
}

fn default_arrow_down() -> String {
    "".into()
}

fn default_triangle_right() -> String {
    "▶".into()
}

fn default_triangle_down() -> String {
    "▼".into()
}

fn default_file() -> String {
    "".into()
}

fn default_disk() -> String {
    "󰋊".into()
}

fn default_help() -> String {
    "󰘥".into()
}

fn default_success() -> String {
    "".into()
}

fn default_failure() -> String {
    "".into()
}

fn default_searching() -> String {
    "".into()
}

fn default_verifying() -> String {
    "󰑓".into()
}

fn default_loading() -> String {
    "󱥸".into()
}

fn default_pause() -> String {
    "󰏤".into()
}

fn default_idle() -> String {
    "󱗼".into()
}

fn default_magnifying_glass() -> String {
    "".into()
}

fn default_provider_disabled() -> String {
    "󰪎".into()
}

fn default_provider_category_general() -> String {
    "".into()
}

fn default_provider_category_anime() -> String {
    "󰎁".into()
}
