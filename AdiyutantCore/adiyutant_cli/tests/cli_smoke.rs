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
