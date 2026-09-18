"""Regenerates `crates/wuim-probe/src/board_ids.rs` from board-identify's table.

The table itself is not ours. board-identify merges it from a published dump of
`arduino-cli board details`, and this script copies that result rather than
re-deriving it, so both tools name the same board the same way. Adding a step of
our own between the two would be a way for the names to drift apart.

    python scripts/generate_board_ids.py ../board-identify

Espressif pairs are dropped: an ESP32 is identified from its eFuse MAC, which
comes from the silicon and survives a bridge being reflashed or replaced. Pairs
with no single board behind them are kept — the family still rules a probe out
even where it cannot name anything.
"""

import argparse
import ast
import pathlib
import sys

HEADER = """//! VID/PID pairs that name a board on their own.
//!
//! **Generated. Do not edit by hand** — see `scripts/generate_board_ids.py`.
//!
//! Source: board-identify's `arduino_ids.py` ({entries} pairs), itself merged
//! from a published dump of `arduino-cli board details`. Copied rather than
//! re-derived so that both tools give one board one name.
//!
//! Espressif pairs are left out: those boards are identified from the eFuse MAC
//! instead, which comes from the silicon rather than from a descriptor a
//! reflash can change.

/// One VID/PID pair, and what the board definitions say claims it.
pub struct BoardId {{
    /// `vid << 16 | pid`, which is what the table is sorted and searched on.
    pub key: u32,
    /// The family, e.g. `rp2040`. Known even where the board is not.
    pub family: &'static str,
    /// `None` when several boards report this pair, so no one name is right.
    pub variant: Option<&'static str>,
}}

/// Sorted by `key`, for [`slice::binary_search_by_key`].
pub static BOARD_IDS: &[BoardId] = &[
"""

FOOTER = """];
"""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "board_identify",
        type=pathlib.Path,
        help="path to a board-identify checkout",
    )
    parser.add_argument(
        "--out",
        type=pathlib.Path,
        default=pathlib.Path("crates/wuim-probe/src/board_ids.rs"),
    )
    args = parser.parse_args()

    source = args.board_identify / "src" / "board_identify" / "arduino_ids.py"
    if not source.is_file():
        print(f"no arduino_ids.py under {args.board_identify}", file=sys.stderr)
        return 1

    table = read_table(source)
    rows = sorted(
        (vid << 16 | pid, family, variant)
        for (vid, pid), (family, _platform, variant) in table.items()
        if family != "espressif"
    )

    lines = [HEADER.format(entries=len(table))]
    for key, family, variant in rows:
        rendered = f'Some("{variant}")' if variant else "None"
        lines.append(
            f'    BoardId {{ key: 0x{key:08X}, family: "{family}", variant: {rendered} }},\n'
        )
    lines.append(FOOTER)

    args.out.write_text("".join(lines), encoding="utf-8", newline="\n")
    print(f"{args.out}: {len(rows)} pairs from {len(table)}")
    return 0


def read_table(source: pathlib.Path) -> dict:
    """Reads the dict without importing the package it lives in."""
    tree = ast.parse(source.read_text(encoding="utf-8"))
    for node in tree.body:
        target = getattr(node, "target", None)
        if getattr(target, "id", None) == "ARDUINO_USB_IDS":
            return ast.literal_eval(node.value)
    raise SystemExit("ARDUINO_USB_IDS not found")


if __name__ == "__main__":
    raise SystemExit(main())
