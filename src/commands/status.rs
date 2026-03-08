use anyhow::Result;

use crate::context::{AppContext, OutputFormat};
use crate::output::CommandOutput;
use crate::output::color;
use crate::output::fmt;
use crate::types::{HeartbeatResource, IncidentFilters, IncidentResource, MonitorResource};

#[derive(clap::Args)]
pub struct StatusCmd;

impl StatusCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let mon_filters = crate::types::MonitorFilters::default();
        let inc_filters = IncidentFilters::default();
        let (monitors, heartbeats, incidents) = tokio::try_join!(
            ctx.uptime.list_monitors(&mon_filters),
            ctx.uptime.list_heartbeats(),
            ctx.uptime.list_incidents(&inc_filters),
        )?;

        if ctx.global.output_format == OutputFormat::Table {
            Ok(CommandOutput::Raw(render_status(
                &monitors,
                &heartbeats,
                &incidents,
            )))
        } else {
            Ok(status_to_table(&monitors, &heartbeats, &incidents))
        }
    }
}

fn render_status(
    monitors: &[MonitorResource],
    heartbeats: &[HeartbeatResource],
    incidents: &[IncidentResource],
) -> String {
    let mut out = String::new();

    // Active incidents (not resolved)
    let active: Vec<&IncidentResource> = incidents
        .iter()
        .filter(|i| i.attributes.resolved_at.is_none())
        .collect();

    if !active.is_empty() {
        out.push_str(&format!("{}\n", color::bold("Active Incidents")));
        for i in &active {
            let a = &i.attributes;
            let name = a.name.as_deref().unwrap_or("Unnamed");
            let status = if a.acknowledged_at.is_some() {
                "acknowledged"
            } else {
                "started"
            };
            let since = a
                .started_at
                .as_deref()
                .map(fmt::relative_time)
                .unwrap_or_else(|| "-".to_string());
            out.push_str(&format!(
                "  {} {} {} {}\n",
                color::status(status),
                color::bold(name),
                color::dim(&format!("#{}", i.id)),
                color::dim(&since),
            ));
        }
        out.push('\n');
    }

    // Monitors summary
    let mon_up = monitors
        .iter()
        .filter(|m| m.attributes.status.as_deref() == Some("up"))
        .count();
    let mon_down = monitors
        .iter()
        .filter(|m| m.attributes.status.as_deref() == Some("down"))
        .count();
    let mon_paused = monitors
        .iter()
        .filter(|m| m.attributes.status.as_deref() == Some("paused"))
        .count();
    let mon_other = monitors.len() - mon_up - mon_down - mon_paused;

    out.push_str(&format!("{} ", color::bold("Monitors")));
    let mut parts = Vec::new();
    if mon_up > 0 {
        parts.push(color::green(&format!("{mon_up} up")));
    }
    if mon_down > 0 {
        parts.push(color::red(&format!("{mon_down} down")));
    }
    if mon_paused > 0 {
        parts.push(color::yellow(&format!("{mon_paused} paused")));
    }
    if mon_other > 0 {
        parts.push(color::dim(&format!("{mon_other} other")));
    }
    if parts.is_empty() {
        parts.push("none".to_string());
    }
    out.push_str(&parts.join(", "));
    out.push('\n');

    // Show down monitors
    for m in monitors
        .iter()
        .filter(|m| m.attributes.status.as_deref() == Some("down"))
    {
        let a = &m.attributes;
        let name = a.pronounceable_name.as_deref().unwrap_or("-");
        let url = a.url.as_deref().unwrap_or("");
        let last = a
            .last_checked_at
            .as_deref()
            .map(fmt::relative_time)
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!(
            "  {} {} {} {}\n",
            color::red("●"),
            name,
            color::dim(url),
            color::dim(&last),
        ));
    }

    // Heartbeats summary
    let hb_up = heartbeats
        .iter()
        .filter(|h| h.attributes.status.as_deref() == Some("up"))
        .count();
    let hb_down = heartbeats
        .iter()
        .filter(|h| h.attributes.status.as_deref() == Some("down"))
        .count();
    let hb_paused = heartbeats
        .iter()
        .filter(|h| h.attributes.status.as_deref() == Some("paused"))
        .count();
    let hb_other = heartbeats.len() - hb_up - hb_down - hb_paused;

    out.push_str(&format!("\n{} ", color::bold("Heartbeats")));
    let mut parts = Vec::new();
    if hb_up > 0 {
        parts.push(color::green(&format!("{hb_up} up")));
    }
    if hb_down > 0 {
        parts.push(color::red(&format!("{hb_down} down")));
    }
    if hb_paused > 0 {
        parts.push(color::yellow(&format!("{hb_paused} paused")));
    }
    if hb_other > 0 {
        parts.push(color::dim(&format!("{hb_other} other")));
    }
    if parts.is_empty() {
        parts.push("none".to_string());
    }
    out.push_str(&parts.join(", "));
    out.push('\n');

    // Show down heartbeats
    for h in heartbeats
        .iter()
        .filter(|h| h.attributes.status.as_deref() == Some("down"))
    {
        let name = h.attributes.name.as_deref().unwrap_or("-");
        out.push_str(&format!("  {} {}\n", color::red("●"), name));
    }

    out.trim_end().to_string()
}

/// Structured table for JSON/CSV output: one row per resource with type and status.
fn status_to_table(
    monitors: &[MonitorResource],
    heartbeats: &[HeartbeatResource],
    incidents: &[IncidentResource],
) -> CommandOutput {
    let headers = vec![
        "Type".to_string(),
        "ID".to_string(),
        "Name".to_string(),
        "Status".to_string(),
    ];
    let mut rows: Vec<Vec<String>> = Vec::new();
    for m in monitors {
        rows.push(vec![
            "monitor".to_string(),
            m.id.clone(),
            m.attributes
                .pronounceable_name
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            m.attributes
                .status
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        ]);
    }
    for h in heartbeats {
        rows.push(vec![
            "heartbeat".to_string(),
            h.id.clone(),
            h.attributes.name.clone().unwrap_or_else(|| "-".to_string()),
            h.attributes
                .status
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        ]);
    }
    for i in incidents
        .iter()
        .filter(|i| i.attributes.resolved_at.is_none())
    {
        let status = if i.attributes.acknowledged_at.is_some() {
            "acknowledged"
        } else {
            "started"
        };
        rows.push(vec![
            "incident".to_string(),
            i.id.clone(),
            i.attributes.name.clone().unwrap_or_else(|| "-".to_string()),
            status.to_string(),
        ]);
    }
    CommandOutput::Table { headers, rows }
}
