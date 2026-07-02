# Build & publish

**Русская версия:** [ru/BuildAndPublish.md](ru/BuildAndPublish.md)

---

## Local build (`build_wheel.sh`)

| Mode | Command |
|---|---|
| **Local only** (default) | `./build_wheel.sh` or `./build_wheel.sh --local` |
| **Upload to PyPI** | `./build_wheel.sh --upload` or `./build_wheel.sh -u` |

Common flags:

```bash
./build_wheel.sh --local --no-sdist -v 3.11   # one wheel, no sdist
./build_wheel.sh -i                             # build + install + smoke test
./build_wheel.sh -c --upload                    # clean + build + upload
./build_wheel.sh --test                         # build + import smoke test
```

Output directory: **`dist/`** (wheels + optional `*.tar.gz` sdist).

Python targets: **3.9–3.13** (builds for every interpreter found on the machine).

---

## Manual maturin

```bash
maturin sdist -o dist
maturin build --release --interpreter python3.11 -o dist
maturin develop --release          # editable dev install
twine check dist/*
```

---

## Cross-platform wheels (cibuildwheel)

Configured in [`pyproject.toml`](../pyproject.toml):

```bash
pip install cibuildwheel maturin
maturin sdist -o dist
cibuildwheel --output-dir dist
twine check dist/*
twine upload dist/*
```

| Platform | Tags |
|---|---|
| Linux | manylinux2014 x86_64 + aarch64 |
| macOS | x86_64 + arm64 |
| Windows | AMD64 |

---

## GitHub Actions

| Workflow | Trigger | Action |
|---|---|---|
| [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) | push / PR | `cargo test`, clippy, maturin wheels (py3.9–3.13), sdist |
| [`.github/workflows/publish.yml`](../.github/workflows/publish.yml) | Release / manual | sdist + cibuildwheel on Linux/macOS/Windows → PyPI |

**PyPI trusted publishing:** configure environment `pypi` on GitHub, create a Release tag (e.g. `v2.0.0-beta`).

---

## Requirements

- Rust (`rustup`)
- `maturin >= 1.9`
- Python 3.9+ (per wheel ABI)
- Optional: `twine`, `cibuildwheel`

[← Index](README.md)
