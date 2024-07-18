use htmlescape::decode_html;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct Map {
    character_map: HashMap<String, String>,
    post_rules: Vec<Vec<String>>,
}

pub fn preeti_to_unicode(input: String) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(std::env::current_dir()?.join("src").join("preeti.json"));
    let map_string = fs::read_to_string(&file_path).unwrap();
    let rules: Map = serde_json::from_str(&map_string)?;

    //normalise html entities
    let normalised_input = decode_html(&input).unwrap_or(input);

    //convert
    let mut res = String::new();
    for i in normalised_input.split("") {
        res.push_str(rules.character_map.get(i).unwrap_or(&"".to_owned()));
    }

    //post rules
    for i in &rules.post_rules {
        let re = Regex::new(&i[0])?;
        res = re.replace_all(&res, &i[1]).to_string();
    }

    Ok(res)
}
