# LATAM 2026-2027

This event uses the unified `event.toml` manifest. The initial site maps are
shared from `config/regional_2026`; update those references or replace them
with inline `[[contests]]` and `[[contests.sites]]` entries after the webcast
team prefixes are confirmed.

Run the feeder with:

```bash
make run-feeder CONFIG=config/latam_2026_2027/event.toml \
  BOCA_URL=tests/inputs/webcast_jones.zip
```
