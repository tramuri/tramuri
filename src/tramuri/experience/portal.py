"""Build the central offline Product Experience map."""

from __future__ import annotations

import os
from html import escape
from pathlib import Path
from typing import Any

from tramuri.core.paths import output_root
from tramuri.core.status import build_status
from tramuri.experience.models import DemoReview, ExperienceStage, SensorResult


class ProductPortalBuilder:
    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()

    def build(
        self,
        details: dict[str, Any],
        report_path: Path,
        demo_path: Path,
        review: DemoReview | None = None,
        sensors: list[SensorResult] | None = None,
    ) -> Path:
        root = output_root(self.project_dir) / "portal"
        root.mkdir(parents=True, exist_ok=True)
        report_href = os.path.relpath(report_path, root).replace(os.sep, "/")
        demo_href = os.path.relpath(demo_path, root).replace(os.sep, "/")
        stages = ExperienceStage.default_map()
        status_view = build_status(details)
        event_types = {str(item.get("event_type")) for item in details.get("events", [])}
        report_status = (
            "completed"
            if report_path.is_file()
            and (not event_types or "experience.product_definition" in event_types)
            else "pending"
        )
        if status_view["semantic_status"] in {"blocked", "failed", "stale"}:
            report_status = status_view["semantic_status"]
        stages[0] = ExperienceStage(
            stages[0].key, stages[0].label, report_status, stages[0].summary, report_href
        )
        demo_status = (
            "completed"
            if demo_path.is_file()
            and (not event_types or "experience.demo_generation" in event_types)
            else "pending"
        )
        stages[1] = ExperienceStage(
            stages[1].key, stages[1].label, demo_status, stages[1].summary, demo_href
        )
        title = escape(str(details["run"].get("idea", "Tramuri product")))
        nodes: list[str] = []
        for stage in stages:
            action = (
                f'<a class="stage-link" href="{escape(stage.href)}">Open stage</a>'
                if stage.href
                else f'<button class="stage-link future-button" type="button" data-stage="{escape(stage.key)}">Explore later</button>'
            )
            nodes.append(
                f'<article class="stage stage-{stage.status}" data-stage-card="{stage.key}"><span class="stage-number">{len(nodes) + 1:02d}</span><p class="eyebrow">{escape(stage.status)}</p><h2>{escape(stage.label)}</h2><p>{escape(stage.summary)}</p>{action}</article>'
            )
        review_text = escape((review.status if review else "not-run").replace("_", " "))
        sensor_text = escape(str(sum(item.status == "passed" for item in (sensors or []))))
        workflow_status = escape(str(status_view["semantic_status"]))
        next_action = escape(str(status_view["next_action"]))
        html = f"""<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Tramuri · Product Experience</title><style>
:root{{--ink:#0b2530;--teal:#00b3a6;--mint:#a8efe1;--orange:#ffa229;--paper:#eef8f5;--muted:#557078;font-family:Inter,ui-sans-serif,system-ui,sans-serif}}*{{box-sizing:border-box}}body{{margin:0;background:var(--paper);color:var(--ink);line-height:1.5}}main,header{{max-width:1200px;margin:auto;padding:2rem 1.25rem}}header{{padding-top:4rem}}.eyebrow{{color:var(--teal);letter-spacing:.14em;text-transform:uppercase;font-size:.7rem;font-weight:800}}h1{{font-size:clamp(2.4rem,7vw,6rem);max-width:8ch;line-height:.95;letter-spacing:-.06em;margin:.5rem 0}}.lede{{color:var(--muted);max-width:44rem;font-size:1.1rem}}.map{{display:grid;grid-template-columns:repeat(4,1fr);gap:1rem;position:relative;margin-top:3rem}}.stage{{background:white;border:1px solid #cde5df;border-radius:1rem;padding:1.25rem;min-height:19rem;display:flex;flex-direction:column;box-shadow:0 12px 32px #0b25300d}}.stage::after{{content:"";height:2px;background:var(--mint);position:absolute;margin-top:7rem;width:1rem;transform:translateX(calc(100% + 1rem))}}.stage:last-child::after{{display:none}}.stage-completed{{border-top:5px solid var(--teal)}}.stage-pending{{border-top:5px solid var(--orange)}}.stage-future{{border-top:5px solid #9bb7b4;opacity:.82}}.stage-number{{font-weight:900;font-size:2rem;color:var(--teal)}}.stage h2{{margin:.25rem 0;line-height:1.05}}.stage p:not(.eyebrow){{color:var(--muted);flex:1}}.stage-link{{display:inline-block;border:0;border-radius:999px;background:var(--ink);color:white;padding:.7rem .9rem;font-weight:800;text-decoration:none;text-align:center;font:inherit;cursor:pointer}}.stage-pending .stage-link{{background:var(--orange);color:var(--ink)}}.stage-link:focus-visible,.skip-link:focus-visible{{outline:3px solid var(--orange);outline-offset:3px}}.summary{{display:flex;gap:1rem;flex-wrap:wrap;margin-top:1.5rem}}.summary span{{padding:.6rem .8rem;background:var(--mint);border-radius:.5rem;font-weight:700}}.stage-detail{{margin-top:1rem;color:var(--muted)}}.skip-link{{position:absolute;left:-10000px}}.skip-link:focus-visible{{left:1rem;top:1rem;background:white;padding:.5rem;z-index:2}}@media(max-width:900px){{.map{{grid-template-columns:repeat(2,1fr)}}.stage::after{{display:none}}}}@media(max-width:560px){{.map{{grid-template-columns:1fr}}}}@media(prefers-reduced-motion:reduce){{*{{transition-duration:.01ms!important;scroll-behavior:auto!important}}}}
</style></head><body><a class="skip-link" href="#main">Skip to content</a><header><p class="eyebrow">TRAMURI / PRODUCT EXPERIENCE</p><h1>From intent to verified outcomes.</h1><p class="lede">Um mapa vivo para {title}. Cada etapa revela o que existe agora, o que foi validado e o que ainda é futuro.</p><div class="summary"><span>Workflow: {workflow_status}</span><span>Review: {review_text}</span><span>Sensors passed: {sensor_text}</span></div><p class="stage-detail" role="status" aria-live="polite">Next: {next_action}</p></header><main id="main"><p class="eyebrow">PRODUCT JOURNEY</p><section class="map" aria-label="Product experience stages">{"".join(nodes)}</section><p class="stage-detail" id="stage-detail" role="status" aria-live="polite">Selecione uma etapa futura para ver seu estado.</p></main><script>document.querySelectorAll(".future-button").forEach((button)=>button.addEventListener("click",()=>{{document.querySelector("#stage-detail").textContent="Esta etapa está no mapa de evolução, mas ainda não faz parte desta versão do Tramuri.";}}));</script></body></html>"""
        path = root / "index.html"
        path.write_text(html, encoding="utf-8")
        return path
