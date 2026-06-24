/// CLI integration / smoke tests.
/// Uses `std::process::Command` to run the real binary against a temp SQLite DB
/// via the `ADIYUTANT_DB_PATH` environment variable.
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn adiyutant(args: &[&str], db_path: &str) -> std::process::Output {
    let out = Command::new(env!("CARGO_BIN_EXE_adiyutant"))
        .args(args)
        .env("ADIYUTANT_DB_PATH", db_path)
        .output()
        .expect("failed to execute adiyutant");
    if !out.status.success() {
        eprintln!("STDERR: {}", String::from_utf8_lossy(&out.stderr));
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&out.stdout));
    }
    out
}

static DB_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_db_path() -> PathBuf {
    let n = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("adiyutant-test-{}-{}.db", std::process::id(), n))
}

fn temp_db() -> String {
    temp_db_path().to_string_lossy().to_string()
}

fn cleanup(db: &str) {
    let _ = std::fs::remove_file(db);
    let _ = std::fs::remove_file(format!("{}-shm", db));
    let _ = std::fs::remove_file(format!("{}-wal", db));
}

#[test]
fn help_prints_commands() {
    let db = temp_db();
    let out = adiyutant(&["--help"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("today"));
    assert!(stdout.contains("checkin"));
    assert!(stdout.contains("habit"));
    assert!(stdout.contains("timer"));
    assert!(stdout.contains("suggest"));
    cleanup(&db);
}

#[test]
fn today_on_fresh_db() {
    let db = temp_db();
    let out = adiyutant(&["today"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Today:"));
    cleanup(&db);
}

#[test]
fn full_day_scenario() {
    let db = temp_db();

    // 1. Morning check-in
    let out = adiyutant(&["checkin", "morning", "hello"], &db);
    assert!(out.status.success(), "morning checkin failed");

    // 2. Add a habit
    let out = adiyutant(&["habit", "add", "спорт"], &db);
    assert!(out.status.success(), "habit add failed");

    // 3. List habits
    let out = adiyutant(&["habit", "list"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("спорт"));

    // 4. Start timer
    let out = adiyutant(&["timer", "start", "focus", "--minutes", "25"], &db);
    assert!(out.status.success(), "timer start failed");

    // 5. Suggestions
    let out = adiyutant(&["suggest"], &db);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Suggestions"));

    // 6. Context document
    let out = adiyutant(&["context", "add", "core", "Values", "# Values"], &db);
    assert!(out.status.success(), "context add failed");

    // 7. Evening check-in
    let out = adiyutant(&["checkin", "evening", "done"], &db);
    assert!(out.status.success(), "evening checkin failed");

    // 8. Today summary — habits tracked count updated
    let out = adiyutant(&["today"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Today:"));
    assert!(stdout.contains("Check-ins:"));

    cleanup(&db);
}

// Resolved at compile time from the adiyutant_cli crate root
const EXAMPLE_BUNDLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/contracts/examples/adiyutant-bundle-basic"
);

#[test]
fn import_validate_help() {
    let db = temp_db();
    let out = adiyutant(&["import", "validate", "--help"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Validate"));
    assert!(stdout.contains("bundle"));
    cleanup(&db);
}

#[test]
fn import_preview_help() {
    let db = temp_db();
    let out = adiyutant(&["import", "preview", "--help"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Preview"));
    assert!(stdout.contains("bundle"));
    cleanup(&db);
}

#[test]
fn export_help() {
    let db = temp_db();
    let out = adiyutant(&["export", "--help"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Export"));
    cleanup(&db);
}

#[test]
fn basic_import_flow() {
    let db = temp_db();
    let out = adiyutant(&["import", "validate", EXAMPLE_BUNDLE], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Bundle ID:"));
    assert!(stdout.contains("example-basic-2026-06"));
    assert!(stdout.contains("Bundle is valid"));
    cleanup(&db);
}

fn test_bundle_path() -> String {
    EXAMPLE_BUNDLE.to_string()
}

#[test]
fn import_then_project_list() {
    let db = temp_db();
    let bundle = test_bundle_path();

    let out = adiyutant(&["import", "apply", &bundle, "--mode", "append"], &db);
    assert!(
        out.status.success(),
        "import failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = adiyutant(&["project", "list"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("adiyutant"),
        "project list should show adiyutant"
    );
    assert!(
        stdout.contains("warehouse"),
        "project list should show warehouse"
    );

    cleanup(&db);
}

#[test]
fn import_then_project_show() {
    let db = temp_db();
    let bundle = test_bundle_path();

    let out = adiyutant(&["import", "apply", &bundle, "--mode", "append"], &db);
    assert!(out.status.success());

    let out = adiyutant(&["project", "show", "adiyutant"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Adiyutant"));
    assert!(stdout.contains("active"));

    cleanup(&db);
}

#[test]
fn import_then_roadmap_list() {
    let db = temp_db();
    let bundle = test_bundle_path();

    let out = adiyutant(&["import", "apply", &bundle, "--mode", "append"], &db);
    assert!(out.status.success());

    let out = adiyutant(&["roadmap", "list", "adiyutant"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("adiyutant-roadmap"));

    cleanup(&db);
}

#[test]
fn import_then_roadmap_show() {
    let db = temp_db();
    let bundle = test_bundle_path();

    let out = adiyutant(&["import", "apply", &bundle, "--mode", "append"], &db);
    assert!(out.status.success());

    let out = adiyutant(&["roadmap", "show", "adiyutant-roadmap"], &db);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("MVP"));
    assert!(stdout.contains("bundle-v1"));
    assert!(stdout.contains("Android"));
    assert!(stdout.contains("today-screen"));

    cleanup(&db);
}

#[test]
fn import_then_take_item() {
    let db = temp_db();
    let bundle = test_bundle_path();

    let out = adiyutant(&["import", "apply", &bundle, "--mode", "append"], &db);
    assert!(out.status.success(), "import failed");

    let out = adiyutant(
        &["roadmap", "take-item", "bundle-v1", "--date", "tomorrow"],
        &db,
    );
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Bundle v1 contract"));
    assert!(stdout.contains("Plan item id"));
    assert!(stdout.contains("Link id"));
    assert!(stdout.contains("Target date"));

    cleanup(&db);
}
