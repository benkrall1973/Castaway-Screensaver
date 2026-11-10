# Release runbook

1. Update the workspace, lockfile and macOS bundle versions together. Update
   current installation records and changelog. Keep prior released tags intact.
2. Push to `main` and wait for all eight CI checks to pass for the exact commit.
3. Tag that commit with `v` plus the workspace version, then push the tag.
4. The Release workflow builds all platforms and calculates SHA256SUMS. One
   publisher verifies the complete download set, checks exact-commit CI, uploads
   to a draft, verifies server hashes and only then makes the release public.
5. Download and verify checksums. Smoke-test Windows EXE/SCR and confirm macOS
   bundle metadata. Record validation and refresh the offline Git/release backup.

Manual workflow dispatch produces artifacts only. If a tagged publisher runs
before CI finishes, it refuses publication; after CI succeeds, rerun only the
failed job. If an upload fails, the release remains a draft and can be resumed.
An already published tag is immutable: publish a new version for corrections.

The v1.0.5 release exercised the guarded publication path successfully. See
[its validation record](V1_0_5_VALIDATION_2026-11-10.json).

## Cache behavior

ZIP caches use SHA-256 of the archive snapshot. Installer caches hash the map,
compressed resource data and optional compressed sound source with length framing.
Private temporary staging prevents incomplete output from becoming a reusable
cache. Changes to archive contents select a new cache even when file names and
sizes match. Old caches are not migrated or deleted by this repair.

Graphical screensaver installation and playback testing are separate acceptance
steps. Cross-platform CI and help-command smoke tests do not replace them.
