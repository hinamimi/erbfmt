#!/usr/bin/env python3
"""Read-only formatter/idempotency audit of local ERB corpora; no Rails runtime."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("roots", nargs="+", type=Path)
    parser.add_argument("--binary", type=Path, default=Path("target/debug/erbfmt"))
    parser.add_argument("--allow-unbalanced", action="store_true")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    reports = []
    for root in args.roots:
        if not root.is_dir():
            parser.error(f"not a directory: {root}")
    with tempfile.TemporaryDirectory(prefix="erbfmt-audit-") as directory:
        config = Path(directory) / "erbfmt.json"
        config.write_text(json.dumps({"parser": {"allowUnbalancedHtmlFragments": args.allow_unbalanced}}))
        for root in args.roots:
            revision = subprocess.run(["git", "-C", str(root), "rev-parse", "HEAD"], capture_output=True, text=True)
            result = {"root": str(root), "revision": revision.stdout.strip() or None,
                      "files": 0, "formatted": 0, "changed": 0, "rejected": [], "unstable": []}
            for file in sorted(root.rglob("*.html.erb")):
                source = file.read_bytes()
                command = [str(binary), "--config", str(config), "--stdin-filepath", str(file.resolve())]
                once = subprocess.run(command, input=source, capture_output=True)
                result["files"] += 1
                if once.returncode:
                    result["rejected"].append({"file": str(file), "message": once.stderr.decode()})
                    continue
                result["formatted"] += 1
                result["changed"] += source != once.stdout
                twice = subprocess.run(command, input=once.stdout, capture_output=True)
                if twice.returncode or twice.stdout != once.stdout:
                    result["unstable"].append(str(file))
            reports.append(result)
    payload = {"allowUnbalancedHtmlFragments": args.allow_unbalanced, "corpora": reports}
    args.report.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n")
    for result in reports:
        print(f'{result["root"]}: {result["files"]} files, {result["formatted"]} formatted, '
              f'{len(result["rejected"])} rejected, {len(result["unstable"])} unstable')
    return int(any(result["unstable"] for result in reports))


if __name__ == "__main__":
    raise SystemExit(main())
