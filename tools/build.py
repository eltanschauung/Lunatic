#!/usr/bin/env python3
"""Build Lunatic's content package and native behavior bundle."""
import argparse
import importlib.util
import os
import json
import zipfile
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--horizon", type=Path, default=ROOT.parent / "horizon")
    parser.add_argument("--skip-tool-build", action="store_true", help="Reuse an already built content packager")
    parser.add_argument("--tools-target", type=Path, help="Reusable Cargo target for the content packager")
    args = parser.parse_args()
    horizon = args.horizon.resolve()
    wrapper = horizon / "tools/fork.py"
    env = os.environ.copy()
    cargo = "cargo"
    if wrapper.is_file():
        spec = importlib.util.spec_from_file_location("horizon_fork", wrapper)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        env, cargo = module.build_environment()
    target = (args.tools_target or ROOT / "target/tools").resolve()
    env["CARGO_TARGET_DIR"] = str(target)
    if not args.skip_tool_build:
        subprocess.run([cargo, "build", "--locked", "--manifest-path", horizon / "Cargo.toml",
        "-p", "cargo-horizon-mod", "-j", "2"], env=env, check=True)
    tool = target / "debug/cargo-horizon-mod"
    subprocess.run(["python3", ROOT / "tools/generate_orbs.py"], check=True)
    subprocess.run(["python3", ROOT / "tools/generate_lampad.py"], check=True)
    subprocess.run(["python3", ROOT / "tools/generate_mushroom_blocks.py"], check=True)
    subprocess.run([tool, "build"], cwd=ROOT, env=env, check=True)
    subprocess.run([tool, "check", ROOT / "target/horizon-mod/lunatic.hmod", "--json"],
        cwd=ROOT, env=env, check=True)

    manifest = {
        "hooks": 359, "schema": 1, "id": "lunatic", "version": "0.13.0", "game": "26.3",
        "horizon": "36f5b83bc6dbc8c19541e55d4625414b095cc446",
        "mixins": "5a64af938ec86b00d53d16a408aabc0038fb3e57",
    }
    output = ROOT / "target/horizon-mod/lunatic.hnmod"
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        def add(name, data):
            entry = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = 0o100644 << 16
            archive.writestr(entry, data)
        add("horizon-native.json", json.dumps(manifest, indent=2).encode())
        for file in sorted((ROOT / "native/mixins").rglob("*")):
            if file.is_file(): add(file.relative_to(ROOT / "native").as_posix(), file.read_bytes())
        for file in sorted((ROOT / "native/licenses").rglob("*")):
            if file.is_file(): add(file.relative_to(ROOT / "native").as_posix(), file.read_bytes())
    print("Built native behavior bundle:", output)


if __name__ == "__main__":
    main()
