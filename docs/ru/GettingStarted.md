# Начало работы

**English:** [../GettingStarted.md](../GettingStarted.md)

## Установка

```bash
pip install kf-sdk
```

Импорт модуля (имя намеренно отличается от имени пакета на PyPI):

```python
import kappa_apk
```

Сборки wheels публикуются для **Linux (manylinux)**, **macOS** (Intel + Apple Silicon) и **Windows**, Python **3.9–3.13**.

### Из исходников (разработка)

```bash
pip install maturin
git clone https://github.com/nsu-ai/kappa.git
cd kappa
maturin develop --release
```

### Локальная сборка wheel

```bash
./build_wheel.sh              # локальная сборка → dist/
./build_wheel.sh -i             # сборка + установка
```

Подробнее о кроссплатформенных wheels и публикации на PyPI — в [BuildAndPublish.md](BuildAndPublish.md).

---

## Шлюз и аутентификация

Весь трафик идёт через **шлюз Traefik API**. Укажите `base_url` как адрес шлюза (не отдельные порты микросервисов):

```python
base_url = "https://kappa.nsu.ru:8061/"   # шлюз Traefik API
```

**Модель идентификации v2:** JWT bearer-токен определяет пользователя и организацию на стороне сервера. Методы клиента **не** принимают `user_id` или `user_type_id` в пути URL (в отличие от устаревшего SDK v1).

```python
from kappa_apk import KappaApkClient

client = KappaApkClient(base_url, login_id="user@example.com", passwd="secret")
info = client.connect()          # POST …/user-micro-services/v2/session/new
print(info["token"], info.get("user_name"))

client.close()                   # DELETE …/user-micro-services/v2/session
```

**Рекомендуется:** контекстный менеджер (автоматический вход и выход):

```python
with KappaApkClient(base_url, login_id, passwd) as client:
    datasets = client.list_datasets_typed()
```

---

## Проверка установки

```python
import kappa_apk
print(kappa_apk.version())   # "3.0.2"
```

---

## Дальнейшие шаги

- [Datasets.md](Datasets.md) — список, создание, загрузка образцов, версии  
- [LoadersAndTransforms.md](LoadersAndTransforms.md) — конвейеры обучения  
- [Benchmarks.md](Benchmarks.md) — отправка результатов бенчмарков  
- [KappaApkClient.md](KappaApkClient.md) — полный справочник API  

[← Оглавление](README.md)
