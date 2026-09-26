#!/usr/bin/env python3
"""Archive a Spine release with the upstream native voice package contract."""

import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "third_party" / "voice"))
sys.path.insert(0, str(ROOT / "scripts"))
from assemble_package import assemble
from codex_package.archive import write_archive


def build_archive(package: Path, voice_release_dir: Path, archive: Path) -> None:
    metadata = json.loads((package / "codex-package.json").read_text())
    target = metadata["target"]
    voice_target = (
        target.removesuffix("-musl") + "-gnu" if target.endswith("-musl") else target
    )
    suffix = ".exe" if target.endswith("-windows-msvc") else ""
    commit = subprocess.check_output(
        ["git", "-C", str(ROOT), "rev-parse", "HEAD"], text=True
    ).strip()
    output = package.with_name(package.name + "-voice")
    assemble(
        package,
        voice_release_dir / f"codex-voice-host{suffix}",
        voice_target,
        commit,
        output,
        runtime=voice_release_dir / "runtime",
        release_version=metadata["version"],
    )
    write_archive(output, archive, force=False)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--voice-release-dir", type=Path, required=True)
    parser.add_argument("--archive", type=Path, required=True)
    args = parser.parse_args()
    build_archive(args.package, args.voice_release_dir, args.archive)
