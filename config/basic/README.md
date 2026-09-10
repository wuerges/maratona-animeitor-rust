# Basic configuration example

`event.toml` is the smallest complete event manifest. Run it with a local
webcast using:

```bash
make run-basic BOCA_URL=tests/inputs/webcast_jones.zip
```

For a real event, use `make run-config CONFIG=...` with that event's manifest.
