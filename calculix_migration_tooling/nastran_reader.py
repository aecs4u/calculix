from __future__ import annotations

from dataclasses import asdict, dataclass
from pathlib import Path
import argparse
import json
from typing import Any

from pyNastran.bdf.bdf import BDF


SUPPORTED_SUFFIXES = {".bdf", ".dat", ".nas"}


@dataclass(frozen=True)
class NastranSummary:
    path: str
    sol: int | None
    nnodes: int
    nelements: int
    nproperties: int
    nmaterials: int
    nloads: int
    nspc_sets: int
    nmpc_sets: int
    ncoords: int
    element_type_counts: dict[str, int]


def read_nastran_analysis(path: str | Path) -> NastranSummary:
    deck_path = Path(path).expanduser().resolve()
    suffix = deck_path.suffix.lower()
    if suffix not in SUPPORTED_SUFFIXES:
        raise ValueError(
            f"unsupported file extension '{suffix}' for {deck_path}; "
            f"expected one of {sorted(SUPPORTED_SUFFIXES)}"
        )
    if not deck_path.is_file():
        raise FileNotFoundError(deck_path)

    model = BDF(debug=False, log=None)
    model.read_bdf(str(deck_path), xref=False, validate=True)

    element_type_counts: dict[str, int] = {}
    for element in model.elements.values():
        etype = str(getattr(element, "type", "UNKNOWN"))
        element_type_counts[etype] = element_type_counts.get(etype, 0) + 1

    # Use native BDF containers to keep this deterministic and cheap.
    summary = NastranSummary(
        path=str(deck_path),
        sol=int(model.sol) if model.sol is not None else None,
        nnodes=len(model.nodes),
        nelements=len(model.elements),
        nproperties=len(model.properties),
        nmaterials=len(model.materials),
        nloads=len(model.loads),
        nspc_sets=len(model.spcs),
        nmpc_sets=len(model.mpcs),
        ncoords=len(model.coords),
        element_type_counts=dict(sorted(element_type_counts.items())),
    )
    return summary


def summary_to_dict(summary: NastranSummary) -> dict[str, Any]:
    return asdict(summary)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Read and summarize a Nastran analysis deck using pyNastran."
    )
    parser.add_argument("path", help="Path to .bdf/.dat/.nas analysis deck")
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit JSON summary (default is plain key/value text).",
    )
    args = parser.parse_args(argv)

    summary = read_nastran_analysis(args.path)
    payload = summary_to_dict(summary)

    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        for key, value in payload.items():
            print(f"{key}: {value}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

