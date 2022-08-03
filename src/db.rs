use std::fs;
use rand::{distributions::Alphanumeric, prelude::*};

use tui::widgets::ListState;
use chrono::prelude::*;

mod model;

pub const ASSETS_DB_PATH: &str = "./data/assets_db.json";
pub const VULN_DB_PATH: &str = "./data/vuln_db.json";


pub fn read_db() -> Result<Vec<model::Asset>, model::Error> {
    let db_content = fs::read_to_string(ASSETS_DB_PATH)?;
    let parsed: Vec<model::Asset> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}


pub fn remove_asset_at_index(asset_list_state: &mut ListState) -> Result<(), model::Error> {
    if let Some(selected) = asset_list_state.selected() {
        let db_content = fs::read_to_string(ASSETS_DB_PATH)?;
        let mut parsed: Vec<model::Asset> = serde_json::from_str(&db_content)?;
        parsed.remove(selected);
        fs::write(ASSETS_DB_PATH, &serde_json::to_vec(&parsed)?)?;
        let _amount_assets = read_db().expect("can fetch asset list").len(); // remaining assets you're tracking
        if selected > 0 {
            asset_list_state.select(Some(selected - 1));
        } else {
            asset_list_state.select(Some(0));
        }
    }
    Ok(())
}



pub fn add_random_asset_to_db() -> Result<Vec<model::Asset>, model::Error> {
    let mut rng = rand::thread_rng();
    let db_content = fs::read_to_string(ASSETS_DB_PATH)?;
    let mut parsed: Vec<model::Asset> = serde_json::from_str(&db_content)?;
    let catsdogs = match rng.gen_range(0, 1) {
        0 => "hardware",
        _ => "software",
    };

    let random_asset = model::Asset {
        id: rng.gen_range(0, 9999999),
        name: rng.sample_iter(Alphanumeric).take(10).collect(),
        category: catsdogs.to_owned(),
        vulnerabilities: rng.gen_range(1, 15),
        created_at: Utc::now(),
    };
    
    parsed.push(random_asset);
    fs::write(ASSETS_DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}


