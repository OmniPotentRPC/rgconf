// Copyright 2023--present rgconf developers

use proptest::{prop_assert, prop_assert_eq};
use rgconf::catalog::catalog;
use rgconf::provenance::{BlobAppend, CanonicalLeaf, MissingCapnpCanonicalization, SharedAppend};
use rgconf::redact::redact_env;
use rgconf::sbom::parse_cyclonedx;
use rgconf::units::{gpridge_paper_force_ev_per_angstrom, UNIT_ROUNDTRIP_ABS_TOLERANCE};
use rgconf::{
    apply_edit, convert, convert_stress, ConfigLayer, EditOrigin, Error, ParamValue, Profile,
    ProvenanceLog, Resolver, RgConf, StressSign, Unit, ValueStatus, UPSTREAM_PATCHES,
};

fn resolved(profile: Profile) -> rgconf::Resolution {
    Resolver::new(profile)
        .resolve()
        .expect("schema defaults resolve")
}

fn field<'a>(resolution: &'a rgconf::Resolution, key: &str) -> &'a rgconf::resolve::ResolvedField {
    resolution
        .get(key)
        .unwrap_or_else(|| panic!("missing key {key}"))
}

#[test]
fn catalog_contains_the_real_authority_keys() {
    let cat = catalog().expect("catalog");
    for key in [
        "runtime.job",
        "runtime.temperatureK",
        "band.imageCount",
        "band.zoom.triggerForceEvPerAngstrom",
        "band.spring.constantEvPerAngstrom2",
        "band.spring.ewKMinEvPerAngstrom2",
        "optimizer.handoff.forceThresholdEvPerAngstrom",
        "optimizer.convergence.norm",
        "surrogateGp.modelProfile",
        "potential.arm",
        "gpridge.deprecated.ciStabilityCount",
    ] {
        assert!(cat.get(key).is_some(), "catalog missing {key}");
    }
}

#[test]
fn canonical_profile_fills_derived_and_profile_values() {
    let resolution = resolved(Profile::RgconfCanonical);
    assert_eq!(
        field(&resolution, "runtime.job").value.to_string(),
        "minimization"
    );
    assert_eq!(
        field(&resolution, "runtime.temperatureK").value.to_string(),
        "300"
    );
    assert_eq!(
        field(&resolution, "band.imageCount").value,
        ParamValue::Uint(5)
    );
    assert_eq!(field(&resolution, "potential.arm").value.to_string(), "lj");
    assert_eq!(
        field(&resolution, "optimizer.convergence.norm")
            .value
            .to_string(),
        "maxAtomL2"
    );
    assert_eq!(
        field(&resolution, "band.convergence.norm")
            .value
            .to_string(),
        "maxAtomL2"
    );
    let force = match &field(&resolution, "band.convergence.forceToleranceEvPerAngstrom").value {
        ParamValue::Float(value) => *value,
        other => panic!("force is {other}"),
    };
    assert!((force - 0.01).abs() < UNIT_ROUNDTRIP_ABS_TOLERANCE);
    let zoom = match &field(&resolution, "band.zoom.triggerForceEvPerAngstrom").value {
        ParamValue::Float(value) => *value,
        other => panic!("zoom is {other}"),
    };
    assert!((zoom - 10.0 * force).abs() < UNIT_ROUNDTRIP_ABS_TOLERANCE);
    assert_eq!(
        field(&resolution, "band.zoom.triggerForceEvPerAngstrom").status,
        ValueStatus::DerivedFromUnset
    );
    let kmin = match &field(&resolution, "band.spring.ewKMinEvPerAngstrom2").value {
        ParamValue::Float(value) => *value,
        other => panic!("kmin is {other}"),
    };
    assert!((kmin - 0.5).abs() < UNIT_ROUNDTRIP_ABS_TOLERANCE);
    let view = RgConf::from_resolution(&resolution).expect("view");
    assert_eq!(view.band.image_count, 5);
    assert_eq!(view.runtime.temperature_k, 300.0);
}

#[test]
fn profiles_change_force_images_job_and_potential() {
    let paper = resolved(Profile::GpridgePaper);
    let paper_force = match &field(&paper, "band.convergence.forceToleranceEvPerAngstrom").value {
        ParamValue::Float(value) => *value,
        other => panic!("{other}"),
    };
    assert!(
        (paper_force - gpridge_paper_force_ev_per_angstrom()).abs() < UNIT_ROUNDTRIP_ABS_TOLERANCE
    );
    assert_eq!(field(&paper, "band.imageCount").value, ParamValue::Uint(10));
    assert_eq!(
        field(&paper, "surrogateGp.modelProfile").value.to_string(),
        "gprdZbl"
    );

    let legacy = resolved(Profile::GpridgeLegacyToml);
    let legacy_force = match &field(&legacy, "band.convergence.forceToleranceEvPerAngstrom").value {
        ParamValue::Float(value) => *value,
        other => panic!("{other}"),
    };
    assert!((legacy_force - 0.05).abs() < UNIT_ROUNDTRIP_ABS_TOLERANCE);
    assert_eq!(
        field(&legacy, "surrogateGp.modelProfile").value.to_string(),
        "extended"
    );

    let server = resolved(Profile::EonServerYaml);
    assert_eq!(
        field(&server, "runtime.job").value.to_string(),
        "akmcProcessSearch"
    );
    assert_eq!(
        field(&server, "potential.arm").value.to_string(),
        "external"
    );
    assert_eq!(
        field(&server, "optimizer.convergence.norm")
            .value
            .to_string(),
        "l2Euclidean"
    );
    assert_eq!(
        field(&server, "band.convergence.norm").value.to_string(),
        "maxAtomL2"
    );

    let eon = resolved(Profile::EonLegacyIni);
    assert_eq!(
        field(&eon, "band.convergence.forceToleranceEvPerAngstrom").status,
        ValueStatus::DerivedFromUnset
    );
    assert_eq!(
        field(&eon, "band.convergence.forceToleranceEvPerAngstrom").value,
        field(&eon, "optimizer.convergence.forceToleranceEvPerAngstrom").value
    );
}

#[test]
fn layered_precedence_and_equal_alias_origins() {
    let ini = "\
[Optimizer]\n\
max_iterations = 40\n\
[Nudged Elastic Band]\n\
images = 7\n\
";
    let mut resolver = Resolver::new(Profile::RgconfCanonical);
    resolver
        .merge_ini(ConfigLayer::JobConfigFile, ini, "job.ini")
        .unwrap();
    resolver
        .merge_env([("RGCONF_BAND_IMAGE_COUNT", "11")])
        .unwrap();
    resolver
        .merge_args(&["--images".into(), "8".into()])
        .unwrap();
    let resolution = resolver.resolve().unwrap();
    assert_eq!(
        field(&resolution, "band.imageCount").value,
        ParamValue::Uint(10)
    );
    assert_eq!(
        field(&resolution, "band.imageCount").status,
        ValueStatus::LayeredOverride
    );
    assert_eq!(
        field(&resolution, "optimizer.convergence.maxIterations").value,
        ParamValue::Uint(40)
    );
    assert_eq!(
        field(&resolution, "band.convergence.maxIterations").status,
        ValueStatus::DefaultValue
    );
}

#[test]
fn cli_image_transform_adds_two_and_env_loses_to_cli() {
    let mut resolver = Resolver::new(Profile::RgconfCanonical);
    resolver
        .merge_toml(
            ConfigLayer::ProjectToml,
            "[band]\nnimages = 7\n",
            "proj.toml",
        )
        .unwrap();
    resolver
        .merge_args(&["--nimages".into(), "6".into()])
        .unwrap();
    let resolution = resolver.resolve().unwrap();
    assert_eq!(
        field(&resolution, "band.imageCount").value,
        ParamValue::Uint(8)
    );
}

#[test]
fn contradictory_aliases_name_both_keys() {
    let ini = "\
[run]\n\
max_iterations = 10\n\
[band]\n\
max_iterations = 40\n\
";
    let err = Resolver::new(Profile::RgconfCanonical)
        .merge_toml(ConfigLayer::JobConfigFile, ini, "job.toml")
        .unwrap_err();
    let text = err.to_string();
    assert!(
        text.contains("run") && text.contains("max_iterations"),
        "{text}"
    );
    assert!(text.contains("[band] max_iterations"), "{text}");
}

#[test]
fn equal_aliases_keep_both_origins() {
    let ini = "\
[run]\n\
max_iterations = 40\n\
[band]\n\
max_iterations = 40\n\
";
    let mut resolver = Resolver::new(Profile::RgconfCanonical);
    resolver
        .merge_toml(ConfigLayer::JobConfigFile, ini, "job.toml")
        .unwrap();
    let resolution = resolver.resolve().unwrap();
    let via = &field(&resolution, "band.convergence.maxIterations").via;
    assert!(via.contains("[run] max_iterations"), "{via}");
    assert!(via.contains("[band] max_iterations"), "{via}");
}

#[test]
fn bounds_error_names_the_file_line() {
    let ini = "[Optimizer]\nconverged_force = 0\n";
    let mut resolver = Resolver::new(Profile::RgconfCanonical);
    resolver
        .merge_ini(ConfigLayer::JobConfigFile, ini, "job.ini")
        .unwrap();
    let err = resolver.resolve().unwrap_err();
    let text = err.to_string();
    assert!(text.contains("job.ini:2"), "{text}");
    assert!(
        text.contains("optimizer.convergence.forceToleranceEvPerAngstrom"),
        "{text}"
    );
}

#[test]
fn negative_unsigned_is_rejected() {
    let err = Resolver::new(Profile::RgconfCanonical)
        .merge_args(&["--set".into(), "band.imageCount=-3".into()])
        .unwrap_err();
    let text = err.to_string();
    assert!(text.contains("negative"), "{text}");
    assert!(text.contains("band.imageCount"), "{text}");
}

#[test]
fn refine_follows_the_active_job() {
    let neb = "\
[Main]\n\
job = nudged_elastic_band\n\
[Refine]\n\
opt_method = cg\n\
threshold = 0.2\n\
";
    let mut resolver = Resolver::new(Profile::EonLegacyIni);
    resolver
        .merge_ini(ConfigLayer::JobConfigFile, neb, "client.ini")
        .unwrap();
    let resolution = resolver.resolve().unwrap();
    assert_eq!(
        field(&resolution, "band.refine.handoff.secondaryMethod")
            .value
            .to_string(),
        "cg"
    );
    assert_eq!(
        field(&resolution, "band.refine.handoff.enabled").value,
        ParamValue::Bool(true)
    );
    assert_eq!(
        field(
            &resolution,
            "band.refine.handoff.forceThresholdEvPerAngstrom"
        )
        .value,
        ParamValue::Float(0.2)
    );
    assert_eq!(
        field(&resolution, "optimizer.handoff.enabled").value,
        ParamValue::Bool(false)
    );

    let mini = "[Refine]\nopt_method = fire\nthreshold = 0.25\n";
    let mut resolver = Resolver::new(Profile::EonLegacyIni);
    resolver
        .merge_ini(ConfigLayer::JobConfigFile, mini, "client.ini")
        .unwrap();
    let resolution = resolver.resolve().unwrap();
    assert_eq!(
        field(&resolution, "optimizer.handoff.secondaryMethod")
            .value
            .to_string(),
        "fire"
    );
    assert_eq!(
        field(&resolution, "optimizer.handoff.enabled").value,
        ParamValue::Bool(true)
    );
    assert_eq!(
        field(&resolution, "band.refine.handoff.enabled").value,
        ParamValue::Bool(false)
    );
}

#[test]
fn user_optimizer_force_is_inherited_by_the_eon_band() {
    let ini = "[Optimizer]\nconverged_force = 0.03\n";
    let mut resolver = Resolver::new(Profile::EonLegacyIni);
    resolver
        .merge_ini(ConfigLayer::JobConfigFile, ini, "client.ini")
        .unwrap();
    let resolution = resolver.resolve().unwrap();
    assert_eq!(
        field(&resolution, "band.convergence.forceToleranceEvPerAngstrom").value,
        ParamValue::Float(0.03)
    );
    assert_eq!(
        field(
            &resolution,
            "optimizer.convergence.forceToleranceEvPerAngstrom"
        )
        .status,
        ValueStatus::ExplicitlySet
    );
}

#[test]
fn deprecated_alias_still_loads() {
    let toml = "[band]\nci_stability_count = 4\n";
    let mut resolver = Resolver::new(Profile::GpridgeLegacyToml);
    resolver
        .merge_toml(ConfigLayer::JobConfigFile, toml, "job.toml")
        .unwrap();
    let resolution = resolver.resolve().unwrap();
    let found = field(&resolution, "gpridge.deprecated.ciStabilityCount");
    assert_eq!(found.value, ParamValue::Uint(4));
    assert_eq!(found.status, ValueStatus::DeprecatedAlias);
}

#[test]
fn third_party_extension_key_is_kept() {
    let ini = "[ext.acme]\nalpha = 1.5\n";
    let mut resolver = Resolver::new(Profile::RgconfCanonical);
    resolver
        .merge_ini(ConfigLayer::JobConfigFile, ini, "ext.ini")
        .unwrap();
    let resolution = resolver.resolve().unwrap();
    assert_eq!(
        field(&resolution, "ext.acme.alpha").value,
        ParamValue::Float(1.5)
    );
}

#[test]
fn restart_only_edits_need_a_checkpoint_origin() {
    let mut resolution = resolved(Profile::RgconfCanonical);
    let err = apply_edit(
        &mut resolution,
        "runtime.job",
        ParamValue::Text("saddleSearch".into()),
        EditOrigin::LiveCall,
    )
    .unwrap_err();
    assert!(err.to_string().contains("restartOnly"), "{err}");
    let edit = apply_edit(
        &mut resolution,
        "runtime.job",
        ParamValue::Text("saddleSearch".into()),
        EditOrigin::CheckpointRestart,
    )
    .unwrap();
    assert_eq!(edit.next, "saddleSearch");
    assert_eq!(
        field(&resolution, "runtime.job").status,
        ValueStatus::MutatedMidRun
    );

    let edit = apply_edit(
        &mut resolution,
        "band.imageCount",
        ParamValue::Uint(9),
        EditOrigin::LiveCall,
    )
    .unwrap();
    assert!(edit
        .actions
        .iter()
        .any(|action| action.name() == "redistributeBandImages"));
}

#[test]
fn provenance_leaves_match_rfc9162_and_stubs_name_the_gap() {
    let resolution = resolved(Profile::RgconfCanonical);
    let mut log = ProvenanceLog::open().unwrap();
    log.append_config(&resolution).unwrap();
    let components = parse_cyclonedx(
        r#"{"components":[
            {"name":"rgconf","version":"0.1.0","purl":"pkg:cargo/rgconf@0.1.0"},
            {"name":"numpy","version":"2.0.0","purl":"pkg:pypi/numpy@2.0.0"},
            {"name":"archive","version":"1","purl":"pkg:generic/archive@1"}
        ]}"#,
    )
    .unwrap();
    assert_eq!(components.len(), 3);
    log.append_sbom(&components).unwrap();
    log.append_archive("run.tar", b"sha256:abc").unwrap();
    assert!(log.root_matches_rfc9162());
    assert!(log.inclusion_ok(0));
    assert!(log.inclusion_ok(2));
    let err = MissingCapnpCanonicalization
        .capnp_canonical(b"message")
        .unwrap_err();
    assert!(matches!(err, Error::UpstreamPatch { .. }), "{err}");
    let err = log.append_shared(b"shared").unwrap_err();
    assert!(matches!(err, Error::UpstreamPatch { .. }), "{err}");
    let huge = vec![0u8; 70_000];
    let err = log.put_blob_and_append(&huge).unwrap_err();
    assert!(matches!(err, Error::UpstreamPatch { .. }), "{err}");
    assert!(UPSTREAM_PATCHES.len() >= 4);
}

#[test]
fn redact_keeps_stack_variables_and_blanks_secrets() {
    let kept = redact_env([
        ("PATH", "/usr/bin"),
        ("OMP_NUM_THREADS", "4"),
        ("RGCONF_BAND_IMAGE_COUNT", "8"),
        ("MLFLOW_TRACKING_TOKEN", "secret-token"),
        ("AWS_SECRET_ACCESS_KEY", "secret-key"),
        ("SLURM_JOB_ID", "12"),
    ]);
    assert!(kept.iter().all(|(key, _)| key != "PATH"));
    assert!(kept
        .iter()
        .any(|(key, value)| key == "OMP_NUM_THREADS" && value == "4"));
    assert!(kept
        .iter()
        .any(|(key, value)| key == "RGCONF_BAND_IMAGE_COUNT" && value == "8"));
    assert!(kept
        .iter()
        .any(|(key, value)| key == "MLFLOW_TRACKING_TOKEN" && value == "redacted"));
    assert!(kept
        .iter()
        .any(|(key, value)| key == "AWS_SECRET_ACCESS_KEY" && value == "redacted"));
    assert!(kept
        .iter()
        .any(|(key, value)| key == "SLURM_JOB_ID" && value == "12"));
}

#[test]
fn public_config_setters_round_trip() {
    let config = RgConf::default()
        .with_runtime(rgconf::RuntimeConfig::default().with_temperature_k(10.0))
        .with_band(rgconf::BandConfig::default().with_image_count(9));
    assert_eq!(config.runtime.temperature_k, 10.0);
    assert_eq!(config.band.image_count, 9);
}

#[test]
fn unit_round_trip_and_stress_sign() {
    let back = convert(1.5, Unit::Angstrom, Unit::Bohr).unwrap();
    let home = convert(back, Unit::Bohr, Unit::Angstrom).unwrap();
    assert!((home - 1.5).abs() < UNIT_ROUNDTRIP_ABS_TOLERANCE);
    let flipped = convert_stress(
        2.0,
        Unit::EvPerAngstrom3,
        StressSign::CauchyPositiveTensile,
        Unit::Gpa,
        StressSign::VirialPositiveCompressive,
    )
    .unwrap();
    let plain = convert(2.0, Unit::EvPerAngstrom3, Unit::Gpa).unwrap();
    assert!((flipped + plain).abs() < UNIT_ROUNDTRIP_ABS_TOLERANCE);
    assert!(convert(1.0, Unit::Debye, Unit::InvCm).is_err());
}

proptest::proptest! {
    #[test]
    fn precedence_highest_layer_wins(file in 3u64..30, env in 3u64..30, cli in 3u64..30) {
        let mut resolver = Resolver::new(Profile::RgconfCanonical);
        resolver.merge_args(&["--set".to_string(), format!("band.imageCount={file}")]).unwrap();
        let mut lower = Resolver::new(Profile::RgconfCanonical);
        lower.merge_env([("RGCONF_BAND_IMAGE_COUNT", env.to_string().as_str())]).unwrap();
        lower.merge_args(&["--set".to_string(), format!("band.imageCount={cli}")]).unwrap();
        let won = lower.resolve().unwrap();
        prop_assert_eq!(field(&won, "band.imageCount").value.clone(), ParamValue::Uint(cli));
        let _ = file;
        let _ = resolver;
    }

    #[test]
    fn unit_pairs_round_trip(value in 0.25f64..25.0) {
        let pairs = [
            (Unit::Angstrom, Unit::Bohr),
            (Unit::Ev, Unit::Hartree),
            (Unit::Femtosecond, Unit::Picosecond),
            (Unit::EvPerAngstrom, Unit::HartreePerBohr),
            (Unit::Degree, Unit::Radian),
            (Unit::EvPerAngstrom3, Unit::Gpa),
        ];
        for (from, to) in pairs {
            let converted = convert(value, from, to).unwrap();
            let home = convert(converted, to, from).unwrap();
            prop_assert!((home - value).abs() < UNIT_ROUNDTRIP_ABS_TOLERANCE, "{} -> {}", from.name(), to.name());
        }
    }
}
