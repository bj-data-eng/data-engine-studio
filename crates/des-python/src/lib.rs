use des_ui_lab::NativeLaunchOptions;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
struct NativeRuntimeInfo {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    version: String,
}

#[pyfunction]
fn hello() -> String {
    "hello from data_engine_studio".to_string()
}

#[pyfunction]
fn runtime_info() -> NativeRuntimeInfo {
    let info = des_core::AppInfo::current();
    NativeRuntimeInfo {
        name: info.name,
        version: info.version,
    }
}

#[pyfunction]
#[pyo3(signature = (title=None))]
fn launch(title: Option<String>) -> PyResult<()> {
    let mut options = NativeLaunchOptions::default();
    if let Some(title) = title {
        options.title = title;
    }
    des_ui_lab::run_native(options).map_err(|error| PyRuntimeError::new_err(error.to_string()))
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(hello, module)?)?;
    module.add_function(wrap_pyfunction!(launch, module)?)?;
    module.add_function(wrap_pyfunction!(runtime_info, module)?)?;
    Ok(())
}
