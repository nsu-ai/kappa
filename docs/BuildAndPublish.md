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

Configured in [`pyproject.toml`](../pyproject.toml). By default Linux builds use **`archs = ["auto"]`** (native CPU only), so `cibuildwheel` works on x86_64 without QEMU.

```bash
pip install cibuildwheel maturin
maturin sdist -o dist
cibuildwheel --output-dir dist
twine check dist/*
twine upload dist/*
```

### Linux architectures

| Goal | Command |
|---|---|
| **Local (native arch)** | `cibuildwheel --output-dir dist` |
| **x86_64 + aarch64** (release) | See [GitHub Actions publish](#github-actions) or enable QEMU locally (below) |

**Why `exec format error`?** On x86_64, building `manylinux2014_aarch64` requires ARM emulation. Without QEMU, Docker cannot run the aarch64 entrypoint.

**Local multi-arch (optional):** register binfmt + QEMU, then override archs:

```bash
docker run --privileged --rm tonistiigi/binfmt --install all
CIBW_ARCHS_LINUX="x86_64 aarch64" cibuildwheel --output-dir dist
```

| Platform | Default `cibuildwheel` tags |
|---|---|
| Linux | manylinux2014, native arch (`auto`) |
| macOS | x86_64 + arm64 |
| Windows | AMD64 |

---

## GitHub Actions

| Workflow | Trigger | Action |
|---|---|---|
| [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) | push / PR | `cargo test`, clippy, maturin wheels (py3.9–3.13), sdist |
| [`.github/workflows/publish.yml`](../.github/workflows/publish.yml) | Release / manual | sdist + cibuildwheel on Linux/macOS/Windows → PyPI (Linux uses QEMU for aarch64) |

**PyPI trusted publishing:** configure environment `pypi` on GitHub, create a Release tag (e.g. `v3.0.1`).

---

## Requirements

- Rust (`rustup`)
- `maturin >= 1.9`
- Python 3.9+ (per wheel ABI)
- Optional: `twine`, `cibuildwheel`

[← Index](README.md)
