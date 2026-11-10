# Security and build maintenance - 2026-11-10

The first full audit found ten vulnerability records and ten unsoundness records
in the previous lockfile. These describe dependency versions; they do not prove
that all affected APIs were reachable from the application.

## Repairs

- ZIP 2.4.2 is replaced by pinned ZIP 9.5.0 in both desktop and browser loaders.
- Test archive writers use only needed Deflate features. Unused Zstandard,
  BZip2, LZMA and encryption dependencies are removed from the lockfile.
- Wayland scanner 0.31.11 selects quick-xml 0.41.0, fixing the reported XML issues.
- Withdrawn clipboard-win 5.4.1 is replaced by 5.4.3.
- Rust 1.99.1 is pinned; CI/release builds enforce the checked-in Cargo lock.
- CI audits dependencies with cargo-audit 0.22.2. Vulnerabilities, yanked crates
  and unsoundness fail the gate. No advisory is ignored.

A full local scan of the updated lock found zero vulnerability, unsoundness
and yanked-package records. Two unmaintained-package warnings remain: paste 1.0.15
(RUSTSEC-2024-0436) and ttf-parser 0.25.1 (RUSTSEC-2026-0192). These are visible
maintenance debt, not suppressed findings. Replacing them may require a separate
GUI dependency upgrade and display testing.

The repository retains original reference data by owner instruction. That data
is separate from the GPL source-code license. Downloadable application packages
continue to require original data supplied at runtime.

## Platform boundary

Windows, Linux, macOS, WASM and browser CI do not establish graphical screensaver
installation or playback correctness on every user's machine. macOS users should
place original data under `~/Library/Application Support/Castaway_Screensaver/`.
The bundle name and identifier now belong to this fork; the previous bundle can
remain installed separately until the new one is tested.

## Completed validation

All seven CI jobs, including the strict dependency audit, and all five release
jobs passed for the v1.0.3 source commit. The rebuilt Windows executable also
passed a local help-command smoke test. Archive checksums and macOS bundle
metadata were verified before publication. See the
[validation record](V1_0_3_VALIDATION_2026-11-10.json) and
[v1.0.3 downloads](https://github.com/benkrall1973/Castaway-Screensaver/releases/tag/v1.0.3).

The [web page](https://benkrall1973.github.io/Castaway-Screensaver/) is deployed
using this repository's Pages workflow.
