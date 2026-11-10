# Repository consolidation — 2026-11-10

The private `Castaway_Screensaver` repository is the active project repository.
The former public and private repositories had unrelated Git histories but
identical application source at consolidation time. Both histories are retained
as parents of the consolidation commit.

Public packaging corrections, installation documentation, portable default
configuration and Windows binaries were incorporated. Private original data,
SCR reference, installer archive and forensic tools were retained unchanged.
No application runtime source was changed by consolidation.

The original branch tips remain at these recovery tags:

- `archive/pre-consolidation-private-2026-11-10`
- `archive/pre-consolidation-public-2026-11-10`

Public release tags and downloadable release assets are preserved in the private
repository. Original release metadata and SHA-256 inventories are retained in
`docs/archive/public-releases-2026-11-10.json`. Existing release binaries are
historical artifacts, not new builds produced by consolidation.

Keep this repository private. Public package workflows deliberately omit the
private original-data and analysis directories. Deleting the former public
repository is a separate final action after verifying the consolidation.
