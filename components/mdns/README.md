# Test with

```bash
sudo apt-get install avahi-utils

avahi-browse -at

```

In Homeassistant add the following to `configuration.yaml`: 

```yaml
logger:
  logs:
    zeroconf: debug
```

or

```bash
python3 -m venv .venv
.\.venv\Scripts\activate.bat
pip install aioesphomeapi
aioesphomeapi-discover -v
```

```
# aioesphomeapi-discover -v
Status |Name                            |Address        |MAC         |Version         |Platform  |Board
------------------------------------------------------------------------------------------------------------------------
ONLINE |new_awesome                     |172.20.208.1   |unknown     |2024.4.2        |unknown   |unknown
```