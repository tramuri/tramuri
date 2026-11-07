"""Conflict-aware project-local component installer."""

from __future__ import annotations

import json
from pathlib import Path

from tramuri import __version__
from tramuri.core.errors import ProviderConflictError
from tramuri.core.models import InstallationComponent, InstallationPlan, now_iso
from tramuri.core.safety import atomic_write, backup_file, project_root, safe_child, sha256_bytes
from tramuri.providers.base import TRAMURI_BLOCK_END, TRAMURI_BLOCK_START
from tramuri.storage.database import Database


class Installer:
    def __init__(self, project_dir: Path, database: Database):
        self.project_dir = project_root(project_dir)
        self.database = database
        self.manifest_dir = self.project_dir / ".tramuri" / "manifests"
        self.backup_dir = self.project_dir / ".tramuri" / "backups"

    def manifest_path(self, provider: str) -> Path:
        return self.manifest_dir / f"{provider}.json"

    def _load_manifest(self, provider: str) -> dict[str, object] | None:
        path = self.manifest_path(provider)
        if not path.is_file():
            return None
        loaded = json.loads(path.read_text(encoding="utf-8"))
        return loaded if isinstance(loaded, dict) else None

    def _merged_content(self, component: InstallationComponent, destination: Path) -> str:
        if component.merge_text:
            incoming = component.content.strip() + "\n"
            if not destination.exists():
                return incoming
            current = destination.read_text(encoding="utf-8")
            start = current.find(TRAMURI_BLOCK_START)
            end = current.find(TRAMURI_BLOCK_END)
            if (start == -1) != (end == -1) or (start != -1 and end < start):
                raise ProviderConflictError(
                    f"Incomplete Tramuri managed block: {component.relative_path}"
                )
            if start != -1 and end != -1:
                end += len(TRAMURI_BLOCK_END)
                parts = [current[:start].strip(), incoming.strip(), current[end:].strip()]
                return "\n\n".join(part for part in parts if part) + "\n"
            if not current.strip():
                return incoming
            return current.rstrip() + "\n\n" + incoming
        if not component.merge_json:
            return component.content
        incoming = json.loads(component.content)
        if not destination.exists():
            return json.dumps(incoming, ensure_ascii=False, indent=2) + "\n"
        current = json.loads(destination.read_text(encoding="utf-8"))
        if not isinstance(current, dict) or not isinstance(incoming, dict):
            raise ProviderConflictError(f"JSON merge requires objects: {component.relative_path}")
        merged = dict(current)
        for key, value in incoming.items():
            if key == "hooks" and isinstance(value, dict) and isinstance(merged.get(key), dict):
                hooks = dict(merged[key])
                for hook_key, hook_value in value.items():
                    existing = hooks.get(hook_key, [])
                    if not isinstance(existing, list) or not isinstance(hook_value, list):
                        raise ProviderConflictError(
                            f"Cannot merge hook section: {component.relative_path}"
                        )
                    hooks[hook_key] = existing + [
                        item for item in hook_value if item not in existing
                    ]
                merged[key] = hooks
            else:
                merged[key] = value
        return json.dumps(merged, ensure_ascii=False, indent=2) + "\n"

    def plan(self, installation: InstallationPlan) -> list[dict[str, object]]:
        result: list[dict[str, object]] = []
        for component in installation.components:
            destination = safe_child(self.project_dir, component.relative_path)
            content = self._merged_content(component, destination)
            existing_content = (
                destination.read_text(encoding="utf-8") if destination.is_file() else None
            )
            result.append(
                {
                    "key": component.key,
                    "path": component.relative_path,
                    "hash": sha256_bytes(content.encode()),
                    "exists": destination.exists(),
                    "changed": existing_content != content,
                    "optional": component.optional,
                    "merge_text": component.merge_text,
                }
            )
        return result

    def install(
        self, installation: InstallationPlan, dry_run: bool = False, force: bool = False
    ) -> dict[str, object]:
        previous = self._load_manifest(installation.provider)
        items = self.plan(installation)
        previous_components = previous.get("components", []) if previous else []
        if not isinstance(previous_components, list):
            previous_components = []
        previous_items = {
            str(item["path"]): item
            for item in previous_components
            if isinstance(item, dict) and "path" in item
        }
        components_by_path = {
            component.relative_path: component for component in installation.components
        }
        conflicts: list[str] = []
        for item in items:
            destination = self.project_dir / str(item["path"])
            old = previous_items.get(str(item["path"]))
            if (
                destination.exists()
                and item["changed"]
                and not force
                and not components_by_path[str(item["path"])].merge_json
                and not components_by_path[str(item["path"])].merge_text
                and (not old or old.get("hash") != sha256_bytes(destination.read_bytes()))
            ):
                conflicts.append(str(item["path"]))
        if conflicts:
            raise ProviderConflictError("Managed component conflicts: " + ", ".join(conflicts))
        if dry_run:
            return {
                "provider": installation.provider,
                "dry_run": True,
                "components": items,
                "notes": installation.notes,
            }
        installed: list[dict[str, object]] = []
        for component in installation.components:
            destination = safe_child(self.project_dir, component.relative_path)
            content = self._merged_content(component, destination)
            if destination.is_file() and destination.read_text(encoding="utf-8") == content:
                installed.append(
                    {
                        "key": component.key,
                        "path": component.relative_path,
                        "hash": sha256_bytes(content.encode()),
                        "merge_text": component.merge_text,
                        "installed_at": now_iso(),
                    }
                )
                continue
            if destination.exists():
                backup_file(destination, self.backup_dir / installation.provider)
            atomic_write(destination, content)
            installed.append(
                {
                    "key": component.key,
                    "path": component.relative_path,
                    "hash": sha256_bytes(content.encode()),
                    "merge_text": component.merge_text,
                    "installed_at": now_iso(),
                }
            )
        manifest: dict[str, object] = {
            "provider": installation.provider,
            "tramuri_version": __version__,
            "created_at": now_iso(),
            "components": installed,
            "notes": installation.notes,
        }
        atomic_write(
            self.manifest_path(installation.provider),
            json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        )
        self.database.save_manifest(
            installation.provider,
            str(self.manifest_path(installation.provider)),
            __version__,
            installed,
        )
        return manifest

    def uninstall(self, provider: str, force: bool = False) -> dict[str, object]:
        manifest = self._load_manifest(provider)
        if not manifest:
            return {"provider": provider, "removed": [], "preserved": [], "missing": True}
        removed: list[str] = []
        preserved: list[str] = []
        components = manifest.get("components", [])
        if not isinstance(components, list):
            components = []
        for item in components:
            if not isinstance(item, dict) or "path" not in item:
                continue
            destination = safe_child(self.project_dir, str(item["path"]))
            if not destination.is_file():
                continue
            if item.get("merge_text"):
                current = destination.read_text(encoding="utf-8")
                start = current.find(TRAMURI_BLOCK_START)
                end = current.find(TRAMURI_BLOCK_END)
                if start != -1 and end > start:
                    end += len(TRAMURI_BLOCK_END)
                    remaining = (current[:start].rstrip() + "\n" + current[end:].lstrip()).strip()
                    if remaining:
                        atomic_write(destination, remaining + "\n")
                    else:
                        destination.unlink()
                    removed.append(str(item["path"]))
                    continue
            if force or sha256_bytes(destination.read_bytes()) == item.get("hash"):
                destination.unlink()
                removed.append(str(item["path"]))
            else:
                preserved.append(str(item["path"]))
        if not preserved:
            self.manifest_path(provider).unlink(missing_ok=True)
        return {"provider": provider, "removed": removed, "preserved": preserved, "missing": False}
