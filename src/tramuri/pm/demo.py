"""Reproducible offline demonstration input."""

from tramuri.core.models import Mode
from tramuri.pm.runner import PMRunner, ProgressCallback
from tramuri.storage.database import Database

DEMO_IDEA = "Quero criar um SaaS para agendamento de serviços."


def run_demo(
    database: Database, mode: Mode = Mode.MVP, progress: ProgressCallback | None = None
) -> str:
    return PMRunner(database, progress).run(DEMO_IDEA, mode, provider="local-demo")
