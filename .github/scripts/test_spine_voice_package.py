"""Exercise the release archive, including its complete voice manifest."""

import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sys
import tarfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "third_party" / "voice"))
import test_assemble_package as fixtures
from release_runtime import seal, stage

spec = importlib.util.spec_from_file_location(
    "spine_voice_package", Path(__file__).with_name("assemble-spine-voice-package.py")
)
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)


class SpineVoicePackageTest(unittest.TestCase):
    def test_release_archives_include_verified_runtime_for_all_native_targets(self):
        for arch in ("aarch64", "x86_64"):
            for app_suffix, voice_suffix, plugin in (
                (
                    "unknown-linux-musl",
                    "unknown-linux-gnu",
                    "lib/gstreamer-1.0/libgst{}.so",
                ),
                ("apple-darwin", "apple-darwin", "plugins/libgst{}.dylib"),
                ("pc-windows-msvc", "pc-windows-msvc", "bin/gst{}.dll"),
            ):
                target = f"{arch}-{app_suffix}"
                with self.subTest(target=target):
                    fixture = fixtures.AssembleTests()
                    fixture.setUp()
                    self.addCleanup(fixture.doCleanups)
                    suffix = ".exe" if "windows" in target else ""
                    fixture.metadata.update(
                        target=target, entrypoint=f"bin/codex{suffix}", version="0.5.0"
                    )
                    (fixture.package / "codex-package.json").write_text(
                        json.dumps(fixture.metadata)
                    )
                    (fixture.package / f"bin/codex{suffix}").write_bytes(b"app")
                    voice_target = f"{arch}-{voice_suffix}"
                    runtime, _ = fixture.make_runtime(voice_target, plugin)
                    release = fixture.root / "release"
                    release.mkdir()
                    stage(runtime, release / "runtime", voice_target)
                    seal(release / "runtime", voice_target)
                    shutil.copy2(fixture.helper, release / f"codex-voice-host{suffix}")
                    archive = fixture.root / "release.tar.gz"
                    with patch.object(
                        builder.subprocess, "check_output", return_value="b" * 40
                    ):
                        builder.build_archive(fixture.package, release, archive)
                    with tarfile.open(archive) as packaged:
                        manifest = json.load(
                            packaged.extractfile("codex-resources/voice/manifest.json")
                        )
                        self.assertEqual(manifest["appTarget"], target)
                        self.assertEqual(manifest["voiceTarget"], voice_target)
                        self.assertIn(
                            "codex-resources/voice/NOTICE.md", manifest["sha256"]
                        )
                        actual = {
                            name: hashlib.sha256(
                                packaged.extractfile(name).read()
                            ).hexdigest()
                            for name in manifest["sha256"]
                        }
                        self.assertEqual(actual, manifest["sha256"])


if __name__ == "__main__":
    unittest.main()
