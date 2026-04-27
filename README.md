# Data Engine Studio

Rust-first visual ETL and data exploration studio, distributed through a thin Python package.

## Launch The Milestone 1 Shell

Install the native extension into the active Python environment:

```powershell
py -3.14 -m venv .venv
.\.venv\Scripts\python.exe -m pip install --upgrade pip
.\.venv\Scripts\python.exe -m pip install maturin
.\.venv\Scripts\python.exe -m maturin develop --manifest-path crates\des-python\Cargo.toml
```

Launch the app:

```powershell
.\.venv\Scripts\python.exe -m data_engine_studio
```

Run the initial Rust tests:

```powershell
cargo test
```
