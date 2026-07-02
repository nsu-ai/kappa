# KappaApk Sync Tracking

> Maintainer log for syncing code from the private **KappaApk** repository into this public **kappa** repo.
> Source repo: `/media/data/Projects/χ-framework/KappaApk` (GitLab: `https://bigdata.nsu.ru:7445/kappa/KappaApk`)

---

## Current Sync State

| Field | Value |
|---|---|
| **Synced tag** | `kappa-apk-3.0.0-beta` |
| **Source commit** | `0a319ef4bb1d7454d722b1873983406c600786e8` |
| **Sync date** | 2026-07-02 |
| **Target branch** | `dev` |
| **Cargo.toml version** | `2.0.0-beta` (tag name ≠ crate version — see note below) |
| **Python package** | `kappa-apk` (`import kappa_apk`) |

**Version note:** The Git tag is named `kappa-apk-3.0.0-beta` but `Cargo.toml` / `version()` still report `2.0.0-beta`. Treat the tag as the release identifier until the crate version is bumped upstream.

---

## Sync History

### Sync #1 — `kappa-apk-3.0.0-beta` (2026-07-02)

**Base:** Public repo had OpenAPI-generated `kf_sdk` (removed in commit `ef69430`).

**Source range:** `v2.0.0-beta` → `kappa-apk-3.0.0-beta`

**Upstream commits included (non-merge):**

| Commit | Summary |
|---|---|
| `8ac03f9` | Benchmarks v1 API reference upgraded to v2 |
| `15a9851` | PyPI build preparation (cibuildwheel, maturin config) |
| `f2003ce` | SDK documentation preparation for release |

**Files changed upstream (29 files, +1380 / −274 lines):**

| Area | Files | Summary |
|---|---|---|
| **Benchmarks** | `src/benchmarks/benchmarks.rs`, `verifications.rs` | v2 verification endpoints; improved error handling |
| **Datasets** | `src/datasets/datasets.rs`, `kappa_dataloader.rs` | Label parsing fixes; extracted `filter_indices_for_dropout()`; dropout warning on min-remaining |
| **Client** | `src/client.rs`, `src/traits.rs` | Minor v2 API alignment |
| **Users** | `src/users/users.rs` | `GET /user-micro-services/v2/users/me` |
| **Utils** | `src/utils/python_json.rs` (removed dead code), `zip_utils.rs` | Cleanup |
| **Build** | `Cargo.toml`, `pyproject.toml`, `build_wheel.sh` | cibuildwheel matrix; maturin config |
| **Docs** | `docs/*.md` (8 new SDK docs), `README.md` | Full user-facing SDK documentation |
| **Types** | `kappa_apk.pyi` | Updated stubs |
| **Tests** | `tests/fixtures/*` | JSON fixtures for label/verification parsing |

**Local modifications applied during sync:**

| Change | Reason |
|---|---|
| `pyproject.toml` URLs → `github.com/nsu-ai/kappa` | Public repo hosting |
| Removed old `kf_sdk/` OpenAPI package | Replaced by Rust SDK |
| Removed OpenAPI-generated `docs/*.md` | Replaced by KappaApk SDK docs |
| CI: `.github/workflows/ci.yml` (Rust + maturin) | Replaces OpenAPI pytest workflow |
| `.gitlab-ci.yml` → `cargo test` + clippy | Replaces old pytest matrix |
| Kept `kappa-rus.png`, `kappa-eng.png`, `README.en.md` | Public repo branding assets |
| License: `LICENSE` (Apache-2.0) from KappaApk | Replaces BSD-3.0 `LICENSE.md` |

**Not synced (intentionally excluded):**

| Item | Reason |
|---|---|
| `wheels/` prebuilt wheels | Build artifacts — not source |
| `target/`, `dist/`, `.venv/` | Local build artifacts |
| `GPNTB Rusmarc_1.1.0.zip` | Dataset archive, not SDK code |
| Branch `dataset-feedback-code-backup` | Feedbacks not in 3.0.0-beta tag; tracked separately |

---

## How to Sync a New Tag

1. Fetch tags in the source repo:
   ```bash
   cd /media/data/Projects/χ-framework/KappaApk
   git fetch --tags origin
   ```

2. Review changes since last sync:
   ```bash
   git log <last-tag>..<new-tag> --oneline
   git diff <last-tag>..<new-tag> --stat
   ```

3. Extract into this repo (exclude build artifacts):
   ```bash
   cd /media/data/Projects/kappa
   git archive --remote=/media/data/Projects/χ-framework/KappaApk <new-tag> \
     | tar -x --exclude='./wheels' --exclude='./target' --exclude='./dist'
   ```

4. Re-apply local modifications (see table above).

5. Update this file with a new **Sync History** entry.

6. Update [FEATURE_CODE_MAPPING.md](FEATURE_CODE_MAPPING.md) if API surface changed.

7. Update [INTERNAL_DOCS.md](INTERNAL_DOCS.md) header and affected sections.

8. Run `cargo test` and `cargo clippy -- -D warnings`.

---

## Upstream Branch Reference

| Branch / Tag | Status | Notes |
|---|---|---|
| `kappa-apk-3.0.0-beta` | **current sync** | Merged `dev` → `master`, 2026-07-02 |
| `v2.0.0` | previous mainline | Benchmarks v2, dropout hooks |
| `v2.0.0-beta` | prior beta | Base for 3.0.0-beta diff |
| `dataset-feedback-code-backup` | not synced | Full Feedbacks implementation — future sync candidate |

---

*Last updated: 2026-07-02*
