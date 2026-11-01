#!/usr/bin/env python3
"""Build Lunatic with Horizon's official content-mod tool."""
import argparse
import importlib.util
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--horizon", type=Path, default=ROOT.parent / "horizon")
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
    target = ROOT / "target/tools"
    env["CARGO_TARGET_DIR"] = str(target)
    subprocess.run([cargo, "build", "--locked", "--manifest-path", horizon / "Cargo.toml",
        "-p", "cargo-horizon-mod", "-j", "2"], env=env, check=True)
    tool = target / "debug/cargo-horizon-mod"
    subprocess.run([tool, "build"], cwd=ROOT, env=env, check=True)
    subprocess.run([tool, "check", ROOT / "target/horizon-mod/lunatic.hmod", "--json"],
        cwd=ROOT, env=env, check=True)


if __name__ == "__main__":
    main()
