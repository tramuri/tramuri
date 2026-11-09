# Premium Product Reports — Verification Evidence

## Automated checks

| Area | Evidence | Result |
| --- | --- | --- |
| Python suite | `uv run pytest -q` | 102 passed; browser modules skipped because Playwright is not a product dependency |
| Browser flows | `uv --native-tls run --with playwright pytest tests/web/test_report_browser.py tests/web/test_product_experience_browser.py -q` | 3 passed, including desktop/mobile report behavior and existing product journeys |
| Lint | `uv run ruff check .` | Passed |
| Formatting | `uv run ruff format --check` for the 12 Python files changed in this feature | Passed |
| Types | `uv run mypy` | Passed; 63 source files |
| Harness | `python harness/scripts/validate_harness.py` | Passed; 42 local skills and TOML configuration |
| Visual detector | `impeccable.cmd detect --json src/tramuri/reports/templates/report.html src/tramuri/reports/templates/report.css` | No findings |
| Interface review | Manual review against Web Interface Guidelines and computed token pairs | Semantic landmarks and heading order, labeled controls, keyboard focus, touch targets, table headers, reduced motion, offline behavior and narrow layouts reviewed; ordinary text pairs measured at 4.71:1 or higher in both themes |
| Browser acceptance | `test_report_browser.py` | Language, theme, anchor/current-section navigation, graph targets, keyboard focus, reduced motion, responsive overflow, mobile graph scrolling, print CSS, no page errors and no external requests |
| Package build | `uv --native-tls build` | Wheel and source distribution built |
| Distribution metadata | `uv run twine check dist/tramuri-0.13.3-py3-none-any.whl dist/tramuri-0.13.3.tar.gz` | Passed |
| Wheel install | Isolated Python 3.14 environment; installed wheel, imported Tramuri, checked report template resource and ran `tramuri --version` | Passed; version `0.13.3` |
| Package boundary | Inspected wheel and source distribution contents | Report templates included; `tramuri-report-design` and external harness skills absent |
| Product CLI boundary | Tested `experience --help` and the removed `experience skills` command from source and isolated wheel | Product help describes quality checks only; external skill catalog is not exposed |

## Visual artifacts

Generated from a persisted local demo run. The product example is a fixture;
its research remains explicitly `not_run` and does not claim market evidence.

- Report: `tramuri_output/report-design-review/tramuri_output/reports/7b0745a7-8d8d-4bf0-a0e4-3f5a6495dba0/report.html`
- Desktop overview: `tramuri_output/report-design-review/screenshots/report-overview-desktop.png`
- Desktop decision/evidence map: `tramuri_output/report-design-review/screenshots/report-graph-desktop.png`
- Dark theme: `tramuri_output/report-design-review/screenshots/report-dark-desktop.png`
- Mobile overview: `tramuri_output/report-design-review/screenshots/report-mobile.png`
- Mobile Portuguese UI and graph: `tramuri_output/report-design-review/screenshots/report-graph-mobile-pt-BR.png`
- Print export: `tramuri_output/report-design-review/screenshots/report-print-preview.pdf`

The report interface is bilingual; source-authored product text stays in its
recorded language. The report, screenshots and PDF are generated locally and
are not package assets.

## Review notes

- The visual review found and corrected an initial-state navigation marker, a
  generic side-tab accent, and unclear horizontal-map affordance on mobile.
- Key light-theme text pairs measured 4.71:1–13.10:1; dark-theme pairs measured
  6.91:1–13.56:1. Hover links retain the accessible semantic accent color.
- The graph uses only persisted stage dependencies, artifact lineage and exact
  evidence references. Records without explicit links remain disconnected.
- No SQLite schema, workflow state, Markdown/JSON contract or runtime dependency
  changed for this report feature.
- The distributed CLI no longer exposes the harness skill catalog; only the
  harness policy and its report-specific development skill select external
  design/test skills.
- Playwright was supplied temporarily by the development environment for
  browser checks; the Tramuri package does not require it.
- Repository-wide `ruff format --check .` still identifies 10 other dirty Python
  files outside this report change. The 12 report and harness-boundary Python
  files pass the targeted formatter check; unrelated files were left untouched.
