from __future__ import annotations

import importlib.util
from pathlib import Path

import pytest

from tramuri.core.errors import ValidationError
from tramuri.core.models import Mode
from tramuri.modules.pm.runner import PMRunner
from tramuri.storage.database import Database


def test_project_memory_service_exists() -> None:
    assert importlib.util.find_spec("tramuri.knowledge.memory") is not None


@pytest.fixture
def memory_service_type():
    if importlib.util.find_spec("tramuri.knowledge.memory") is None:
        pytest.skip("Project memory service is introduced by this feature.")
    from tramuri.knowledge.memory import MemoryService

    return MemoryService


def test_saved_memory_is_redacted_indexed_and_projected(
    tmp_path: Path, memory_service_type
) -> None:
    database = Database(tmp_path)
    memory = memory_service_type(database)

    saved = memory.save(
        title="Preferred working hours",
        content="The user prefers UTC-3. api_key=private-value",
        classification="fact",
        source_ref="explicit_user_request",
    )

    entry = database.list_knowledge()[0]
    assert saved["saved"] is True
    assert entry["source_kind"] == "user_memory"
    assert entry["classification"] == "fact"
    assert "private-value" not in entry["content"]
    assert saved["current_file"] == str(tmp_path / "tramuri_output" / "memorys" / "current.md")
    assert "Preferred working hours" in Path(saved["current_file"]).read_text(encoding="utf-8")
    assert memory.search("UTC-3")[0]["id"] == saved["entry_id"]


def test_completed_workflow_writes_a_retrievable_memory_snapshot(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = PMRunner(database).run("A local appointment product", Mode.POC)

    run_file = tmp_path / "tramuri_output" / "memorys" / "runs" / f"{run_id}.md"
    current_file = tmp_path / "tramuri_output" / "memorys" / "current.md"

    assert run_file.is_file()
    assert current_file.is_file()
    assert run_id in run_file.read_text(encoding="utf-8")
    assert "completed" in run_file.read_text(encoding="utf-8").lower()
    assert "Review the report and product demo" in current_file.read_text(encoding="utf-8")


def test_replacing_a_saved_memory_supersedes_the_old_record(
    tmp_path: Path, memory_service_type
) -> None:
    database = Database(tmp_path)
    memory = memory_service_type(database)
    first = memory.save(
        title="Preferred working hours",
        content="UTC-3",
        classification="fact",
    )

    replacement = memory.save(
        title="Preferred working hours",
        content="UTC-5",
        classification="fact",
    )

    assert database.knowledge_entry(first["entry_id"])["status"] == "superseded"
    assert database.knowledge_entry(replacement["entry_id"])["supersedes_id"] == first["entry_id"]
    assert [item["id"] for item in database.list_knowledge()] == [replacement["entry_id"]]


def test_showing_historical_run_preserves_current_memory(
    tmp_path: Path, memory_service_type
) -> None:
    database = Database(tmp_path)
    first_run = PMRunner(database).run("First appointment product", Mode.POC)
    current_run = PMRunner(database).run("Current appointment product", Mode.POC)

    historical = memory_service_type(database).show(run_id=first_run)

    current_text = Path(historical["current_file"]).read_text(encoding="utf-8")
    assert historical["current_run_id"] == current_run
    assert historical["run_file"].endswith(f"{first_run}.md")
    assert historical["display_file"].endswith(f"{first_run}.md")
    assert current_run in current_text
    assert first_run not in current_text


def test_memory_projection_does_not_overwrite_an_unmanaged_file(
    tmp_path: Path, memory_service_type
) -> None:
    memory_dir = tmp_path / "tramuri_output" / "memorys"
    memory_dir.mkdir(parents=True)
    current_file = memory_dir / "current.md"
    current_file.write_text("User-authored file\n", encoding="utf-8")

    with pytest.raises(ValidationError, match="unowned memory file"):
        memory_service_type(Database(tmp_path)).show()

    assert current_file.read_text(encoding="utf-8") == "User-authored file\n"


def test_memory_projection_conflict_does_not_block_product_workflow(tmp_path: Path) -> None:
    database = Database(tmp_path)
    memory_dir = tmp_path / "tramuri_output" / "memorys"
    memory_dir.mkdir(parents=True)
    (memory_dir / "current.md").write_text("User-authored file\n", encoding="utf-8")

    run_id = PMRunner(database).run("A local appointment product", Mode.POC)

    details = database.run_details(run_id)
    assert details["run"]["status"] == "completed"
    assert any(event["event_type"] == "memory.projection.failed" for event in details["events"])
