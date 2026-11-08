"""Deterministic knowledge retrieval with explicit provenance and safe ingestion."""

from __future__ import annotations

import hashlib
import re
from pathlib import Path
from typing import Any

from tramuri.storage.database import Database

_SENSITIVE_NAME = re.compile(r"(^|\.)(env|pem|key|credentials?|secrets?)(\.|$)", re.IGNORECASE)
_ALLOWED_SUFFIXES = {".md", ".txt", ".rst", ".json", ".yaml", ".yml"}


class KnowledgeService:
    """Read and retrieve knowledge without treating its content as executable policy."""

    def __init__(self, database: Database, *, max_bytes: int = 256_000):
        self.database = database
        self.max_bytes = max_bytes

    def record(self, **values: Any) -> str:
        return self.database.add_knowledge(**values)

    def retrieve(
        self,
        query: str,
        *,
        run_id: str | None = None,
        limit: int = 8,
    ) -> list[dict[str, Any]]:
        terms = {term for term in re.findall(r"[\w-]{3,}", query.casefold())}
        entries = self.database.list_knowledge(run_id=run_id)
        for entry in entries:
            if entry.get("source_kind") == "project_file" and entry.get("source_ref"):
                self.refresh_entry(str(entry["id"]), Path(str(entry["source_ref"])))
        entries = self.database.list_knowledge(run_id=run_id)
        scored: list[tuple[int, str, dict[str, Any]]] = []
        for entry in entries:
            haystack = f"{entry.get('title', '')} {entry.get('content', '')}".casefold()
            score = sum(1 for term in terms if term in haystack)
            if score:
                scored.append((score, str(entry.get("updated_at", "")), entry))
        scored.sort(key=lambda item: (-item[0], item[1]), reverse=False)
        return [entry for _, _, entry in scored[: max(1, min(limit, 50))]]

    def refresh_entry(self, entry_id: str, path: Path) -> str:
        """Recheck an indexed source and mark it stale/unavailable before retrieval."""
        entry = self.database.knowledge_entry(entry_id)
        if not entry:
            return "missing"
        requested = path.expanduser()
        if requested.is_symlink() or not requested.is_file():
            self.database.update_knowledge_status(
                entry_id, status="unavailable", freshness="unknown", reason="source_unreadable"
            )
            return "unavailable"
        if requested.stat().st_size > self.max_bytes:
            self.database.update_knowledge_status(
                entry_id, status="unavailable", freshness="unknown", reason="source_oversized"
            )
            return "unavailable"
        content = requested.read_text(encoding="utf-8", errors="replace")
        source_hash = hashlib.sha256(content.encode("utf-8")).hexdigest()
        if entry.get("source_hash") != source_hash:
            self.database.update_knowledge_status(
                entry_id, status="invalidated", freshness="stale", reason="source_changed"
            )
            return "stale"
        self.database.update_knowledge_status(
            entry_id, status="active", freshness="current", reason="source_verified"
        )
        return "current"

    def ingest_file(
        self,
        path: Path,
        *,
        tier: str = "project",
        run_id: str | None = None,
        classification: str = "evidence",
    ) -> str | None:
        """Ingest a bounded documentation file; skip secrets, symlinks and unknown formats."""
        requested = path.expanduser()
        candidate = requested.resolve(strict=False)
        if (
            requested.is_symlink()
            or not candidate.is_file()
            or candidate.suffix.casefold() not in _ALLOWED_SUFFIXES
            or _SENSITIVE_NAME.search(candidate.name)
        ):
            return None
        if candidate.stat().st_size > self.max_bytes:
            return None
        content = candidate.read_text(encoding="utf-8", errors="replace")
        source_hash = hashlib.sha256(content.encode("utf-8")).hexdigest()
        return self.record(
            title=candidate.name,
            content=content,
            classification=classification,
            source_kind="project_file",
            tier=tier,
            run_id=run_id,
            source_ref=str(candidate),
            source_hash=source_hash,
        )

    def ingest_project_documents(self, root: Path, *, run_id: str | None = None) -> list[str]:
        """Index a small, safe documentation surface without executing its contents."""
        candidates: list[Path] = []
        for name in ("README.md", "README.pt-BR.md", "VISION.md"):
            candidate = root / name
            if candidate.is_file():
                candidates.append(candidate)
        docs_root = root / "docs"
        if docs_root.is_dir():
            candidates.extend(
                item
                for item in sorted(docs_root.rglob("*.md"))
                if item.is_file()
                and not any(part in {".git", "node_modules"} for part in item.parts)
            )
        known = {
            str(item.get("source_ref"))
            for item in self.database.list_knowledge(run_id=run_id)
            if item.get("source_ref")
        }
        indexed: list[str] = []
        for candidate in candidates[:40]:
            if str(candidate.resolve()) in known:
                continue
            entry_id = self.ingest_file(candidate, run_id=run_id)
            if entry_id:
                indexed.append(entry_id)
        return indexed
