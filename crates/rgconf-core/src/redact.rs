// Copyright 2023--present rgconf developers

const SECRET_SUFFIXES: &[&str] = &["_TOKEN", "_KEY", "_SECRET", "_PASSWORD"];
const KEEP_PREFIXES: &[&str] = &[
    "RGCONF_",
    "RGPKGS_",
    "RGPOT_",
    "OMP_",
    "MKL_",
    "OPENBLAS_",
    "CUDA_",
    "SLURM_",
];

/// Keep stack and scheduler variables. Replace secret values with `redacted`.
pub fn redact_env<I, K, V>(pairs: I) -> Vec<(String, String)>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut out = Vec::new();
    for (key, value) in pairs {
        let key = key.as_ref();
        if !kept(key) {
            continue;
        }
        let stored = if secret(key) {
            "redacted".to_string()
        } else {
            value.as_ref().to_string()
        };
        out.push((key.to_string(), stored));
    }
    out
}

fn kept(key: &str) -> bool {
    KEEP_PREFIXES.iter().any(|prefix| key.starts_with(prefix)) || secret(key)
}

fn secret(key: &str) -> bool {
    key == "MLFLOW_TRACKING_TOKEN" || SECRET_SUFFIXES.iter().any(|suffix| key.ends_with(suffix))
}
