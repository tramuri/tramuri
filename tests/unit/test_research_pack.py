import json
from pathlib import Path

import pytest

from tramuri.core.errors import ValidationError
from tramuri.modules.pm.research import load_research_pack


def _pack() -> dict[str, object]:
    return {
        "version": "tramuri.research.v1",
        "researched_at": "2026-11-08T12:00:00Z",
        "query": "online learning platforms for independent creators",
        "sources": [
            {
                "id": "s1",
                "url": "https://example.com/report",
                "title": "Learning market report",
                "publisher": "Example Research",
                "accessed_at": "2026-11-08",
                "confidence": "medium",
            }
        ],
        "competitors": [
            {
                "name": "Example Academy",
                "positioning": "Creator-led learning",
                "strengths": ["catalog"],
                "gaps": ["progress support"],
                "source_ids": ["s1"],
            }
        ],
        "market_signals": [
            {"signal": "Creators package expertise as courses", "source_ids": ["s1"]}
        ],
        "buying_motivations": [{"motivation": "Reach a practical outcome", "source_ids": ["s1"]}],
        "build_buy_partner": [
            {"option": "build", "trade_off": "control vs. time", "source_ids": ["s1"]}
        ],
        "positioning": [
            {"statement": "Outcome-led learning for busy practitioners", "source_ids": ["s1"]}
        ],
        "market_sizing": {"status": "pending_validation", "claims": []},
        "unknowns": ["Willingness to pay requires validation."],
    }


def test_research_pack_requires_traceable_sources(tmp_path: Path) -> None:
    path = tmp_path / "research.json"
    path.write_text(json.dumps(_pack()), encoding="utf-8")

    loaded = load_research_pack(path)

    assert loaded.payload["status"] == "imported"
    assert loaded.payload["source_count"] == 1
    assert loaded.payload["competitors"][0]["source_ids"] == ["s1"]


@pytest.mark.parametrize(
    "change, message",
    [
        ({"sources": [{"id": "s1", "url": "not-a-url"}]}, "URL"),
        ({"competitors": [{"name": "Untraceable", "source_ids": ["missing"]}]}, "source"),
    ],
)
def test_research_pack_rejects_unverifiable_claims(
    tmp_path: Path, change: dict[str, object], message: str
) -> None:
    payload = _pack()
    payload.update(change)
    path = tmp_path / "invalid.json"
    path.write_text(json.dumps(payload), encoding="utf-8")

    with pytest.raises(ValidationError, match=message):
        load_research_pack(path)
