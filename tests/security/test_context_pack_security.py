from __future__ import annotations

from tramuri.core.context import build_context_pack
from tramuri.core.models import Mode


def test_context_pack_redacts_common_credentials_and_bounds_lists(database) -> None:
    run_id = database.create_run("A product", Mode.POC)
    details = database.run_details(run_id)
    details["events"] = [{"payload": "token=abc123", "id": str(index)} for index in range(200)]
    pack = build_context_pack(
        details,
        knowledge=[{"content": "Authorization: Bearer abc.def.ghi", "title": "input"}],
        max_bytes=4_000,
    )
    serialized = str(pack).lower()
    assert "abc123" not in serialized
    assert "bearer abc" not in serialized
    assert len(str(pack).encode("utf-8")) <= 4_000
