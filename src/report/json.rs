use crate::analysis::ProjectHealthReport;
use anyhow::Result;

pub fn render(report: &ProjectHealthReport) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}
