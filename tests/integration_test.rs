use std::process::Command;

fn ktool() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ktool"))
}

#[test]
fn version_subcommand() {
    let output = ktool().arg("version").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("ktool "));
}

#[test]
fn flag_version() {
    let output = ktool().arg("--version").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ktool"));
}

#[test]
fn no_args_shows_help() {
    let output = ktool().output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"));
}

#[test]
fn help_flag() {
    let output = ktool().arg("--help").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"));
}

#[test]
fn magic_hero_screen() {
    let output = ktool().arg("magic").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DEVELOPER BY PASSION"));
    assert!(stdout.contains("COMMODORE 64 FOREVER"));
    assert!(stdout.contains("Made by CaDi Labs with love <3"));
}

#[test]
fn update_help() {
    let output = ktool()
        .args(["update", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
}

#[test]
fn auth_help() {
    let output = ktool()
        .args(["auth", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kenv"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("logout"));
}

#[test]
fn auth_kenv_help() {
    let output = ktool()
        .args(["auth", "kenv", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("login"));
    assert!(stdout.contains("logout"));
    assert!(stdout.contains("whoami"));
}

#[test]
fn auth_status_runs() {
    let output = ktool()
        .args(["auth", "status"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kenv"));
}

#[test]
fn kenv_help() {
    let output = ktool()
        .args(["kenv", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kenv"));
}
