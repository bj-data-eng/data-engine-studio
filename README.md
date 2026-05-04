# Data Engine Studio

Rust-first visual ETL and data exploration studio, distributed through a thin Python package.

## Launch The Milestone 1 Shell

Install the native extension into the active Python environment on macOS or Linux:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install --upgrade pip maturin
.venv/bin/python -m maturin develop --manifest-path crates/des-python/Cargo.toml
```

On Windows:

```powershell
py -3.14 -m venv .venv
.\.venv\Scripts\python.exe -m pip install --upgrade pip
.\.venv\Scripts\python.exe -m pip install maturin
.\.venv\Scripts\python.exe -m maturin develop --manifest-path crates\des-python\Cargo.toml
```

Launch the app on macOS or Linux:

```sh
.venv/bin/python -m data_engine_studio
```

On Windows:

```powershell
.\.venv\Scripts\python.exe -m data_engine_studio
```

Run the initial Rust tests:

```sh
cargo test
```

Use `just ui-shot-mac` or `just ui-debug-mac` for native macOS/Linux screenshot captures.
Use `just ui-shot-windows` or `just ui-debug-windows` on Windows.
Use `just dev-mac` or `just dev-windows` to build and run the native development shell.
