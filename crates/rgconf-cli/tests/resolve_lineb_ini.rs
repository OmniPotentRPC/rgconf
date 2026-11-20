// Copyright 2023--present rgconf developers

//! CLI resolution of an INI accepted by eOn `[Lineb]` (904ad4f27).

use std::path::PathBuf;
use std::process::Command;

fn rgconf_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rgconf"))
}

fn example_ini() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/lineb.ini")
}

#[test]
fn cli_resolve_eon_lineb_ini() {
    let output = Command::new(rgconf_bin())
        .args([
            "resolve",
            "--profile",
            "eonLegacyIni",
            "--ini",
            example_ini().to_str().expect("utf-8 path"),
        ])
        .output()
        .expect("spawn rgconf");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "rgconf failed: status={} stderr={stderr} stdout={stdout}",
        output.status
    );
    assert!(stdout.contains("profile eonLegacyIni"), "{stdout}");
    assert!(
        stdout.contains("runtime.job=lineb"),
        "missing job: {stdout}"
    );
    assert!(
        stdout.contains("lineb.initialPath=neb.con"),
        "missing initial_path: {stdout}"
    );
    assert!(
        stdout.contains("lineb.energyEv=0.25"),
        "missing energy: {stdout}"
    );
    assert!(
        stdout.contains("lineb.optTolSqrtEv=0.01"),
        "missing opt_tol: {stdout}"
    );
    assert!(
        stdout.contains("lineb.perpTolSqrtEv=0.01"),
        "missing perp_tol: {stdout}"
    );
    assert!(stdout.contains("lineb.beads=0"), "missing beads: {stdout}");
    assert!(
        stdout.contains("lineb.molecular=false"),
        "missing molecular: {stdout}"
    );
    assert!(
        stdout.contains("lineb.maxSteps=400"),
        "missing max_steps: {stdout}"
    );
    assert!(
        stdout.contains("lineb.hbar=0.06465415129579072"),
        "missing hbar: {stdout}"
    );
    assert!(
        stdout.contains("lineb.boltzmann=0.00008617333262145177")
            || stdout.contains("lineb.boltzmann=8.617333262145177e-5"),
        "missing boltzmann: {stdout}"
    );
}

#[test]
fn cli_resolve_lineb_derived_force_tolerances() {
    let dir = tempfile_dir();
    let path = dir.join("minimal-lineb.ini");
    std::fs::write(
        &path,
        "\
[Main]
job = lineb

[Lineb]
initial_path = neb.con
energy = 0.5
result_cache = my-results.db
beads = 0
molecular = false
",
    )
    .expect("write minimal ini");
    let output = Command::new(rgconf_bin())
        .args([
            "resolve",
            "--profile",
            "eonLegacyIni",
            "--ini",
            path.to_str().expect("utf-8 path"),
        ])
        .output()
        .expect("spawn rgconf");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lineb.resultCache=my-results.db"), "{stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "rgconf failed: status={} stderr={stderr} stdout={stdout}",
        output.status
    );
    // Unset opt_tol / perp_tol resolve to eOn 1e-3 (derivedFromUnset).
    assert!(
        stdout.contains("lineb.optTolSqrtEv=0.001"),
        "missing derived opt_tol: {stdout}"
    );
    assert!(
        stdout.contains("derivedFromUnset"),
        "expected derivedFromUnset: {stdout}"
    );
    assert!(
        stdout.contains("lineb.energyEv=0.5"),
        "missing energy: {stdout}"
    );
}

fn tempfile_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rgconf-lineb-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}
