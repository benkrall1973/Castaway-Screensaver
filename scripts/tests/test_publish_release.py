import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("publisher", Path(__file__).parents[1] / "publish-release.py")
publisher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publisher)


class PublisherTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.folder = Path(self.temp.name)
        lines = []
        for name in sorted(publisher.ARCHIVES):
            data = name.encode()
            (self.folder / name).write_bytes(data)
            lines.append(hashlib.sha256(data).hexdigest() + "  " + name)
        (self.folder / "SHA256SUMS.txt").write_text("\n".join(lines) + "\n")

    def test_modified_archive_rejected_before_api(self):
        (self.folder / sorted(publisher.ARCHIVES)[0]).write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "Checksum mismatch"):
            publisher.publish(self.folder, "v1.0.4", "abc", lambda *a, **k: self.fail("API called"))

    def test_missing_archive_rejected(self):
        (self.folder / sorted(publisher.ARCHIVES)[0]).unlink()
        with self.assertRaises(ValueError):
            publisher.verify_assets(self.folder)

    def test_published_release_is_never_modified(self):
        calls = []
        def api(method, path, *args, **kwargs):
            calls.append(method)
            return {"draft": False}
        with self.assertRaisesRegex(ValueError, "already published"):
            publisher.publish(self.folder, "v1.0.4", "abc", api)
        self.assertEqual(calls, ["GET"])

    def test_ci_must_pass_before_draft_creation(self):
        def api(method, path, *args, **kwargs):
            self.assertEqual(method, "GET")
            return None if path.startswith("releases/tags/") else {"workflow_runs": []}
        with self.assertRaisesRegex(ValueError, "passing main-branch CI"):
            publisher.publish(self.folder, "v1.0.4", "abc", api)

    def test_all_assets_verified_before_single_publication(self):
        state = {"id": 7, "draft": True, "upload_url": "https://uploads.example/7{?name}", "assets": []}
        published = []
        def api(method, path, data=None, raw=None, **kwargs):
            if path.startswith("releases/tags/"):
                return None
            if path.startswith("actions/"):
                return {"workflow_runs": [{"head_sha": "abc", "conclusion": "success", "event": "push", "head_branch": "main"}]}
            if method == "POST" and path == "releases":
                self.assertTrue(data["draft"])
                return state
            if method == "POST":
                from urllib.parse import parse_qs, urlparse
                name = parse_qs(urlparse(path).query)["name"][0]
                asset = {"name": name, "size": len(raw), "digest": "sha256:" + hashlib.sha256(raw).hexdigest()}
                state["assets"].append(asset)
                return asset
            if method == "GET":
                return state
            self.assertEqual(method, "PATCH")
            self.assertEqual(len(state["assets"]), 6)
            published.append(data)
            return {"html_url": "https://example/release"}
        publisher.publish(self.folder, "v1.0.4", "abc", api)
        self.assertEqual(published, [{"draft": False, "make_latest": "true"}])


if __name__ == "__main__":
    unittest.main()
