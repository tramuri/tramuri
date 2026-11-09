from tramuri.reports.presentation import build_relationship_graph, safe_http_url


def test_relationship_graph_uses_only_persisted_links() -> None:
    details = {
        "stages": [
            {
                "id": "stage-framing",
                "stage_key": "framing",
                "position": 0,
                "status": "completed",
                "dependencies": [],
            },
            {
                "id": "stage-research",
                "stage_key": "research",
                "position": 1,
                "status": "completed",
                "dependencies": ["framing"],
            },
        ],
        "artifacts": [
            {
                "id": "artifact-framing",
                "kind": "pm.framing",
                "stage_id": "stage-framing",
                "status": "valid",
                "dependencies": [],
            }
        ],
        "evidence": [
            {
                "id": "evidence-market",
                "claim": "A recorded market signal",
                "source_ref": "source-1",
                "classification": "evidence",
            }
        ],
        "decisions": [
            {
                "id": "decision-linked",
                "title": "Use the recorded signal",
                "source_ref": "evidence-market",
                "classification": "decision",
            },
            {
                "id": "decision-unlinked",
                "title": "No evidence reference",
                "source_ref": None,
                "classification": "decision",
            },
        ],
        "hypotheses": [{"id": "hypothesis-1", "statement": "Test a need"}],
    }

    graph = build_relationship_graph(details)
    links = {(edge["source"], edge["target"], edge["kind"]) for edge in graph["edges"]}

    assert ("stage:framing", "stage:research", "stage_dependency") in links
    assert ("stage:framing", "artifact:artifact-framing", "produced") in links
    assert ("evidence:evidence-market", "decision:decision-linked", "source_reference") in links
    assert not any("hypothesis" in source or "hypothesis" in target for source, target, _ in links)
    assert len(graph["nodes"]) == 7


def test_safe_http_url_rejects_non_web_schemes() -> None:
    assert safe_http_url("https://research.example/report") == "https://research.example/report"
    assert safe_http_url("http://research.example/report") == "http://research.example/report"
    assert safe_http_url("javascript:alert(1)") is None
    assert safe_http_url("//research.example/report") is None
    assert safe_http_url("https:///missing-host") is None
    assert safe_http_url("https://user:password@research.example/report") is None
    assert safe_http_url("https://research.example/report?token=private") is None
