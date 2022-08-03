use std::fs::File;


pub async fn init() {
    let _assets_db = File::create("data/db.json");
    let _db = File::create("data/vuln_db.json");
}

