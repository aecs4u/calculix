from __future__ import annotations

from pathlib import Path

import pytest

from calculix_migration_tooling.nastran_reader import (
    main,
    read_nastran_analysis,
    summary_to_dict,
)


def test_reads_minimal_bdf_summary(repo_root: Path) -> None:
    path = repo_root / "tests" / "fixtures" / "nastran" / "minimal_static.bdf"
    summary = read_nastran_analysis(path)

    assert summary.sol == 101
    assert summary.nnodes == 4
    assert summary.nelements == 1
    assert summary.nmaterials == 1
    assert summary.nproperties == 1
    assert summary.nloads == 1
    assert summary.nspc_sets == 1
    assert summary.element_type_counts == {"CQUAD4": 1}

    payload = summary_to_dict(summary)
    assert payload["nnodes"] == 4
    assert payload["element_type_counts"]["CQUAD4"] == 1


def test_rejects_unsupported_extension(tmp_path: Path) -> None:
    bad = tmp_path / "model.inp"
    bad.write_text("*NODE\n1,0.,0.,0.\n", encoding="utf-8")
    with pytest.raises(ValueError):
        read_nastran_analysis(bad)


def test_cli_json_output(repo_root: Path, capsys: pytest.CaptureFixture[str]) -> None:
    path = repo_root / "tests" / "fixtures" / "nastran" / "minimal_static.bdf"
    rc = main([str(path), "--json"])
    out = capsys.readouterr().out

    assert rc == 0
    assert '"sol": 101' in out
    assert '"nnodes": 4' in out

