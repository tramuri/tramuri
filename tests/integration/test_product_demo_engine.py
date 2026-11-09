import json
from pathlib import Path

import pytest

from tramuri.experience.classifier import ProductExperienceClassifier
from tramuri.experience.demo import ProductDemoBuilder
from tramuri.experience.registry import DemoTemplate
from tramuri.experience.renderer import default_template_registry
from tramuri.experience.sensors import VisualQualityValidator


def _context(
    idea: str,
    *,
    platforms: list[str],
    audience: list[str],
    features: list[str],
    flows: list[str],
) -> dict[str, object]:
    return {
        "run": {"id": "fixture-run", "idea": idea, "mode": "mvp"},
        "artifacts": [
            {
                "kind": "pm.framing",
                "payload": {
                    "product_name": "Fixture Product",
                    "description": idea,
                    "target_users": audience,
                    "platforms": platforms,
                    "features": features,
                    "core_journey": flows,
                },
            },
            {
                "kind": "pm.shaping",
                "payload": {"in_scope": features, "out_of_scope": []},
            },
            {
                "kind": "pm.ux",
                "payload": {"audience": audience, "journeys": flows},
            },
        ],
        "decisions": ["Keep external integrations simulated until validated."],
        "evidence": [],
        "hypotheses": [],
    }


PRODUCT_CASES = [
    (
        "mobile-finance",
        _context(
            "Mobile finance app for people who track daily spending and plan a budget.",
            platforms=["iOS", "Android"],
            audience=["People managing household budgets"],
            features=["Record an expense", "Review spending by category", "Set a monthly budget"],
            flows=["Add expense, see the budget update, and review the month"],
        ),
        "mobile_app",
        "mobile",
    ),
    (
        "developer-cli",
        _context(
            "A command-line developer tool for initializing and inspecting Python projects.",
            platforms=["CLI", "terminal"],
            audience=["Python developers"],
            features=["Initialize a project", "Inspect project status", "Print diagnostics"],
            flows=["Install the tool, run init, then inspect status"],
        ),
        "cli_developer_tool",
        "cli",
    ),
    (
        "project-saas",
        _context(
            "A web SaaS workspace that helps distributed teams plan projects and move tasks.",
            platforms=["Browser", "Web"],
            audience=["Project teams", "Team leads"],
            features=["Create a project", "Assign a task", "Move work across a project board"],
            flows=["Create project, assign work, and review delivery"],
        ),
        "web_saas",
        "web",
    ),
    (
        "data-api",
        _context(
            "A backend API that accepts event batches, normalizes records, and returns a processing summary.",
            platforms=["REST API", "backend"],
            audience=["Developers integrating data services"],
            features=["POST /events", "Validate a batch", "Return a normalized response"],
            flows=["Send a request, process the event batch, and inspect the response"],
        ),
        "api_backend",
        "api",
    ),
    (
        "field-hybrid",
        _context(
            "A field service product combines a mobile app for technicians with a web dashboard for operations.",
            platforms=["Android mobile app", "Browser dashboard"],
            audience=["Field technicians", "Operations managers"],
            features=[
                "Capture an equipment reading",
                "Review fleet status",
                "Acknowledge an alert",
            ],
            flows=["Technician records a reading; operations reviews and acknowledges it"],
        ),
        "hybrid",
        "hybrid",
    ),
    (
        "desktop-application",
        _context(
            "A native desktop application for reviewing and editing long-form research notes.",
            platforms=["Windows desktop"],
            audience=["Research analysts"],
            features=["Organize notes", "Edit a source outline", "Save workspace preferences"],
            flows=["Open a project, edit notes, and save the local workspace"],
        ),
        "desktop_app",
        "desktop",
    ),
    (
        "data-pipeline",
        _context(
            "An ETL data pipeline ingests event records, transforms them, and publishes a batch result.",
            platforms=["Scheduled data pipeline"],
            audience=["Data engineers"],
            features=["Ingest source rows", "Transform events", "Publish the output"],
            flows=["Run the batch and inspect each processing stage"],
        ),
        "data_ai_pipeline",
        "pipeline",
    ),
    (
        "iot-device",
        _context(
            "An IoT device monitoring system reads equipment sensors and reports temperature telemetry.",
            platforms=["IoT hardware dashboard"],
            audience=["Facility operators"],
            features=["Read sensor telemetry", "Show device status", "Review threshold alerts"],
            flows=["Adjust a sensor reading and review the device state"],
        ),
        "iot_hardware",
        "iot",
    ),
    (
        "contextual-other",
        _context(
            "A guided community workshop experience for neighbors who want to exchange practical skills.",
            platforms=[],
            audience=["Local community members"],
            features=["Propose a workshop", "Join a session", "Share a learning outcome"],
            flows=["Find a session, participate, and share a result"],
        ),
        "other",
        "other",
    ),
]


@pytest.mark.parametrize(
    ("name", "details", "category", "template"),
    PRODUCT_CASES,
    ids=[case[0] for case in PRODUCT_CASES],
)
def test_product_demo_selects_contextual_template_and_versioned_spec(
    tmp_path: Path,
    name: str,
    details: dict[str, object],
    category: str,
    template: str,
) -> None:
    result = ProductDemoBuilder(tmp_path / name).build(details)  # type: ignore[arg-type]
    serialized = result.to_dict()
    root = Path(str(serialized["root"]))
    demo_html = Path(str(serialized["index"])).read_text(encoding="utf-8")
    spec = json.loads((root / "demo_spec.json").read_text(encoding="utf-8"))
    traceability = json.loads((root / "traceability.json").read_text(encoding="utf-8"))

    assert serialized["category"] == category
    assert spec["version"] == "tramuri.demo.v1"
    assert spec["classification"]["category"] == category
    assert spec["template_id"] == template
    assert traceability["template_id"] == template
    assert f'data-demo-category="{category}"' in demo_html
    assert "data-simulation-level=" in demo_html
    assert "Simulated" in demo_html or "simulation" in demo_html.casefold()
    assert spec["screens"] and spec["interactions"]
    assert spec["classification"]["evidence"]
    if name == "mobile-finance":
        assert 'role="tablist" aria-label="App navigation"' in demo_html
        assert 'data-mobile-view="activity"' in demo_html
        assert 'data-mobile-screen="activity"' in demo_html
        assert 'data-action="add-expense"' in demo_html
    assert all(
        result.status == "passed"
        for result in VisualQualityValidator().run(root, details)  # type: ignore[arg-type]
    )


def test_ambiguous_classification_is_recorded_and_can_be_corrected(tmp_path: Path) -> None:
    details = _context(
        "A workspace to help teams make progress visible and coordinate decisions.",
        platforms=[],
        audience=["Small teams"],
        features=["Share updates", "Review decisions"],
        flows=["Post an update and review a decision"],
    )

    inferred = ProductDemoBuilder(tmp_path / "inferred").build(details)  # type: ignore[arg-type]
    corrected_details = details | {"experience_category_override": "desktop_app"}
    corrected = ProductDemoBuilder(tmp_path / "corrected").build(  # type: ignore[arg-type]
        corrected_details
    )

    inferred_spec = json.loads((inferred.root / "demo_spec.json").read_text(encoding="utf-8"))
    corrected_spec = json.loads((corrected.root / "demo_spec.json").read_text(encoding="utf-8"))
    assert inferred_spec["classification"]["ambiguous"] is True
    assert corrected_spec["classification"]["category"] == "desktop_app"
    assert corrected_spec["classification"]["override"] == "desktop_app"


def test_demo_can_be_regenerated_from_spec_without_pdlc_context(tmp_path: Path) -> None:
    details = dict(PRODUCT_CASES[0][1])
    first = ProductDemoBuilder(tmp_path / "regenerate").build(details)  # type: ignore[arg-type]
    spec_path = first.root / "demo_spec.json"
    first.index.unlink()

    regenerated = ProductDemoBuilder(tmp_path / "regenerate").render_spec(spec_path)

    assert regenerated.index.is_file()
    assert 'data-demo-category="mobile_app"' in regenerated.index.read_text(encoding="utf-8")


def test_marketplace_uses_commerce_journey_instead_of_learning_catalog(tmp_path: Path) -> None:
    details = _context(
        "An online marketplace where independent makers list home goods for shoppers.",
        platforms=[],
        audience=["Independent makers", "Shoppers"],
        features=["Browse listings", "Add an item to the basket", "Checkout locally"],
        flows=["Browse the catalog, review an item, and complete a sample checkout"],
    )

    result = ProductDemoBuilder(tmp_path / "marketplace").build(details)  # type: ignore[arg-type]
    spec = json.loads((result.root / "demo_spec.json").read_text(encoding="utf-8"))
    html = result.index.read_text(encoding="utf-8")

    assert result.category == "web_saas"
    assert spec["domain"] == "marketplace"
    assert spec["template_id"] == "web"
    assert "Shop by collection" in html
    assert 'data-action="add-to-cart"' in html
    assert "Applied AI Foundations" not in html
    assert all(
        sensor.status == "passed"
        for sensor in VisualQualityValidator().run(result.root, details)  # type: ignore[arg-type]
    )


def test_learning_marketplace_keeps_the_education_journey(tmp_path: Path) -> None:
    details = _context(
        "A learning marketplace where course creators publish lessons for learners.",
        platforms=["Browser"],
        audience=["Course creators", "Learners"],
        features=["Browse courses", "Enroll in a path", "Mark lesson progress"],
        flows=["Choose a course, enroll, and complete a lesson"],
    )

    result = ProductDemoBuilder(tmp_path / "learning-marketplace").build(details)  # type: ignore[arg-type]
    spec = json.loads((result.root / "demo_spec.json").read_text(encoding="utf-8"))
    html = result.index.read_text(encoding="utf-8")

    assert result.category == "web_saas"
    assert spec["domain"] == "education"
    assert 'data-action="enroll"' in html
    assert "LEARNER JOURNEY" in html
    assert "Sample listings · synthetic" not in html


def test_android_mobile_context_uses_android_device_chrome(tmp_path: Path) -> None:
    details = _context(
        "An Android mobile app for field technicians to record equipment visits.",
        platforms=["Android"],
        audience=["Field technicians"],
        features=["Record a visit", "Review equipment status"],
        flows=["Open the equipment record and save a local field update"],
    )

    result = ProductDemoBuilder(tmp_path / "android-mobile").build(details)  # type: ignore[arg-type]
    spec = json.loads((result.root / "demo_spec.json").read_text(encoding="utf-8"))
    html = result.index.read_text(encoding="utf-8")

    assert result.category == "mobile_app"
    assert spec["platform"] == "Android"
    assert 'class="device-shell android-device"' in html
    assert 'aria-label="Android smartphone product simulation"' in html
    assert "This Android proposal uses Android device chrome" in html
    assert "device-island" not in html


def test_cli_simulator_rejects_commands_outside_predefined_scenarios(tmp_path: Path) -> None:
    details = dict(PRODUCT_CASES[1][1])
    result = ProductDemoBuilder(tmp_path / "safe-cli").build(details)  # type: ignore[arg-type]
    script = (result.root / "app.js").read_text(encoding="utf-8")

    assert "Command is not available in this simulation" in script
    assert "rm -rf /" not in script
    assert "eval(" not in script


def test_new_category_can_register_its_own_classifier_and_template(tmp_path: Path) -> None:
    registry = default_template_registry()
    classifier = ProductExperienceClassifier()
    classifier.register_category("wearable_device", (("smartwatch", 2.8),))
    registry.register(
        DemoTemplate(
            "wearable",
            ("wearable_device",),
            lambda spec: {
                "index.html": (
                    '<!doctype html><html lang="en"><head><meta charset="utf-8">'
                    '<meta name="viewport" content="width=device-width,initial-scale=1">'
                    '</head><body data-demo-category="wearable_device" '
                    'data-simulation-level="interactive_simulation"><main id="screen">'
                    f"{spec['product_name']}</main></body></html>"
                )
            },
            screens=(("screen", "Watch", "Review a glanceable update"),),
            interactions=(("refresh", "Refresh sample", "Update a local reading"),),
        )
    )
    builder = ProductDemoBuilder(tmp_path / "wearable", classifier=classifier, registry=registry)
    details = _context(
        "A smartwatch companion that summarizes a runner's daily recovery state.",
        platforms=["smartwatch"],
        audience=["Runners"],
        features=["Review recovery status"],
        flows=["Open the watch to read a recovery summary"],
    )

    result = builder.build(details)  # type: ignore[arg-type]

    assert result.category == "wearable_device"
    assert result.template_id == "wearable"
    assert (
        "Review a glanceable update"
        in json.loads(
            result.specification.read_text(encoding="utf-8")  # type: ignore[union-attr]
        )["screens"][0]["purpose"]
    )
