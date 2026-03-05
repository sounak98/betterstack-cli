use super::CommandOutput;

pub fn render(output: &CommandOutput) -> String {
    match output {
        CommandOutput::Table { headers, rows } => {
            let mut lines = vec![headers.join(",")];
            for row in rows {
                let escaped: Vec<String> = row
                    .iter()
                    .map(|v| {
                        if v.contains(',') || v.contains('"') || v.contains('\n') {
                            format!("\"{}\"", v.replace('"', "\"\""))
                        } else {
                            v.clone()
                        }
                    })
                    .collect();
                lines.push(escaped.join(","));
            }
            lines.join("\n")
        }
        CommandOutput::Detail { fields } => {
            let headers: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
            let values: Vec<&str> = fields.iter().map(|(_, v)| v.as_str()).collect();
            format!("{}\n{}", headers.join(","), values.join(","))
        }
        CommandOutput::Message(msg) => msg.clone(),
        CommandOutput::Raw(s) => s.clone(),
        CommandOutput::Empty => String::new(),
    }
}
