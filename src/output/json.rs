use super::CommandOutput;

pub fn render(output: &CommandOutput) -> String {
    match output {
        CommandOutput::Table { headers, rows } => {
            let items: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let obj: serde_json::Map<String, serde_json::Value> = headers
                        .iter()
                        .zip(row.iter())
                        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                        .collect();
                    serde_json::Value::Object(obj)
                })
                .collect();
            serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
        }
        CommandOutput::Detail { fields } => {
            let obj: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            serde_json::to_string_pretty(&serde_json::Value::Object(obj))
                .unwrap_or_else(|_| "{}".to_string())
        }
        CommandOutput::Message(msg) => {
            serde_json::to_string_pretty(&serde_json::json!({ "message": msg }))
                .unwrap_or_else(|_| "{}".to_string())
        }
        CommandOutput::Empty => String::new(),
    }
}
