#!/usr/bin/env python3
"""Extract every historical TCG / OCG Forbidden & Limited list from the
ProjectIgnis/LFLists git history into one file per list.

ProjectIgnis only keeps the *current* lists at HEAD, but every older list is
still reachable through git history. This walks all commits touching the
TCG / OCG files, splits each snapshot into its `!<name>` sections and keeps the
*most recent* revision of each section (so later ID fixes win).

Usage:
    git clone https://github.com/ProjectIgnis/LFLists.git /tmp/LFLists
    python3 tools/extract_lflist_history.py /tmp/LFLists data/lflists
"""
import re
import subprocess
import sys
from pathlib import Path

TRACKED = ["0TCG.lflist.conf", "OCG.lflist.conf", "TCG.lflist.conf", "lflist.conf"]
HEADER = re.compile(r"^!(\d{4})\.(\d{1,2})(?:\.(\d{1,2}))?\s+(TCG|OCG)\s*$")


def git(repo, *args):
    return subprocess.run(["git", "-C", repo, *args], check=True,
                          capture_output=True).stdout.decode("utf-8", "replace")


def split_sections(text):
    """Yield (header_line, body_lines) for each `!` section."""
    header, body = None, []
    for raw in text.splitlines():
        line = raw.rstrip("\r")
        if line.startswith("!"):
            if header:
                yield header, body
            header, body = line.strip(), []
        elif header and line.strip() and not line.startswith("#["):
            body.append(line)
    if header:
        yield header, body


def main(repo, out_dir):
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    # Oldest -> newest so that newer revisions of a section overwrite older ones.
    commits = git(repo, "log", "--reverse", "--format=%H %cI", "--", *TRACKED).split("\n")
    sections = {}
    for entry in filter(None, commits):
        sha, date = entry.split(" ", 1)
        for f in TRACKED:
            try:
                text = git(repo, "show", f"{sha}:{f}")
            except subprocess.CalledProcessError:
                continue
            for header, body in split_sections(text):
                m = HEADER.match(header)
                if not m or len(body) < 5:
                    continue
                y, mo, d, region = m.groups()
                key = (int(y), int(mo), int(d or 1), region)
                sections[key] = (header, body, sha[:10], date)

    for (y, mo, d, region), (header, body, sha, date) in sorted(sections.items()):
        name = f"{y:04d}.{mo:02d}{'' if d == 1 else f'.{d:02d}'} {region}"
        path = out / f"{name}.lflist.conf"
        path.write_text(
            f"#[{name}]\n# source: ProjectIgnis/LFLists @ {sha} ({date})\n"
            f"{header}\n" + "\n".join(body) + "\n", encoding="utf-8")
    print(f"wrote {len(sections)} lists to {out}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
