"""Build the selected revision and export its assembled API contract for CI."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys


def main():
    root = Path.cwd()
    destination = Path(os.environ["CRUCIBLE_MODEL_OUT"])
    metadata = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
        check=True, capture_output=True, text=True,
    )
    manifest = (root / "service/Cargo.toml").resolve()
    package = next(
        item for item in json.loads(metadata.stdout)["packages"]
        if Path(item["manifest_path"]).resolve() == manifest
    )
    result = subprocess.run(
        ["cargo", "build", "--locked", "-p", package["name"], "--message-format=json"],
        capture_output=True, text=True,
    )
    sys.stderr.write(result.stderr)
    outputs = set()
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if message["reason"] == "compiler-message":
            sys.stderr.write(message["message"].get("rendered") or "")
        if message["reason"] == "build-script-executed" and message["package_id"] == package["id"]:
            outputs.add(Path(message["out_dir"]))
    if result.returncode:
        raise SystemExit(result.returncode)
    if len(outputs) != 1:
        raise SystemExit("Expected one service build output containing the assembled API model.")
    # The first baseline uses a build helper without CRUCIBLE_MODEL_OUT support.
    # Cargo's reported output selects that revision's model, including cached builds.
    model = outputs.pop() / "smithy/compiled/crucible/model/model.json"
    if not model.is_file():
        raise SystemExit(f"Missing assembled API model: {model}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(model, destination)


if __name__ == "__main__":
    main()
