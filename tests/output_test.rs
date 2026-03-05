use bs_cli::context::OutputFormat;
use bs_cli::output::{self, CommandOutput};

#[test]
fn table_renders_monitors() {
    let output = CommandOutput::Table {
        headers: vec!["ID".into(), "Name".into(), "Status".into()],
        rows: vec![
            vec!["1".into(), "Example".into(), "up".into()],
            vec!["2".into(), "API".into(), "down".into()],
        ],
    };

    let rendered = output::render(&output, OutputFormat::Table, true);
    assert!(rendered.contains("ID"));
    assert!(rendered.contains("Example"));
    assert!(rendered.contains("up"));
    assert!(rendered.contains("API"));
    assert!(rendered.contains("down"));
}

#[test]
fn table_empty_shows_message() {
    let output = CommandOutput::Table {
        headers: vec!["ID".into()],
        rows: vec![],
    };

    let rendered = output::render(&output, OutputFormat::Table, true);
    assert_eq!(rendered, "No results found.");
}

#[test]
fn json_renders_array_of_objects() {
    let output = CommandOutput::Table {
        headers: vec!["ID".into(), "Name".into()],
        rows: vec![vec!["1".into(), "Test".into()]],
    };

    let rendered = output::render(&output, OutputFormat::Json, true);
    let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();

    assert!(parsed.is_array());
    assert_eq!(parsed[0]["ID"], "1");
    assert_eq!(parsed[0]["Name"], "Test");
}

#[test]
fn json_detail_renders_object() {
    let output = CommandOutput::Detail {
        fields: vec![
            ("ID".into(), "123".into()),
            ("Name".into(), "My Monitor".into()),
        ],
    };

    let rendered = output::render(&output, OutputFormat::Json, true);
    let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();

    assert_eq!(parsed["ID"], "123");
    assert_eq!(parsed["Name"], "My Monitor");
}

#[test]
fn json_message_wraps_in_object() {
    let output = CommandOutput::Message("Monitor deleted.".into());

    let rendered = output::render(&output, OutputFormat::Json, true);
    let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();

    assert_eq!(parsed["message"], "Monitor deleted.");
}

#[test]
fn csv_renders_with_headers() {
    let output = CommandOutput::Table {
        headers: vec!["ID".into(), "Name".into()],
        rows: vec![
            vec!["1".into(), "Test".into()],
            vec!["2".into(), "Other".into()],
        ],
    };

    let rendered = output::render(&output, OutputFormat::Csv, true);
    let lines: Vec<&str> = rendered.lines().collect();

    assert_eq!(lines[0], "ID,Name");
    assert_eq!(lines[1], "1,Test");
    assert_eq!(lines[2], "2,Other");
}

#[test]
fn csv_escapes_commas() {
    let output = CommandOutput::Table {
        headers: vec!["Name".into()],
        rows: vec![vec!["hello, world".into()]],
    };

    let rendered = output::render(&output, OutputFormat::Csv, true);
    assert!(rendered.contains("\"hello, world\""));
}

#[test]
fn detail_table_renders_key_value() {
    let output = CommandOutput::Detail {
        fields: vec![("ID".into(), "123".into()), ("Status".into(), "up".into())],
    };

    let rendered = output::render(&output, OutputFormat::Table, true);
    assert!(rendered.contains("ID"));
    assert!(rendered.contains("123"));
    assert!(rendered.contains("Status"));
    assert!(rendered.contains("up"));
}
