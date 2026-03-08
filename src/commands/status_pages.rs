use anyhow::Result;

use crate::context::{AppContext, OutputFormat};
use crate::output::CommandOutput;
use crate::output::color;
use crate::output::fmt;
use crate::types::*;

#[derive(clap::Args)]
pub struct StatusPagesCmd {
    #[command(subcommand)]
    command: Option<StatusPagesSubCmd>,
}

#[derive(clap::Subcommand)]
enum StatusPagesSubCmd {
    /// List all status pages.
    List,
    /// Get details of a status page.
    #[command(arg_required_else_help = true)]
    Get {
        /// Status page ID.
        id: String,
    },
    /// Create a new status page.
    #[command(arg_required_else_help = true)]
    Create {
        /// Company/page name.
        #[arg(long)]
        name: String,
        /// Subdomain for the status page URL.
        #[arg(long)]
        subdomain: String,
        /// Company URL.
        #[arg(long)]
        url: Option<String>,
        /// Custom domain.
        #[arg(long)]
        domain: Option<String>,
        /// Timezone (e.g. "UTC", "America/New_York").
        #[arg(long)]
        timezone: Option<String>,
        /// Theme: light or dark.
        #[arg(long)]
        theme: Option<String>,
        /// Enable subscriber notifications.
        #[arg(long)]
        subscribable: bool,
    },
    /// Update a status page.
    #[command(arg_required_else_help = true)]
    Update {
        /// Status page ID.
        id: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// New subdomain.
        #[arg(long)]
        subdomain: Option<String>,
        /// Company URL.
        #[arg(long)]
        url: Option<String>,
        /// Custom domain.
        #[arg(long)]
        domain: Option<String>,
        /// Timezone.
        #[arg(long)]
        timezone: Option<String>,
        /// Theme: light or dark.
        #[arg(long)]
        theme: Option<String>,
        /// Enable/disable subscriber notifications.
        #[arg(long)]
        subscribable: Option<bool>,
    },
    /// Delete a status page.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Status page ID.
        id: String,
    },
    /// Manage sections on a status page.
    #[command(arg_required_else_help = true)]
    Sections(SectionsCmd),
    /// Manage resources on a status page.
    #[command(arg_required_else_help = true)]
    Resources(ResourcesCmd),
    /// Manage reports on a status page.
    #[command(arg_required_else_help = true)]
    Reports(ReportsCmd),
}

// --- Sections subcommand ---

#[derive(clap::Args)]
struct SectionsCmd {
    #[command(subcommand)]
    command: SectionsSubCmd,
}

#[derive(clap::Subcommand)]
enum SectionsSubCmd {
    /// List sections on a status page.
    #[command(arg_required_else_help = true)]
    List {
        /// Status page ID.
        page_id: String,
    },
    /// Get a section.
    #[command(arg_required_else_help = true)]
    Get {
        /// Status page ID.
        page_id: String,
        /// Section ID.
        section_id: String,
    },
    /// Create a section.
    #[command(arg_required_else_help = true)]
    Create {
        /// Status page ID.
        page_id: String,
        /// Section name.
        #[arg(long)]
        name: String,
        /// Display position.
        #[arg(long)]
        position: Option<u64>,
    },
    /// Update a section.
    #[command(arg_required_else_help = true)]
    Update {
        /// Status page ID.
        page_id: String,
        /// Section ID.
        section_id: String,
        /// New name.
        #[arg(long)]
        name: Option<String>,
        /// New position.
        #[arg(long)]
        position: Option<u64>,
    },
    /// Delete a section.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Status page ID.
        page_id: String,
        /// Section ID.
        section_id: String,
    },
}

// --- Resources subcommand ---

#[derive(clap::Args)]
struct ResourcesCmd {
    #[command(subcommand)]
    command: ResourcesSubCmd,
}

#[derive(clap::Subcommand)]
enum ResourcesSubCmd {
    /// List resources on a status page.
    #[command(arg_required_else_help = true)]
    List {
        /// Status page ID.
        page_id: String,
    },
    /// Get a resource.
    #[command(arg_required_else_help = true)]
    Get {
        /// Status page ID.
        page_id: String,
        /// Resource ID.
        resource_id: String,
    },
    /// Add a resource to a status page.
    #[command(arg_required_else_help = true)]
    Create {
        /// Status page ID.
        page_id: String,
        /// Monitor/heartbeat ID to add.
        #[arg(long)]
        resource_id: u64,
        /// Resource type (e.g. "Monitor", "Heartbeat").
        #[arg(long, default_value = "Monitor")]
        resource_type: String,
        /// Public display name.
        #[arg(long)]
        public_name: Option<String>,
        /// Section ID to place in.
        #[arg(long)]
        section: Option<u64>,
        /// Widget type (e.g. "history").
        #[arg(long)]
        widget: Option<String>,
    },
    /// Update a resource on a status page.
    #[command(arg_required_else_help = true)]
    Update {
        /// Status page ID.
        page_id: String,
        /// Resource ID.
        resource_id: String,
        /// New public display name.
        #[arg(long)]
        public_name: Option<String>,
        /// Move to section.
        #[arg(long)]
        section: Option<u64>,
        /// Widget type.
        #[arg(long)]
        widget: Option<String>,
        /// Display position.
        #[arg(long)]
        position: Option<u64>,
    },
    /// Remove a resource from a status page.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Status page ID.
        page_id: String,
        /// Resource ID.
        resource_id: String,
    },
}

// --- Reports subcommand ---

#[derive(clap::Args)]
struct ReportsCmd {
    #[command(subcommand)]
    command: ReportsSubCmd,
}

#[derive(clap::Subcommand)]
enum ReportsSubCmd {
    /// List reports on a status page.
    #[command(arg_required_else_help = true)]
    List {
        /// Status page ID.
        page_id: String,
    },
    /// Get a report.
    #[command(arg_required_else_help = true)]
    Get {
        /// Status page ID.
        page_id: String,
        /// Report ID.
        report_id: String,
    },
    /// Create a report (incident or maintenance).
    #[command(arg_required_else_help = true)]
    Create {
        /// Status page ID.
        page_id: String,
        /// Report title.
        #[arg(long)]
        title: String,
        /// Initial update message (required by the API).
        #[arg(long)]
        message: String,
        /// Report type: manual or maintenance.
        #[arg(long, default_value = "manual")]
        report_type: String,
        /// Affected resources as JSON array, e.g. '[{"status_page_resource_id":"123","status":"degraded"}]'.
        #[arg(long)]
        affected: Option<String>,
        /// Start time (ISO 8601). Defaults to now.
        #[arg(long)]
        starts_at: Option<String>,
        /// End time (ISO 8601).
        #[arg(long)]
        ends_at: Option<String>,
    },
    /// Update a report.
    #[command(arg_required_else_help = true)]
    Update {
        /// Status page ID.
        page_id: String,
        /// Report ID.
        report_id: String,
        /// New title.
        #[arg(long)]
        title: Option<String>,
        /// End time (ISO 8601). Set to close the report.
        #[arg(long)]
        ends_at: Option<String>,
    },
    /// Delete a report.
    #[command(arg_required_else_help = true)]
    Delete {
        /// Status page ID.
        page_id: String,
        /// Report ID.
        report_id: String,
    },
    /// Post an update to a report.
    #[command(name = "add-update", arg_required_else_help = true)]
    AddUpdate {
        /// Status page ID.
        page_id: String,
        /// Report ID.
        report_id: String,
        /// Update message.
        #[arg(long)]
        message: String,
        /// Affected resources JSON (auto-carried from report if omitted).
        #[arg(long)]
        affected: Option<String>,
        /// Notify subscribers.
        #[arg(long)]
        notify: bool,
    },
    /// List updates on a report.
    #[command(arg_required_else_help = true)]
    Updates {
        /// Status page ID.
        page_id: String,
        /// Report ID.
        report_id: String,
    },
}

impl StatusPagesCmd {
    pub async fn run(&self, ctx: &AppContext) -> Result<CommandOutput> {
        let cmd = match &self.command {
            Some(cmd) => cmd,
            None => {
                use clap::CommandFactory;
                #[derive(clap::Parser)]
                #[command(name = "bs status-pages", about = "Manage status pages.")]
                struct Dummy {
                    #[command(subcommand)]
                    _cmd: Option<StatusPagesSubCmd>,
                }
                Dummy::command().print_help()?;
                println!();
                return Ok(CommandOutput::Empty);
            }
        };
        match cmd {
            StatusPagesSubCmd::List => {
                let pages = ctx.uptime.list_status_pages().await?;
                Ok(pages_to_table(pages))
            }
            StatusPagesSubCmd::Get { id } => {
                let page = ctx.uptime.get_status_page(id).await?;
                if ctx.global.output_format == OutputFormat::Table {
                    let sections = ctx.uptime.list_status_page_sections(id).await.ok();
                    let resources = ctx.uptime.list_status_page_resources(id).await.ok();
                    Ok(page_detail_rich(
                        &page,
                        sections.as_deref(),
                        resources.as_deref(),
                    ))
                } else {
                    Ok(CommandOutput::Detail {
                        fields: build_page_fields(&page),
                    })
                }
            }
            StatusPagesSubCmd::Create {
                name,
                subdomain,
                url,
                domain,
                timezone,
                theme,
                subscribable,
            } => {
                let req = CreateStatusPageRequest {
                    company_name: name.clone(),
                    subdomain: subdomain.clone(),
                    company_url: url.clone(),
                    custom_domain: domain.clone(),
                    timezone: timezone.clone(),
                    theme: theme.clone(),
                    subscribable: if *subscribable { Some(true) } else { None },
                };
                let page = ctx.uptime.create_status_page(&req).await?;
                Ok(CommandOutput::Detail {
                    fields: build_page_fields(&page),
                })
            }
            StatusPagesSubCmd::Update {
                id,
                name,
                subdomain,
                url,
                domain,
                timezone,
                theme,
                subscribable,
            } => {
                let req = UpdateStatusPageRequest {
                    company_name: name.clone(),
                    subdomain: subdomain.clone(),
                    company_url: url.clone(),
                    custom_domain: domain.clone(),
                    timezone: timezone.clone(),
                    theme: theme.clone(),
                    subscribable: *subscribable,
                };
                let page = ctx.uptime.update_status_page(id, &req).await?;
                Ok(CommandOutput::Detail {
                    fields: build_page_fields(&page),
                })
            }
            StatusPagesSubCmd::Delete { id } => {
                ctx.uptime.delete_status_page(id).await?;
                Ok(CommandOutput::Message(format!(
                    "Status page (ID: {id}) deleted."
                )))
            }
            StatusPagesSubCmd::Sections(cmd) => run_sections(ctx, cmd).await,
            StatusPagesSubCmd::Resources(cmd) => run_resources(ctx, cmd).await,
            StatusPagesSubCmd::Reports(cmd) => run_reports(ctx, cmd).await,
        }
    }
}

async fn run_sections(ctx: &AppContext, cmd: &SectionsCmd) -> Result<CommandOutput> {
    match &cmd.command {
        SectionsSubCmd::List { page_id } => {
            let sections = ctx.uptime.list_status_page_sections(page_id).await?;
            Ok(sections_to_table(sections))
        }
        SectionsSubCmd::Get {
            page_id,
            section_id,
        } => {
            let s = ctx
                .uptime
                .get_status_page_section(page_id, section_id)
                .await?;
            Ok(section_to_detail(&s))
        }
        SectionsSubCmd::Create {
            page_id,
            name,
            position,
        } => {
            let req = CreateStatusPageSectionRequest {
                name: name.clone(),
                position: *position,
            };
            let s = ctx.uptime.create_status_page_section(page_id, &req).await?;
            Ok(section_to_detail(&s))
        }
        SectionsSubCmd::Update {
            page_id,
            section_id,
            name,
            position,
        } => {
            let req = UpdateStatusPageSectionRequest {
                name: name.clone(),
                position: *position,
            };
            let s = ctx
                .uptime
                .update_status_page_section(page_id, section_id, &req)
                .await?;
            Ok(section_to_detail(&s))
        }
        SectionsSubCmd::Delete {
            page_id,
            section_id,
        } => {
            ctx.uptime
                .delete_status_page_section(page_id, section_id)
                .await?;
            Ok(CommandOutput::Message(format!(
                "Section (ID: {section_id}) deleted."
            )))
        }
    }
}

async fn run_resources(ctx: &AppContext, cmd: &ResourcesCmd) -> Result<CommandOutput> {
    match &cmd.command {
        ResourcesSubCmd::List { page_id } => {
            let resources = ctx.uptime.list_status_page_resources(page_id).await?;
            Ok(items_to_table(resources))
        }
        ResourcesSubCmd::Get {
            page_id,
            resource_id,
        } => {
            let r = ctx
                .uptime
                .get_status_page_resource(page_id, resource_id)
                .await?;
            Ok(item_to_detail(&r))
        }
        ResourcesSubCmd::Create {
            page_id,
            resource_id,
            resource_type,
            public_name,
            section,
            widget,
        } => {
            let req = CreateStatusPageItemRequest {
                resource_id: *resource_id,
                resource_type: resource_type.clone(),
                public_name: public_name.clone(),
                status_page_section_id: *section,
                widget_type: widget.clone(),
                position: None,
            };
            let r = ctx
                .uptime
                .create_status_page_resource(page_id, &req)
                .await?;
            Ok(item_to_detail(&r))
        }
        ResourcesSubCmd::Update {
            page_id,
            resource_id,
            public_name,
            section,
            widget,
            position,
        } => {
            let req = UpdateStatusPageItemRequest {
                public_name: public_name.clone(),
                status_page_section_id: *section,
                widget_type: widget.clone(),
                position: *position,
            };
            let r = ctx
                .uptime
                .update_status_page_resource(page_id, resource_id, &req)
                .await?;
            Ok(item_to_detail(&r))
        }
        ResourcesSubCmd::Delete {
            page_id,
            resource_id,
        } => {
            ctx.uptime
                .delete_status_page_resource(page_id, resource_id)
                .await?;
            Ok(CommandOutput::Message(format!(
                "Resource (ID: {resource_id}) removed from status page."
            )))
        }
    }
}

async fn run_reports(ctx: &AppContext, cmd: &ReportsCmd) -> Result<CommandOutput> {
    match &cmd.command {
        ReportsSubCmd::List { page_id } => {
            let reports = ctx.uptime.list_status_reports(page_id).await?;
            Ok(reports_to_table(reports))
        }
        ReportsSubCmd::Get { page_id, report_id } => {
            let r = ctx.uptime.get_status_report(page_id, report_id).await?;
            if ctx.global.output_format == OutputFormat::Table {
                let updates = ctx
                    .uptime
                    .list_status_updates(page_id, report_id)
                    .await
                    .ok();
                Ok(report_detail_rich(&r, updates.as_deref()))
            } else {
                Ok(CommandOutput::Detail {
                    fields: build_report_fields(&r),
                })
            }
        }
        ReportsSubCmd::Create {
            page_id,
            title,
            message,
            report_type,
            affected,
            starts_at,
            ends_at,
        } => {
            let affected_resources: Option<Vec<AffectedResource>> = affected
                .as_ref()
                .map(|s| serde_json::from_str(s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("Invalid --affected JSON: {e}"))?;
            let req = CreateStatusReportRequest {
                title: title.clone(),
                report_type: report_type.clone(),
                message: message.clone(),
                affected_resources,
                starts_at: starts_at.clone(),
                ends_at: ends_at.clone(),
            };
            let r = ctx.uptime.create_status_report(page_id, &req).await?;
            Ok(CommandOutput::Detail {
                fields: build_report_fields(&r),
            })
        }
        ReportsSubCmd::Update {
            page_id,
            report_id,
            title,
            ends_at,
        } => {
            let req = UpdateStatusReportRequest {
                title: title.clone(),
                ends_at: ends_at.clone(),
            };
            let r = ctx
                .uptime
                .update_status_report(page_id, report_id, &req)
                .await?;
            Ok(CommandOutput::Detail {
                fields: build_report_fields(&r),
            })
        }
        ReportsSubCmd::Delete { page_id, report_id } => {
            ctx.uptime.delete_status_report(page_id, report_id).await?;
            Ok(CommandOutput::Message(format!(
                "Report (ID: {report_id}) deleted."
            )))
        }
        ReportsSubCmd::AddUpdate {
            page_id,
            report_id,
            message,
            affected,
            notify,
        } => {
            let affected_resources: Option<Vec<AffectedResource>> = if let Some(json) = affected {
                Some(
                    serde_json::from_str(json)
                        .map_err(|e| anyhow::anyhow!("Invalid --affected JSON: {e}"))?,
                )
            } else {
                // Carry forward from the report
                let report = ctx.uptime.get_status_report(page_id, report_id).await?;
                report.attributes.affected_resources
            };
            let req = CreateStatusUpdateRequest {
                message: message.clone(),
                notify_subscribers: if *notify { Some(true) } else { None },
                affected_resources,
            };
            let u = ctx
                .uptime
                .create_status_update(page_id, report_id, &req)
                .await?;
            Ok(CommandOutput::Message(format!(
                "Update posted (ID: {}).",
                u.id
            )))
        }
        ReportsSubCmd::Updates { page_id, report_id } => {
            let updates = ctx.uptime.list_status_updates(page_id, report_id).await?;
            Ok(updates_to_table(updates))
        }
    }
}

// --- Rendering helpers ---

fn pages_to_table(pages: Vec<StatusPageResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Subdomain".to_string(),
        "Status".to_string(),
        "Theme".to_string(),
    ];
    let rows = pages
        .iter()
        .map(|p| {
            let a = &p.attributes;
            vec![
                p.id.clone(),
                a.company_name.clone().unwrap_or_else(|| "-".to_string()),
                a.subdomain.clone().unwrap_or_else(|| "-".to_string()),
                a.aggregate_state.clone().unwrap_or_else(|| "-".to_string()),
                a.theme.clone().unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn build_page_fields(p: &StatusPageResource) -> Vec<(String, String)> {
    let a = &p.attributes;
    let status_raw = a.aggregate_state.as_deref().unwrap_or("-");
    vec![
        ("ID".to_string(), p.id.clone()),
        (
            "Name".to_string(),
            a.company_name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Subdomain".to_string(),
            a.subdomain.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Custom Domain".to_string(),
            a.custom_domain.clone().unwrap_or_else(|| "-".to_string()),
        ),
        ("Status".to_string(), color::status(status_raw)),
        (
            "Theme".to_string(),
            a.theme.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Timezone".to_string(),
            a.timezone.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Subscribable".to_string(),
            a.subscribable
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Company URL".to_string(),
            a.company_url.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Created".to_string(),
            a.created_at
                .as_deref()
                .map(fmt::relative_time)
                .unwrap_or_else(|| "-".to_string()),
        ),
    ]
}

fn page_detail_rich(
    p: &StatusPageResource,
    sections: Option<&[StatusPageSectionResource]>,
    resources: Option<&[StatusPageItemResource]>,
) -> CommandOutput {
    let fields = build_page_fields(p);
    let max_label = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

    let mut out = String::new();
    for (key, value) in &fields {
        out.push_str(&format!(
            "{} {}\n",
            color::bold(&format!("{key:<max_label$}")),
            value
        ));
    }

    if let Some(sections) = sections
        && !sections.is_empty()
    {
        out.push('\n');
        out.push_str(&format!("{}\n", color::bold("Sections")));
        for s in sections {
            let sa = &s.attributes;
            let name = sa.name.as_deref().unwrap_or("(unnamed)");
            let pos = sa.position.map(|v| format!(" #{v}")).unwrap_or_default();
            out.push_str(&format!(
                "  {} {}{}\n",
                color::dim(&format!("#{}", s.id)),
                name,
                color::dim(&pos),
            ));
        }
    }

    if let Some(resources) = resources
        && !resources.is_empty()
    {
        out.push('\n');
        out.push_str(&format!("{}\n", color::bold("Resources")));
        for r in resources {
            let ra = &r.attributes;
            let name = ra.public_name.as_deref().unwrap_or("(unnamed)");
            let status_raw = ra.status.as_deref().unwrap_or("-");
            let avail = ra.availability.map(format_availability).unwrap_or_default();
            out.push_str(&format!(
                "  {} {} {} {}\n",
                color::dim(&format!("#{}", r.id)),
                name,
                color::status(status_raw),
                avail,
            ));
        }
    }

    CommandOutput::Raw(out.trim_end().to_string())
}

fn format_availability(v: f64) -> String {
    let pct = v * 100.0;
    let label = format!("{pct:.2}%");
    if pct >= 99.9 {
        color::green(&label)
    } else if pct >= 99.0 {
        color::yellow(&label)
    } else {
        color::red(&label)
    }
}

fn sections_to_table(sections: Vec<StatusPageSectionResource>) -> CommandOutput {
    let headers = vec!["ID".to_string(), "Name".to_string(), "Position".to_string()];
    let rows = sections
        .iter()
        .map(|s| {
            let a = &s.attributes;
            vec![
                s.id.clone(),
                a.name.clone().unwrap_or_else(|| "-".to_string()),
                a.position
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn section_to_detail(s: &StatusPageSectionResource) -> CommandOutput {
    let a = &s.attributes;
    let fields = vec![
        ("ID".to_string(), s.id.clone()),
        (
            "Name".to_string(),
            a.name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Position".to_string(),
            a.position
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}

fn items_to_table(items: Vec<StatusPageItemResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Type".to_string(),
        "Status".to_string(),
        "Availability".to_string(),
    ];
    let rows = items
        .iter()
        .map(|r| {
            let a = &r.attributes;
            vec![
                r.id.clone(),
                a.public_name.clone().unwrap_or_else(|| "-".to_string()),
                a.resource_type.clone().unwrap_or_else(|| "-".to_string()),
                a.status.clone().unwrap_or_else(|| "-".to_string()),
                a.availability
                    .map(|v| format!("{:.2}%", v * 100.0))
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn item_to_detail(r: &StatusPageItemResource) -> CommandOutput {
    let a = &r.attributes;
    let fields = vec![
        ("ID".to_string(), r.id.clone()),
        (
            "Name".to_string(),
            a.public_name.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Type".to_string(),
            a.resource_type.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Status".to_string(),
            a.status.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Availability".to_string(),
            a.availability
                .map(|v| format!("{:.2}%", v * 100.0))
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Widget".to_string(),
            a.widget_type.clone().unwrap_or_else(|| "-".to_string()),
        ),
    ];
    CommandOutput::Detail { fields }
}

fn reports_to_table(reports: Vec<StatusReportResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Title".to_string(),
        "Type".to_string(),
        "Status".to_string(),
        "Starts At".to_string(),
        "Ends At".to_string(),
    ];
    let rows = reports
        .iter()
        .map(|r| {
            let a = &r.attributes;
            vec![
                r.id.clone(),
                a.title.clone().unwrap_or_else(|| "-".to_string()),
                a.report_type.clone().unwrap_or_else(|| "-".to_string()),
                a.aggregate_state.clone().unwrap_or_else(|| "-".to_string()),
                a.starts_at
                    .as_deref()
                    .map(fmt::relative_time)
                    .unwrap_or_else(|| "-".to_string()),
                a.ends_at
                    .as_deref()
                    .map(fmt::relative_time)
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}

fn build_report_fields(r: &StatusReportResource) -> Vec<(String, String)> {
    let a = &r.attributes;
    let status_raw = a.aggregate_state.as_deref().unwrap_or("-");
    let mut fields = vec![
        ("ID".to_string(), r.id.clone()),
        (
            "Title".to_string(),
            a.title.clone().unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Type".to_string(),
            a.report_type.clone().unwrap_or_else(|| "-".to_string()),
        ),
        ("Status".to_string(), color::status(status_raw)),
        (
            "Starts At".to_string(),
            a.starts_at
                .as_deref()
                .map(fmt::relative_time)
                .unwrap_or_else(|| "-".to_string()),
        ),
        (
            "Ends At".to_string(),
            a.ends_at
                .as_deref()
                .map(fmt::relative_time)
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];
    if let Some(resources) = &a.affected_resources {
        let names: Vec<String> = resources
            .iter()
            .map(|r| {
                let status = r.status.as_deref().unwrap_or("?");
                format!(
                    "{} ({})",
                    r.status_page_resource_id.as_deref().unwrap_or("?"),
                    color::status(status)
                )
            })
            .collect();
        if !names.is_empty() {
            fields.push(("Affected".to_string(), names.join(", ")));
        }
    }
    fields
}

fn report_detail_rich(
    r: &StatusReportResource,
    updates: Option<&[StatusUpdateResource]>,
) -> CommandOutput {
    let fields = build_report_fields(r);
    let max_label = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

    let mut out = String::new();
    for (key, value) in &fields {
        out.push_str(&format!(
            "{} {}\n",
            color::bold(&format!("{key:<max_label$}")),
            value
        ));
    }

    if let Some(updates) = updates
        && !updates.is_empty()
    {
        out.push('\n');
        out.push_str(&format!("{}\n", color::bold("Updates")));
        for (idx, u) in updates.iter().enumerate() {
            let ua = &u.attributes;
            let time = ua
                .published_at
                .as_deref()
                .map(fmt::relative_time)
                .unwrap_or_else(|| "-".to_string());
            let msg = ua.message.as_deref().unwrap_or("-");
            let notified = ua
                .notify_subscribers
                .map(|v| if v { " [notified]" } else { "" })
                .unwrap_or("");

            out.push_str(&format!(
                "  {} {}  {}{}\n",
                color::cyan("●"),
                msg,
                color::dim(&time),
                color::dim(notified),
            ));
            if idx < updates.len() - 1 {
                out.push_str("  │\n");
            }
        }
    }

    CommandOutput::Raw(out.trim_end().to_string())
}

fn updates_to_table(updates: Vec<StatusUpdateResource>) -> CommandOutput {
    let headers = vec![
        "ID".to_string(),
        "Message".to_string(),
        "Published At".to_string(),
        "Notified".to_string(),
    ];
    let rows = updates
        .iter()
        .map(|u| {
            let a = &u.attributes;
            vec![
                u.id.clone(),
                a.message.clone().unwrap_or_else(|| "-".to_string()),
                a.published_at
                    .as_deref()
                    .map(fmt::relative_time)
                    .unwrap_or_else(|| "-".to_string()),
                a.notify_subscribers
                    .map(|v| if v { "yes" } else { "no" })
                    .unwrap_or("-")
                    .to_string(),
            ]
        })
        .collect();
    CommandOutput::Table { headers, rows }
}
