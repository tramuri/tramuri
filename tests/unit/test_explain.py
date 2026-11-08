from pathlib import Path

from tramuri.core.explain import explain_run


def test_explain_uses_artifacts_and_dependencies_without_inventing_facts() -> None:
    details = {
        "run": {"id": "run-1", "status": "completed", "mode": "mvp", "provider": "codex"},
        "artifacts": [
            {
                "kind": "pm.framing",
                "revision": 1,
                "status": "valid",
                "dependencies": [],
                "payload": {
                    "value_proposition": "Help teams schedule appointments.",
                    "classification": "hypothesis",
                },
            },
            {
                "kind": "pm.shaping",
                "revision": 1,
                "status": "valid",
                "dependencies": ["framing"],
                "payload": {"in_scope": ["Booking flow"], "classification": "decision"},
            },
        ],
        "decisions": [],
        "hypotheses": [],
        "evidence": [],
        "knowledge": [],
    }

    result = explain_run(details, "value proposition")

    assert result["found"] is True
    assert result["sources"][0]["artifact"] == "pm.framing"
    assert result["sources"][0]["classification"] == "hypothesis"
    assert result["connections"][0]["to"] == "pm.framing"
    assert result["unknowns"]


def test_explain_reports_unknown_subject_instead_of_fabricating(tmp_path: Path) -> None:
    result = explain_run(
        {
            "run": {"id": "run-1", "status": "completed", "mode": "mvp"},
            "artifacts": [],
            "decisions": [],
            "hypotheses": [],
            "evidence": [],
            "knowledge": [],
        },
        "unrecorded capability",
    )

    assert result["found"] is False
    assert result["connections"] == []
    assert "No persisted Tramuri evidence" in result["unknowns"][0]
