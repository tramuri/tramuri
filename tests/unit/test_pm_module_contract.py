import pytest

from tramuri.core.errors import ValidationError
from tramuri.core.models import Mode
from tramuri.core.workflow import stage_specs
from tramuri.modules.pm.runner import PMRunner
from tramuri.modules.pm.stages import MODE_PROFILES
from tramuri.storage.database import Database


def test_pm_module_exposes_all_modes() -> None:
    assert set(MODE_PROFILES) == {Mode.POC, Mode.MVP, Mode.ENTERPRISE}
    assert stage_specs(Mode.POC)[2].key == "experiment"
    assert "governance" in {stage.key for stage in stage_specs(Mode.ENTERPRISE)}


def test_mvp_has_no_operation_stage() -> None:
    assert "operation" not in {stage.key for stage in stage_specs(Mode.MVP)}
    assert PMRunner is not None


def test_product_workflow_rejects_missing_intent(tmp_path) -> None:
    with pytest.raises(ValidationError, match="product intent"):
        PMRunner(Database(tmp_path)).run("   ", Mode.MVP)
