// Copyright 2023--present rgconf developers

use pyo3::prelude::*;
use rgconf::{ConfigLayer, Profile, Resolver};

#[pyfunction]
fn resolve_ini(profile: &str, text: &str, path: &str) -> PyResult<String> {
    let profile = Profile::parse(profile)
        .map_err(|err| pyo3::exceptions::PyValueError::new_err(err.to_string()))?;
    let mut resolver = Resolver::new(profile);
    if !text.is_empty() {
        resolver
            .merge_ini(ConfigLayer::JobConfigFile, text, path)
            .map_err(|err| pyo3::exceptions::PyValueError::new_err(err.to_string()))?;
    }
    let resolution = resolver
        .resolve()
        .map_err(|err| pyo3::exceptions::PyValueError::new_err(err.to_string()))?;
    let mut lines = vec![format!("profile {}", resolution.profile.name())];
    if let Some(field) = resolution.get("band.imageCount") {
        lines.push(format!("band.imageCount={}", field.value));
    }
    if let Some(field) = resolution.get("optimizer.convergence.forceToleranceEvPerAngstrom") {
        lines.push(format!(
            "optimizer.convergence.forceToleranceEvPerAngstrom={}",
            field.value
        ));
    }
    Ok(lines.join("\n"))
}

#[pymodule]
fn _core(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(resolve_ini, module)?)?;
    Ok(())
}
