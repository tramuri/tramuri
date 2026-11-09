"""Intent parsing and product-archetype classification.

This module deliberately produces an operational hypothesis, not a claimed
market truth. It gives the rest of the workflow enough context to avoid
substituting an unrelated demo journey for the user's idea.
"""

from __future__ import annotations

import re
import unicodedata
from dataclasses import asdict, dataclass
from typing import Any


@dataclass(frozen=True, slots=True)
class IntentProfile:
    title: str
    intent: str
    audience: tuple[str, ...]
    desired_outcome: str
    hypotheses: tuple[str, ...]
    constraints: tuple[str, ...]
    archetype: str
    archetype_confidence: str
    raw: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self) | {
            "audience": list(self.audience),
            "hypotheses": list(self.hypotheses),
            "constraints": list(self.constraints),
        }


def _sections(text: str) -> dict[str, str]:
    found: dict[str, str] = {}
    matches = list(re.finditer(r"^\s*#{1,6}\s+(.+?)\s*$", text, re.MULTILINE))
    for index, match in enumerate(matches):
        name = _key(match.group(1))
        start = match.end()
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        found[name] = text[start:end].strip()
    return found


def _key(value: str) -> str:
    """Make section matching tolerant of accents and terminal encoding damage."""
    return unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode("ascii").casefold()


def _section(sections: dict[str, str], *names: str) -> str:
    wanted = tuple(_key(name) for name in names)
    for key, value in sections.items():
        if key in wanted or any(key.startswith(name) for name in wanted):
            return value
    return ""


def _items(value: str) -> tuple[str, ...]:
    items = [re.sub(r"^[-*•]\s*", "", line).strip() for line in value.splitlines() if line.strip()]
    return tuple(item for item in items if item)


def _title(text: str) -> str:
    heading = re.search(r"^#\s+(.+?)\s*$", text, re.MULTILINE)
    if heading:
        title = heading.group(1).strip()
        if title.casefold() not in {"ideia", "ideia de produto", "product idea"}:
            return title
    first = next((line.strip() for line in text.splitlines() if line.strip()), "")
    first = re.sub(r"^(ideia|product idea)\s*:\s*", "", first, flags=re.IGNORECASE)
    first = re.sub(
        r"^(quero criar|quero construir|i want to build|i want to create)\s+",
        "",
        first,
        flags=re.IGNORECASE,
    )
    first = re.sub(r"^uma\s+", "", first, flags=re.IGNORECASE)
    return first[:1].upper() + first[1:100]


def _classify(text: str) -> tuple[str, str]:
    normalized = text.casefold()
    learning = ("curso", "cursos", "course", "learning", "aula", "aluno", "student", "trilha")
    appointment = (
        "agend",
        "appointment",
        "consulta",
        "booking",
        "serviço",
        "service",
        "horário",
        "schedule",
    )
    learning_score = sum(normalized.count(term) for term in learning)
    appointment_score = sum(normalized.count(term) for term in appointment)
    if learning_score > appointment_score and learning_score:
        return "learning", "high" if learning_score >= 2 else "medium"
    if appointment_score and appointment_score >= learning_score:
        return "appointment", "high" if appointment_score >= 2 else "medium"
    return "generic", "low"


def parse_intent(text: str) -> IntentProfile:
    """Parse supplied intent into a bounded profile without inventing a product."""
    raw = text.strip()
    sections = _sections(raw)
    intent = _section(sections, "intencao", "intent") or raw
    outcome = (
        _section(sections, "resultado desejado", "desired outcome", "objetivo")
        or "Resultado desejado ainda não foi explicitado; registrar como hipótese."
    )
    audience_value = _section(sections, "publico", "publico-alvo", "target users", "audience")
    audience = _items(audience_value) if audience_value else ()
    if not audience:
        lower = raw.casefold()
        if "alun" in lower or "student" in lower:
            audience = ("Alunos",)
        elif "criador" in lower or "creator" in lower:
            audience = ("Criadores independentes",)
        else:
            audience = ("Publico primario a validar com evidencia real",)
    hypothesis_value = _section(sections, "hipoteses iniciais", "initial hypotheses")
    hypotheses = _items(hypothesis_value) if hypothesis_value else ()
    constraint_value = _section(sections, "restricoes", "constraints")
    constraints = _items(constraint_value) if constraint_value else ()
    archetype, confidence = _classify(" ".join((raw, intent, outcome, " ".join(audience))))
    return IntentProfile(
        title=_title(raw) or "Unnamed product idea",
        intent=intent.strip(),
        audience=audience,
        desired_outcome=outcome.strip(),
        hypotheses=hypotheses,
        constraints=constraints,
        archetype=archetype,
        archetype_confidence=confidence,
        raw=raw,
    )
