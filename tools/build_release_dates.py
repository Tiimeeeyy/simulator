#!/usr/bin/env python3
"""Build data/release_dates.csv from a single full YGOPRODeck dump.

YGOPRODeck asks clients to cache instead of hammering the API, so we fetch the
*entire* database exactly once and derive a compact CSV:

    passcode,tcg_date,ocg_date

Every artwork passcode (card_images[].id) gets a row so alternate arts resolve
without alias lookups.

Usage:
    curl -sS --compressed -o /tmp/cardinfo.json \
        "https://db.ygoprodeck.com/api/v7/cardinfo.php?misc=yes"
    python3 tools/build_release_dates.py /tmp/cardinfo.json data/release_dates.csv
"""
import json
import sys


def main(src, dst):
    cards = json.load(open(src, encoding="utf-8"))["data"]
    rows = {}
    for card in cards:
        misc = (card.get("misc_info") or [{}])[0]
        tcg, ocg = misc.get("tcg_date") or "", misc.get("ocg_date") or ""
        if not tcg and not ocg:
            continue
        ids = {card["id"]} | {img["id"] for img in card.get("card_images", [])}
        for pid in ids:
            rows[pid] = (tcg, ocg)
    with open(dst, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("# passcode,tcg_date,ocg_date  (source: YGOPRODeck cardinfo.php?misc=yes)\n")
        for pid in sorted(rows):
            tcg, ocg = rows[pid]
            fh.write(f"{pid},{tcg},{ocg}\n")
    print(f"wrote {len(rows)} rows to {dst}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
