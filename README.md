# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Tramuri logo" width="220">
</p>

<p align="center"><strong>From intent to verified outcomes.</strong></p>

<p align="center">
  <a href="https://github.com/tramuri/tramuri/actions/workflows/ci.yml"><img src="https://github.com/tramuri/tramuri/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://pypi.org/project/tramuri/"><img src="https://img.shields.io/pypi/v/tramuri" alt="PyPI version"></a>
</p>

Tramuri is open-source, local-first infrastructure for turning product and business objectives
into coordinated work and verifiable outcomes. It helps people, agents, and tools carry context
between sessions and distinguish completed work from results that have actually been reviewed.

Tramuri is designed for Product Engineers, Forward Deployed Engineers (FDEs), and product teams.
It works without requiring a source-code repository, an LLM, or a particular agent framework.

## What Tramuri provides

- **Missions and Success Contracts** to describe an objective and its acceptance criteria.
- **Persistent knowledge** with source references and history.
- **Execution and evidence records** that keep work, supporting references, and assessment distinct.
- **Verified outcomes** based on explicit criterion reviews; a completed execution alone does not
  mark a Mission as achieved.
- **Project Discovery** as an optional way to inventory a code project. Scanning does not execute
  repository files and excludes sensitive paths by default.
- A native Rust CLI and a Python SDK backed by the same Rust core.

The PyPI package provides the prebuilt Python SDK. Standalone prebuilt installers for the native
CLI are not yet provided.

Tramuri stores local project state in SQLite. Evidence is recorded as references; Tramuri does not
fetch external resources or store scanned file contents as evidence.

## Get started with Python

The Python SDK requires Python 3.11 or newer. Install the prebuilt package:

```bash
python -m pip install tramuri
```

The wheel includes the compiled Rust core. Users installing a compatible wheel do not need to
install Rust.

```python
from pathlib import Path

import tramuri

workspace = Path("customer-onboarding")
workspace.mkdir(exist_ok=True)
tramuri.init(workspace)

mission = tramuri.mission_create(
    workspace,
    objective="Validate the customer onboarding flow",
    criteria=[
        {
            "description": "The customer can complete onboarding",
            "required": True,
            "evidence_expectation": "Customer acceptance or observed completion",
        }
    ],
)

print(mission["id"])
```

Mission creation does not require a Git repository or a project scan. See the
[product vision](VISION.md) for Tramuri's scope and principles, and the
[changelog](CHANGELOG.md) for release history.

## Project status

Tramuri is in active early development. Project Discovery, persistent knowledge, Missions, manual
Execution and Evidence records, and criterion assessments are part of the current product. Versioned
Context Pack and Evidence Pack export is planned; it will use explicit selection and authorization
and will be safe by default.

## Community

- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security policy](SECURITY.md)
- [Support](SUPPORT.md)
- [License](LICENSE) (Apache-2.0)
