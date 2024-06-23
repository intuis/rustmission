use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::Result;
use toml::Table;

use crate::get_config_path;

pub fn fetch_config_table(config_name: &str) -> Result<Table> {
    let config_path = crate::xdg_dirs()
        .find_config_file(config_name)
        .ok_or_else(|| anyhow::anyhow!("{} not found", config_name))?;

    let mut config_buf = String::new();
    let mut config_file = File::open(config_path).unwrap();
    config_file.read_to_string(&mut config_buf).unwrap();

    Ok(toml::from_str(&config_buf)?)
}

pub fn put_config(content: &str, filename: &str) -> Result<Table> {
    let config_path = get_config_path(filename);
    let mut config_file = File::create(config_path)?;
    config_file.write_all(content.as_bytes())?;
    Ok(toml::from_str(content)?)
}
