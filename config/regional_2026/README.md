# Regional 2026

The active event is described by `event.toml`. It selects the detailed contest
and site maps. Runtime targets are defined in the project root Makefile.

Run the feeder with:

```bash
make run-feeder CONFIG=config/regional_2026/event.toml \
  BOCA_URL=tests/inputs/webcast_jones.zip
```
