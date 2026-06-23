use adiyutant_core::bundle::dto::BundleStatus;
use adiyutant_core::bundle::source::BundleSource;
use adiyutant_core::service::AdiyutantCoreService;
use clap::Subcommand;
use std::path::PathBuf;
use std::process;

/// Bundle import commands.
#[derive(Subcommand)]
pub enum ImportCmd {
    /// Validate a bundle (directory or .adiyutant.zip)
    Validate {
        /// Path to the bundle directory or .adiyutant.zip file
        path: PathBuf,
    },
    /// Preview what would be imported
    Preview {
        /// Path to the bundle directory or .adiyutant.zip file
        path: PathBuf,
        /// Import mode: append or merge
        #[arg(long, default_value = "append")]
        mode: String,
    },
    /// Apply (import) a bundle to the database
    Apply {
        /// Path to the bundle directory or .adiyutant.zip file
        path: PathBuf,
        /// Import mode: append or merge
        #[arg(long, default_value = "append")]
        mode: String,
    },
}

pub fn handle_import(facade: &AdiyutantCoreService, cmd: &ImportCmd) {
    match cmd {
        ImportCmd::Validate { path } => cmd_validate(facade, path.as_path()),
        ImportCmd::Preview { path, mode } => cmd_preview(facade, path.as_path(), mode),
        ImportCmd::Apply { path, mode } => cmd_apply(facade, path.as_path(), mode),
    }
}

fn resolve_source(path: &std::path::Path) -> BundleSource {
    let path_str = path.to_string_lossy();
    if path_str.ends_with(".zip") || path_str.ends_with(".adiyutant.zip") {
        BundleSource::ZipFile(path.to_path_buf())
    } else if path.is_dir() {
        BundleSource::Directory(path.to_path_buf())
    } else {
        eprintln!(
            "Error: path must be a directory or .zip file: {}",
            path.display()
        );
        process::exit(1);
    }
}

fn print_report_header(bundle_id: &str, schema_version: &str, title: &str, status: &BundleStatus) {
    println!("Bundle ID:      {bundle_id}");
    println!("Schema version: {schema_version}");
    println!("Title:          {title}");
    println!("Status:         {status}");
}

fn print_errors(errors: &[adiyutant_core::bundle::dto::BundleIssueDto]) {
    if !errors.is_empty() {
        println!("\nErrors ({}):", errors.len());
        for e in errors {
            println!("  [{}/{}] {}: {}", e.severity, e.section, e.code, e.message);
        }
    }
}

fn print_warnings(warnings: &[adiyutant_core::bundle::dto::BundleIssueDto]) {
    if !warnings.is_empty() {
        println!("\nWarnings ({}):", warnings.len());
        for w in warnings {
            println!("  [{}/{}] {}: {}", w.severity, w.section, w.code, w.message);
        }
    }
}

fn print_sections(sections: &[adiyutant_core::bundle::dto::BundleSectionReportDto]) {
    println!("\nSections:");
    for s in sections {
        println!(
            "  {}: {} ({} entities, {} actions)",
            s.section,
            s.status,
            s.entity_count,
            s.actions.len()
        );
    }
}

fn cmd_validate(facade: &AdiyutantCoreService, path: &std::path::Path) {
    let source = resolve_source(path);

    match facade.validate_bundle(&source) {
        Ok(report) => {
            print_report_header(
                &report.bundle_id,
                &report.schema_version,
                &report.title,
                &report.status,
            );
            print_errors(&report.errors);
            print_warnings(&report.warnings);
            print_sections(&report.sections);

            if report.status == BundleStatus::Invalid {
                println!("\n❌ Bundle is INVALID.");
                process::exit(1);
            } else if report.status == BundleStatus::ValidWithWarnings {
                println!("\n⚠️  Bundle is valid with warnings.");
            } else {
                println!("\n✅ Bundle is valid.");
            }
        }
        Err(e) => {
            eprintln!("Error validating bundle: {e}");
            process::exit(1);
        }
    }
}

fn cmd_preview(facade: &AdiyutantCoreService, path: &std::path::Path, mode: &str) {
    let imp_mode = parse_mode(mode);
    let source = resolve_source(path);

    match facade.preview_bundle_import(&source, imp_mode) {
        Ok(report) => {
            print_report_header(
                &report.bundle_id,
                &report.schema_version,
                &report.title,
                &report.status,
            );
            println!("Mode: {}", report.mode);
            print_errors(&report.errors);
            print_warnings(&report.warnings);
            print_sections(&report.sections);

            println!("\nActions ({}):", report.actions.len());
            for a in &report.actions {
                println!(
                    "  [{}] {}: {} — {}",
                    a.kind, a.section, a.entity_identifier, a.message
                );
            }

            if report.status == BundleStatus::Invalid {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error previewing bundle: {e}");
            process::exit(1);
        }
    }
}

fn cmd_apply(facade: &AdiyutantCoreService, path: &std::path::Path, mode: &str) {
    let imp_mode = parse_mode(mode);
    let source = resolve_source(path);

    println!("Applying bundle import ({mode} mode)...");

    match facade.apply_bundle_import(&source, imp_mode) {
        Ok(report) => {
            print_report_header(
                &report.bundle_id,
                &report.schema_version,
                &report.title,
                &report.status,
            );
            println!("Mode: {}", report.mode);
            println!("Import run ID: {}", report.import_run_id);
            print_errors(&report.errors);
            print_warnings(&report.warnings);
            print_sections(&report.sections);

            println!("\nActions ({}):", report.actions.len());
            for a in &report.actions {
                println!(
                    "  [{}] {}: {} — {}",
                    a.kind, a.section, a.entity_identifier, a.message
                );
            }

            if report.status == BundleStatus::Failed {
                println!("\n❌ Import FAILED.");
                process::exit(1);
            } else if report.status == BundleStatus::Applied {
                println!("\n✅ Import applied successfully.");
            }
        }
        Err(e) => {
            eprintln!("Error applying bundle: {e}");
            process::exit(1);
        }
    }
}

fn parse_mode(mode: &str) -> adiyutant_core::bundle::bundle_models::ImportMode {
    match mode {
        "append" => adiyutant_core::bundle::bundle_models::ImportMode::Append,
        "merge" => adiyutant_core::bundle::bundle_models::ImportMode::Merge,
        _ => {
            eprintln!("Invalid mode '{}'. Use 'append' or 'merge'.", mode);
            process::exit(1);
        }
    }
}
