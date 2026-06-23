use adiyutant_core::bundle::dto::BundleStatus;
use adiyutant_core::service::AdiyutantCoreService;
use clap::Subcommand;
use std::path::PathBuf;
use std::process;

/// Export commands.
#[derive(Subcommand)]
pub enum ExportCmd {
    /// Export a bundle from the database to a directory or .zip file
    Bundle {
        /// Target path for the exported bundle
        target: PathBuf,
    },
}

pub fn handle_export(facade: &AdiyutantCoreService, cmd: &ExportCmd) {
    match cmd {
        ExportCmd::Bundle { target } => cmd_export_bundle(facade, target.as_path()),
    }
}

fn cmd_export_bundle(facade: &AdiyutantCoreService, target: &std::path::Path) {
    match facade.export_bundle(target) {
        Ok(report) => {
            println!("Bundle ID:      {}", report.bundle_id);
            println!("Schema version: {}", report.schema_version);
            println!("Title:          {}", report.title);
            println!("Status:         {}", report.status);
            println!("Target:         {}", report.target_path);
            println!("Sections:       {}", report.section_count);

            if !report.errors.is_empty() {
                println!("\nErrors ({}):", report.errors.len());
                for e in &report.errors {
                    println!("  [{}] {}: {}", e.code, e.section, e.message);
                }
            }
            if !report.warnings.is_empty() {
                println!("\nWarnings ({}):", report.warnings.len());
                for w in &report.warnings {
                    println!("  [{}] {}: {}", w.code, w.section, w.message);
                }
            }

            if report.status == BundleStatus::Failed {
                println!("\n❌ Export FAILED.");
                process::exit(1);
            } else {
                println!("\n✅ Export complete.");
            }
        }
        Err(e) => {
            eprintln!("Error exporting bundle: {e}");
            process::exit(1);
        }
    }
}
