from tramuri.core.errors import ValidationError
from tramuri.core.models import Mode, StageStatus
from tramuri.core.workflow import dependencies_satisfied, stage_specs, validate_transition


def test_modes_have_distinct_stage_shapes() -> None:
    poc = [stage.key for stage in stage_specs(Mode.POC)]
    mvp = [stage.key for stage in stage_specs(Mode.MVP)]
    enterprise = [stage.key for stage in stage_specs(Mode.ENTERPRISE)]
    assert "experiment" in poc and "navigation" not in poc
    assert "navigation" in mvp and "governance" not in mvp
    assert "governance" in enterprise and "risk_review" in enterprise


def test_transition_and_dependencies_are_explicit() -> None:
    assert validate_transition(StageStatus.PENDING, StageStatus.RUNNING).new == StageStatus.RUNNING
    assert dependencies_satisfied(stage_specs(Mode.MVP)[1], {"intent"})
    try:
        validate_transition(StageStatus.COMPLETED, StageStatus.RUNNING)
    except ValidationError:
        pass
    else:
        raise AssertionError("completed stages must not be restarted")
