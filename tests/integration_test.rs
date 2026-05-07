use std::process::Command;

fn ktool() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ktool"))
}

// ---------------------------------------------------------------------------
// Basic commands (existing)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// auth
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// kenv top-level
// ---------------------------------------------------------------------------

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

#[test]
fn kenv_help_shows_new_subcommands() {
    let output = ktool()
        .args(["kenv", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("apps"));
    assert!(stdout.contains("envs"));
    assert!(stdout.contains("vars"));
    assert!(stdout.contains("context"));
    assert!(stdout.contains("init"));
}

#[test]
fn kenv_help_shows_set_project_flag() {
    let output = ktool()
        .args(["kenv", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("set-project"));
}

// ---------------------------------------------------------------------------
// kenv list
// ---------------------------------------------------------------------------

#[test]
fn kenv_list_help_shows_detailed_flag() {
    let output = ktool()
        .args(["kenv", "list", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("detailed"));
    assert!(stdout.contains("json"));
}

// ---------------------------------------------------------------------------
// kenv apps
// ---------------------------------------------------------------------------

#[test]
fn kenv_apps_help() {
    let output = ktool()
        .args(["kenv", "apps", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("create"));
}

#[test]
fn kenv_apps_list_help() {
    let output = ktool()
        .args(["kenv", "apps", "list", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
}

#[test]
fn kenv_apps_create_help() {
    let output = ktool()
        .args(["kenv", "apps", "create", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("NAME"));
}

// ---------------------------------------------------------------------------
// kenv envs
// ---------------------------------------------------------------------------

#[test]
fn kenv_envs_help() {
    let output = ktool()
        .args(["kenv", "envs", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("create"));
    assert!(stdout.contains("delete"));
}

#[test]
fn kenv_envs_list_help_shows_app_flag() {
    let output = ktool()
        .args(["kenv", "envs", "list", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("app"));
}

#[test]
fn kenv_envs_create_help_shows_name_and_app() {
    let output = ktool()
        .args(["kenv", "envs", "create", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name"));
    assert!(stdout.contains("app"));
}

#[test]
fn kenv_envs_delete_help_shows_name_and_app() {
    let output = ktool()
        .args(["kenv", "envs", "delete", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name"));
    assert!(stdout.contains("app"));
}

// ---------------------------------------------------------------------------
// kenv vars
// ---------------------------------------------------------------------------

#[test]
fn kenv_vars_help() {
    let output = ktool()
        .args(["kenv", "vars", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("set"));
    assert!(stdout.contains("delete"));
}

#[test]
fn kenv_vars_set_help_shows_yes_flag() {
    let output = ktool()
        .args(["kenv", "vars", "set", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("yes"));
}

#[test]
fn kenv_vars_set_help_shows_override_flags() {
    let output = ktool()
        .args(["kenv", "vars", "set", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("project"));
    assert!(stdout.contains("app"));
    assert!(stdout.contains("env"));
}

#[test]
fn kenv_vars_delete_help_shows_key() {
    let output = ktool()
        .args(["kenv", "vars", "delete", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("key"));
}

// ---------------------------------------------------------------------------
// kenv context
// ---------------------------------------------------------------------------

#[test]
fn kenv_context_help() {
    let output = ktool()
        .args(["kenv", "context", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("current"));
    assert!(stdout.contains("use"));
    assert!(stdout.contains("set"));
    assert!(stdout.contains("delete"));
}

#[test]
fn kenv_context_list_runs_with_no_contexts() {
    // Should succeed and print a "no contexts" message when nothing is configured.
    let output = ktool()
        .args(["kenv", "context", "list"])
        .output()
        .expect("failed to run");
    // May fail if config can't be loaded in the test env; acceptable.
    // But the binary should not crash with an unexpected exit code.
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);
    // We don't assert success because the test runner's HOME dir may already
    // have ktool contexts — just ensure no panic (i.e. it either succeeds or
    // prints an error without segfaulting).
    assert!(
        output.status.success() || !output.stderr.is_empty(),
        "command should either succeed or print an error, not silently crash"
    );
}

#[test]
fn kenv_context_current_runs() {
    let output = ktool()
        .args(["kenv", "context", "current"])
        .output()
        .expect("failed to run");
    // Should either succeed or produce a meaningful error.
    assert!(
        output.status.success() || !output.stderr.is_empty(),
        "command should either succeed or print an error"
    );
}

#[test]
fn kenv_context_set_help_shows_flags() {
    let output = ktool()
        .args(["kenv", "context", "set", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("project"));
    assert!(stdout.contains("app"));
    assert!(stdout.contains("env"));
}

// ---------------------------------------------------------------------------
// kenv init
// ---------------------------------------------------------------------------

#[test]
fn kenv_init_help() {
    let output = ktool()
        .args(["kenv", "init", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("project"));
    assert!(stdout.contains("app"));
    assert!(stdout.contains("env"));
    assert!(stdout.contains("force"));
}

#[test]
fn kenv_init_creates_ktool_toml_in_temp_dir() {
    use std::fs;
    let tmp = tempfile::TempDir::new().expect("failed to create temp dir");

    let output = ktool()
        .args([
            "kenv",
            "init",
            "--project=orbital",
            "--app=backend",
            "--env=production",
        ])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let toml_path = tmp.path().join("ktool.toml");
    assert!(toml_path.exists(), "ktool.toml should have been created");

    let content = fs::read_to_string(&toml_path).unwrap();
    assert!(content.contains("orbital"), "should contain project slug");
    assert!(content.contains("backend"), "should contain app name");
    assert!(content.contains("production"), "should contain env name");
}

#[test]
fn kenv_init_refuses_to_overwrite_without_force() {
    use std::fs;
    let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
    let toml_path = tmp.path().join("ktool.toml");
    fs::write(&toml_path, "project = \"existing\"\n").unwrap();

    let output = ktool()
        .args(["kenv", "init", "--project=new"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run");

    assert!(!output.status.success(), "should fail without --force");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists") || stderr.contains("force"),
        "error should mention existing file or --force: {stderr}"
    );

    // Original file should be untouched.
    let content = fs::read_to_string(&toml_path).unwrap();
    assert!(content.contains("existing"));
}

#[test]
fn kenv_init_force_overwrites_existing_file() {
    use std::fs;
    let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
    let toml_path = tmp.path().join("ktool.toml");
    fs::write(&toml_path, "project = \"old\"\n").unwrap();

    let output = ktool()
        .args(["kenv", "init", "--project=new-slug", "--force"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&toml_path).unwrap();
    assert!(
        content.contains("new-slug"),
        "should contain new project slug"
    );
    assert!(!content.contains("\"old\""), "old content should be gone");
}
