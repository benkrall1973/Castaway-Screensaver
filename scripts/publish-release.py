"""Publish a complete, verified release once; never mutate an already published tag."""
from pathlib import Path
import hashlib
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request

ARCHIVES = {
    "Castaway_Screensaver-Windows-x86_64.zip",
    "Castaway_Screensaver-linux-x86_64.tar.gz",
    "Castaway_Screensaver-macos-arm64.tar.gz",
    "Castaway_Screensaver.saver.zip",
    "Castaway_Screensaver-web.zip",
}


def verify_assets(folder):
    folder = Path(folder)
    if {p.name for p in folder.iterdir()} != ARCHIVES | {"SHA256SUMS.txt"}:
        raise ValueError("Release must contain exactly five archives and SHA256SUMS.txt")
    checksums = {}
    for line in (folder / "SHA256SUMS.txt").read_text().splitlines():
        digest, name = line.split(maxsplit=1)
        name = name.lstrip(" *")
        if name not in ARCHIVES or name in checksums or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise ValueError("Invalid or duplicate checksum entry")
        checksums[name] = digest
    if set(checksums) != ARCHIVES:
        raise ValueError("Missing checksum entry")
    for name, digest in checksums.items():
        if hashlib.sha256((folder / name).read_bytes()).hexdigest() != digest:
            raise ValueError("Checksum mismatch: " + name)
    return {p.name: p.read_bytes() for p in folder.iterdir()}


def publish(folder, tag, commit, api):
    assets = verify_assets(folder)
    # GET first: the public release must remain immutable even on a workflow retry.
    existing = api("GET", "releases/tags/" + urllib.parse.quote(tag, safe=""), missing_ok=True)
    if existing and not existing["draft"]:
        raise ValueError("Tag is already published; create a new version instead")
    runs = api("GET", "actions/workflows/ci.yml/runs?head_sha=" + commit + "&per_page=100")
    if not any(r["head_sha"] == commit and r["conclusion"] == "success"
               and r["event"] == "push" and r["head_branch"] == "main"
               for r in runs["workflow_runs"]):
        raise ValueError("Exact release commit needs a passing main-branch CI run")
    release = existing or api("POST", "releases", {
        "tag_name": tag, "target_commitish": commit,
        "name": "Castaway_Screensaver " + tag,
        "body": "Verified packages built from commit " + commit +
                ". All platform builds, tests and dependency audit passed. "
                "See SHA256SUMS.txt before extracting downloads.",
        "draft": True, "prerelease": False,
    })
    for name, data in sorted(assets.items()):
        digest = "sha256:" + hashlib.sha256(data).hexdigest()
        old = next((a for a in release.get("assets", []) if a["name"] == name), None)
        if old and old.get("digest") == digest:
            continue
        if old:
            api("DELETE", "releases/assets/" + str(old["id"]))
        uploaded = api("POST", release["upload_url"].split("{")[0] +
                       "?name=" + urllib.parse.quote(name), raw=data)
        if uploaded.get("digest") != digest or uploaded["size"] != len(data):
            raise ValueError("Uploaded asset verification failed: " + name)
    checked = api("GET", "releases/" + str(release["id"]))
    if not checked["draft"] or {a["name"] for a in checked["assets"]} != set(assets):
        raise ValueError("Draft contains unexpected or missing assets")
    for a in checked["assets"]:
        if a["digest"] != "sha256:" + hashlib.sha256(assets[a["name"]]).hexdigest():
            raise ValueError("Draft asset changed before publication")
    return api("PATCH", "releases/" + str(release["id"]), {"draft": False, "make_latest": "true"})


def main():
    repo = os.environ["GITHUB_REPOSITORY"]
    tag = os.environ["GITHUB_REF_NAME"]
    commit = os.environ["GITHUB_SHA"]
    import tomllib
    version = tomllib.loads(Path("Cargo.toml").read_text())["workspace"]["package"]["version"]
    if tag != "v" + version:
        raise ValueError("Release tag must match the workspace version")
    token = os.environ["GH_TOKEN"]

    def api(method, path, data=None, raw=None, missing_ok=False):
        url = path if path.startswith("https://") else "https://api.github.com/repos/" + repo + "/" + path
        payload = raw if raw is not None else (json.dumps(data).encode() if data is not None else None)
        request = urllib.request.Request(url, data=payload, method=method, headers={
            "Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
            "Content-Type": "application/octet-stream" if raw is not None else "application/json",
            "User-Agent": "Castaway-release",
        })
        try:
            with urllib.request.urlopen(request, timeout=90) as response:
                body = response.read()
                return json.loads(body) if body else None
        except urllib.error.HTTPError as error:
            if error.code == 404 and missing_ok:
                return None
            raise RuntimeError(f"GitHub {method} failed with HTTP {error.code}") from None

    print(publish(sys.argv[1], tag, commit, api)["html_url"])


if __name__ == "__main__":
    main()
