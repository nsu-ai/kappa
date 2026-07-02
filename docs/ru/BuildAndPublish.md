# Сборка и публикация

**English:** [../BuildAndPublish.md](../BuildAndPublish.md)

---

## Локальная сборка (`build_wheel.sh`)

| Режим | Команда |
|---|---|
| **Только локально** (по умолчанию) | `./build_wheel.sh` или `./build_wheel.sh --local` |
| **Загрузка на PyPI** | `./build_wheel.sh --upload` или `./build_wheel.sh -u` |

Частые флаги:

```bash
./build_wheel.sh --local --no-sdist -v 3.11   # один wheel, без sdist
./build_wheel.sh -i                             # сборка + установка + smoke test
./build_wheel.sh -c --upload                    # очистка + сборка + загрузка
./build_wheel.sh --test                         # сборка + smoke test импорта
```

Каталог вывода: **`dist/`** (wheels + опциональный sdist `*.tar.gz`).

Целевые версии Python: **3.9–3.13** (сборка для каждого интерпретатора, найденного в системе).

---

## Ручной maturin

```bash
maturin sdist -o dist
maturin build --release --interpreter python3.11 -o dist
maturin develop --release          # редактируемая dev-установка
twine check dist/*
```

---

## Кроссплатформенные wheels (cibuildwheel)

Настроено в [`pyproject.toml`](../../pyproject.toml):

```bash
pip install cibuildwheel maturin
maturin sdist -o dist
cibuildwheel --output-dir dist
twine check dist/*
twine upload dist/*
```

| Платформа | Теги |
|---|---|
| Linux | manylinux2014 x86_64 + aarch64 |
| macOS | x86_64 + arm64 |
| Windows | AMD64 |

---

## GitHub Actions

| Workflow | Триггер | Действие |
|---|---|---|
| [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) | push / PR | `cargo test`, clippy, maturin wheels (py3.9–3.13), sdist |
| [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml) | Release / вручную | sdist + cibuildwheel на Linux/macOS/Windows → PyPI |

**Trusted publishing PyPI:** настройте окружение `pypi` на GitHub, создайте тег Release (например, `v2.0.0-beta`).

---

## Требования

- Rust (`rustup`)
- `maturin >= 1.9`
- Python 3.9+ (для каждого ABI wheel)
- Опционально: `twine`, `cibuildwheel`

[← Оглавление](README.md)
