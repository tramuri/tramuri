from tramuri.experience.models import ExperienceStage, SensorResult, SkillSelection


def test_experience_stage_contract_has_four_lifecycle_states() -> None:
    stages = ExperienceStage.default_map()

    assert [stage.key for stage in stages] == [
        "product-definition",
        "product-demo",
        "product-ux",
        "product-engineering",
    ]
    assert stages[0].status == "completed"
    assert stages[2].status == "future"
    assert stages[3].status == "future"


def test_skill_selection_and_sensor_results_are_serializable() -> None:
    selection = SkillSelection("python", (), (), {"task": "mapped"}, ())
    sensor = SensorResult("links", "passed", "No broken local links", ("demo/index.html",))

    assert selection.to_dict()["task_kind"] == "python"
    assert sensor.to_dict()["status"] == "passed"
