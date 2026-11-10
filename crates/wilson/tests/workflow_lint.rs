// SPDX-License-Identifier: GPL-3.0-or-later
//! Guards invariants of the CI/release workflows so fixed problems can't silently
//! regress. Runs in normal `cargo test` (so CI enforces it).

use std::path::PathBuf;

fn workflow(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.github/workflows")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Producers must never mutate a live release; one publisher runs after checksums.
#[test]
fn release_has_one_publisher_after_complete_artifacts() {
    let yml = workflow("release.yml");
    assert!(!yml.contains("softprops/action-gh-release"));
    assert_eq!(
        yml.matches("python scripts/publish-release.py dist")
            .count(),
        1
    );
    let publisher = yml.split("  publish:").nth(1).expect("publisher job");
    assert!(publisher.contains("needs: checksums"));
    assert!(publisher.contains("if: startsWith(github.ref, 'refs/tags/')"));
    assert!(publisher.contains("contents: write"));
    let producers = yml.split("  publish:").next().unwrap();
    assert!(!producers.contains("contents: write"));
}

/// Both workflows must opt JS actions into Node 24. Node 20 is deprecated — GitHub forces
/// Node 24 from 2026-06-16 — so `actions/upload-artifact`, `softprops/action-gh-release`,
/// `Swatinem/rust-cache`, etc. otherwise log a deprecation warning on every run.
#[test]
fn workflows_force_node24_for_js_actions() {
    for name in ["ci.yml", "release.yml", "pages.yml"] {
        let yml = workflow(name);
        assert!(
            yml.contains("FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true"),
            "{name} must set env `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true` \
             (Node 20 actions are deprecated)"
        );
    }
}

/// The publicly-published artifacts must never bundle the copyright game data — the release
/// binaries and the GitHub Pages site are bring-your-own only.
#[test]
fn public_artifacts_do_not_ship_game_data() {
    for name in ["release.yml", "pages.yml"] {
        let yml = workflow(name);
        for forbidden in ["RESOURCE.", "embed-data", ".wav", "dist.zip"] {
            assert!(
                !yml.contains(forbidden),
                "{name} must not reference `{forbidden}` (copyright data stays out of \
                 public artifacts)"
            );
        }
    }
}

/// `ci.yml` must not trigger on pushes to `claude/**` branches — PR branches are covered by the
/// `pull_request` event, so a `push` trigger would run the whole matrix TWICE per PR push
/// (doubling CI and making "is it green?" ambiguous). It should push-trigger only on `main`.
#[test]
fn ci_does_not_double_run_on_branch_pushes() {
    let yml = workflow("ci.yml");
    assert!(
        !yml.contains("claude/"),
        "ci.yml must not `push`-trigger on claude/** branches (pull_request already covers PR \
         branches; a push trigger doubles every run)"
    );
}
