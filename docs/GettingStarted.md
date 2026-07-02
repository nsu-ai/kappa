# Getting started

**Русская версия:** [ru/GettingStarted.md](ru/GettingStarted.md)

## Install

```bash
pip install kappa-apk
```

Wheels are published for **Linux (manylinux)**, **macOS** (Intel + Apple Silicon), and **Windows**, Python **3.9–3.13**.

### From source (development)

```bash
pip install maturin
git clone https://bigdata.nsu.ru:7445/kappa/KappaApk.git
cd KappaApk
maturin develop --release
```

### Local wheel

```bash
./build_wheel.sh              # local build → dist/
./build_wheel.sh -i             # build + install
```

See [BuildAndPublish.md](BuildAndPublish.md) for cross-platform wheels and PyPI upload.

---

## Gateway & authentication

All traffic goes through the **Traefik API gateway**. Set `base_url` to the gateway host (not individual microservice ports):

```python
base_url = "https://kappa.nsu.ru:8061/"   # Traefik API gateway
```

**v2 identity model:** the JWT bearer token identifies the user and org server-side. Client methods do **not** take `user_id` or `user_type_id` path parameters (unlike legacy v1 SDK).

```python
from kappa_apk import KappaApkClient

client = KappaApkClient(base_url, login_id="user@example.com", passwd="secret")
info = client.connect()          # POST …/user-micro-services/v2/session/new
print(info["token"], info.get("user_name"))

client.close()                   # DELETE …/user-micro-services/v2/session/
```

**Recommended:** context manager (auto connect + logout):

```python
with KappaApkClient(base_url, login_id, passwd) as client:
    datasets = client.list_datasets_typed()
```

---

## Verify installation

```python
import kappa_apk
print(kappa_apk.version())   # "2.0.0-beta"
```

---

## Next steps

- [Datasets.md](Datasets.md) — list, create, upload samples, versions  
- [LoadersAndTransforms.md](LoadersAndTransforms.md) — training pipelines  
- [Benchmarks.md](Benchmarks.md) — model benchmark submission  
- [KappaApkClient.md](KappaApkClient.md) — complete API reference  

[← Index](README.md)
