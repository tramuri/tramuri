"""Extensible registry for category-specific Product Demo templates."""

from __future__ import annotations

from collections.abc import Callable, Mapping
from dataclasses import dataclass
from typing import Any

TemplateRenderer = Callable[[Mapping[str, Any]], dict[str, str]]


@dataclass(frozen=True, slots=True)
class DemoTemplate:
    template_id: str
    categories: tuple[str, ...]
    render: TemplateRenderer
    screens: tuple[tuple[str, str, str], ...] = ()
    interactions: tuple[tuple[str, str, str], ...] = ()


class DemoTemplateRegistry:
    """Register independent renderers without changing the engine pipeline."""

    def __init__(self) -> None:
        self._by_id: dict[str, DemoTemplate] = {}
        self._by_category: dict[str, DemoTemplate] = {}

    def register(self, template: DemoTemplate) -> None:
        """Add a template; duplicate ids or category ownership are rejected."""
        if not template.template_id.strip() or not template.categories:
            raise ValueError("A demo template needs an id and at least one category")
        if template.template_id in self._by_id:
            raise ValueError(f"Demo template {template.template_id!r} is already registered")
        duplicates = [category for category in template.categories if category in self._by_category]
        if duplicates:
            raise ValueError(f"Demo categories already have templates: {', '.join(duplicates)}")
        self._by_id[template.template_id] = template
        for category in template.categories:
            self._by_category[category] = template

    def for_category(self, category: str) -> DemoTemplate:
        """Resolve the registered template for a classified product category."""
        try:
            return self._by_category[category]
        except KeyError as exc:
            raise ValueError(f"No Product Demo template is registered for {category!r}") from exc

    def by_id(self, template_id: str) -> DemoTemplate:
        """Resolve a template when replaying a persisted versioned spec."""
        try:
            return self._by_id[template_id]
        except KeyError as exc:
            raise ValueError(f"No Product Demo template is registered as {template_id!r}") from exc

    def supports(self, category: str) -> bool:
        return category in self._by_category
