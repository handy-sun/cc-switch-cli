     1|#!/usr/bin/env python3
     2|
     3|import json
     4|import sys
     5|from pathlib import Path
     6|
     7|
     8|def asset_entry(release_dir: Path, base_url: str, filename: str):
     9|    return {
    10|        "url": f"{base_url}/{filename}",
    11|        "signature": (release_dir / f"{filename}.minisig")
    12|        .read_text(encoding="utf-8")
    13|        .strip(),
    14|    }
    15|
    16|
    17|def file_exists(release_dir: Path, filename: str) -> bool:
    18|    return (release_dir / filename).is_file() and (
    19|        release_dir / f"{filename}.minisig"
    20|    ).is_file()
    21|
    22|
    23|def add_mac_platforms(manifest: dict, release_dir: Path, base_url: str):
    24|    universal = "cc-switch-tui-darwin-universal.tar.gz"
    25|    x64 = "cc-switch-tui-darwin-x64.tar.gz"
    26|    arm64 = "cc-switch-tui-darwin-arm64.tar.gz"
    27|
    28|    if file_exists(release_dir, x64):
    29|        manifest["platforms"]["darwin-x86_64"] = asset_entry(release_dir, base_url, x64)
    30|    elif file_exists(release_dir, universal):
    31|        manifest["platforms"]["darwin-x86_64"] = asset_entry(
    32|            release_dir, base_url, universal
    33|        )
    34|
    35|    if file_exists(release_dir, arm64):
    36|        manifest["platforms"]["darwin-aarch64"] = asset_entry(
    37|            release_dir, base_url, arm64
    38|        )
    39|    elif file_exists(release_dir, universal):
    40|        manifest["platforms"]["darwin-aarch64"] = asset_entry(
    41|            release_dir, base_url, universal
    42|        )
    43|
    44|
    45|def add_linux_platform(
    46|    manifest: dict,
    47|    release_dir: Path,
    48|    base_url: str,
    49|    platform_key: str,
    50|    musl_name: str,
    51|    glibc_name: str,
    52|):
    53|    if file_exists(release_dir, musl_name):
    54|        entry: dict[str, object] = dict(asset_entry(release_dir, base_url, musl_name))
    55|        if file_exists(release_dir, glibc_name):
    56|            entry["variants"] = {
    57|                "glibc": asset_entry(release_dir, base_url, glibc_name),
    58|            }
    59|        manifest["platforms"][platform_key] = entry
    60|        return
    61|
    62|    if file_exists(release_dir, glibc_name):
    63|        manifest["platforms"][platform_key] = asset_entry(
    64|            release_dir, base_url, glibc_name
    65|        )
    66|
    67|
    68|def main() -> int:
    69|    if len(sys.argv) != 6:
    70|        print(
    71|            "Usage: generate_latest_json.py <release_dir> <version> <pub_date> <base_url> <notes>",
    72|            file=sys.stderr,
    73|        )
    74|        return 1
    75|
    76|    release_dir = Path(sys.argv[1]).resolve()
    77|    version = sys.argv[2]
    78|    pub_date = sys.argv[3]
    79|    base_url = sys.argv[4].rstrip("/")
    80|    notes = sys.argv[5]
    81|
    82|    manifest = {
    83|        "version": version,
    84|        "notes": notes,
    85|        "pub_date": pub_date,
    86|        "platforms": {},
    87|    }
    88|
    89|    add_mac_platforms(manifest, release_dir, base_url)
    90|    add_linux_platform(
    91|        manifest,
    92|        release_dir,
    93|        base_url,
    94|        "linux-x86_64",
    95|        "cc-switch-tui-linux-x64-musl.tar.gz",
    96|        "cc-switch-tui-linux-x64.tar.gz",
    97|    )
    98|    add_linux_platform(
    99|        manifest,
   100|        release_dir,
   101|        base_url,
   102|        "linux-aarch64",
   103|        "cc-switch-tui-linux-arm64-musl.tar.gz",
   104|        "cc-switch-tui-linux-arm64.tar.gz",
   105|    )
   106|
   107|    windows = "cc-switch-tui-windows-x64.zip"
   108|    if file_exists(release_dir, windows):
   109|        manifest["platforms"]["windows-x86_64"] = asset_entry(
   110|            release_dir, base_url, windows
   111|        )
   112|
   113|    if not manifest["platforms"]:
   114|        print("No signed release assets found to build latest.json", file=sys.stderr)
   115|        return 1
   116|
   117|    output_path = release_dir / "latest.json"
   118|    output_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
   119|    return 0
   120|
   121|
   122|if __name__ == "__main__":
   123|    raise SystemExit(main())
   124|