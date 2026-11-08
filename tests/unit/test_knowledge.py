from __future__ import annotations

from tramuri.knowledge.service import KnowledgeService


def test_knowledge_retrieval_is_classified_and_source_aware(database, tmp_path) -> None:
    source = tmp_path / "product.md"
    source.write_text("The first release is local-first.", encoding="utf-8")
    service = KnowledgeService(database)
    entry_id = service.ingest_file(source)
    assert entry_id
    result = service.retrieve("local-first")
    assert result[0]["source_ref"] == str(source.resolve())
    assert result[0]["freshness"] == "current"

    source.write_text("The first release is hosted.", encoding="utf-8")
    assert service.retrieve("hosted") == []
    assert database.knowledge_entry(entry_id)["status"] == "invalidated"
