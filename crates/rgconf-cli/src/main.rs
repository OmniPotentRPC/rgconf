// Copyright 2023--present rgconf developers

use rgconf::provenance::ProvenanceLog;
use rgconf::{ConfigLayer, Profile, Resolver};
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("rgconf: {err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args
        .next()
        .ok_or("usage: rgconf <resolve|explain|validate> [options]")?;
    match command.as_str() {
        "resolve" => resolve(&args.collect::<Vec<_>>(), false),
        "explain" => resolve(&args.collect::<Vec<_>>(), true),
        "validate" => resolve(&args.collect::<Vec<_>>(), false),
        other => Err(format!("unknown command `{other}`")),
    }
}

fn resolve(args: &[String], explain: bool) -> Result<(), String> {
    let mut profile = Profile::RgconfCanonical;
    let mut files: Vec<(String, String)> = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--profile" => {
                let name = args.get(index + 1).ok_or("--profile needs a name")?;
                profile = Profile::parse(name).map_err(|err| err.to_string())?;
                index += 2;
            }
            "--ini" | "--toml" => {
                let flag = args[index].clone();
                let path = args
                    .get(index + 1)
                    .ok_or("--ini or --toml needs a path")?
                    .clone();
                files.push((flag, path));
                index += 2;
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    let mut resolver = Resolver::new(profile);
    for (flag, path) in &files {
        let text = fs::read_to_string(path).map_err(|err| format!("{path}: {err}"))?;
        if flag == "--ini" {
            resolver
                .merge_ini(ConfigLayer::JobConfigFile, &text, path)
                .map_err(|err| err.to_string())?;
        } else {
            resolver
                .merge_toml(ConfigLayer::JobConfigFile, &text, path)
                .map_err(|err| err.to_string())?;
        }
    }
    let resolution = resolver.resolve().map_err(|err| err.to_string())?;
    println!("profile {}", resolution.profile.name());
    for (key, field) in &resolution.fields {
        if field.status.name() == "defaultValue" && !explain {
            continue;
        }
        println!(
            "{key}={} {} {}",
            field.value,
            field.status.name(),
            field.origin
        );
    }
    if explain {
        let mut log = ProvenanceLog::open().map_err(|err| err.to_string())?;
        log.append_config(&resolution)
            .map_err(|err| err.to_string())?;
        if !log.root_matches_rfc9162() || !log.inclusion_ok(0) {
            return Err("rfc9162 inclusion check failed".to_string());
        }
        println!("provenance leaves {} inclusion ok", log.len());
    }
    Ok(())
}
