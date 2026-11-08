from __future__ import annotations

from tramuri.knowledge.service import KnowledgeService


def test_knowledge_ingestion_skips_secrets_symlinks_and_oversized_files(database, tmp_path) -> None:
    service = KnowledgeService(database, max_bytes=10)
    secret = tmp_path / ".env"
    secret.write_text("TOKEN=secret", encoding="utf-8")
    assert service.ingest_file(secret) is None
    large = tmp_path / "large.md"
    large.write_text("01234567890", encoding="utf-8")
    assert service.ingest_file(large) is None
