use serde_json::Value;
use crate::error;
use crate::container::error::error::Stop;

pub fn parse_path(json: &Value) -> Result<Vec<String>, Stop> {
    let path: Result<Vec<String>, Stop> = json
        .as_array()
        .ok_or(error!(
            format!("Path is not of type array: {:?}.\nIf you're referencing a global block, make sure it is present in BuildSource.", json),
            "Parse path".to_string()
        ))?
        .iter()
        .map(|v| {
            v.as_str()
                .map_or_else(|| Err(error!(format!("Invalid string in array: {:?}", v))), |v| Ok(v.to_string()))
        })
        .collect();
    path
}