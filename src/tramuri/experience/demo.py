"""Generate a small, offline and traceable product demo."""

from __future__ import annotations

import json
from html import escape
from pathlib import Path
from typing import Any

from tramuri.core.paths import output_root
from tramuri.experience.models import DemoResult


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


class ProductDemoBuilder:
    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()

    def build(self, details: dict[str, Any]) -> DemoResult:
        root = output_root(self.project_dir) / "demo"
        root.mkdir(parents=True, exist_ok=True)
        run_id = str(details["run"]["id"])
        by_kind = {
            item.get("kind"): item.get("payload", {}) for item in details.get("artifacts", [])
        }
        framing = by_kind.get("pm.framing", {})
        shaping = by_kind.get("pm.shaping", {})
        title = str(framing.get("product_name") or details["run"].get("idea", "Product demo"))
        product_name = escape(title)
        scope = [str(item) for item in shaping.get("in_scope", [])]
        scope_items = "".join(f"<li>{escape(item)}</li>" for item in scope)
        source_artifacts = tuple(str(item.get("kind")) for item in details.get("artifacts", []))
        html = f"""<!doctype html>
<html lang="pt-BR">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>{product_name} · Product Demo</title><link rel="stylesheet" href="styles.css"></head>
<body><a class="skip-link" href="#main">Skip to content</a>
<header class="site-header"><p class="eyebrow">TRAMURI / PRODUCT DEMO</p><h1>{product_name}</h1>
<p class="lede">Uma experiência offline para explorar o caminho principal definido pelo produto.</p></header>
<main id="main" class="demo-shell">
<nav class="journey" aria-label="Demo journey"><a href="#home">Start</a><a href="#availability">Availability</a><a href="#confirmation">Confirmation</a></nav>
<section id="home" class="screen screen-hero" aria-labelledby="home-title"><div><p class="eyebrow">CORE JOURNEY</p><h2 id="home-title">Comece pelo resultado que importa.</h2><p>Escolha um serviço, veja uma disponibilidade e confirme uma reserva fictícia.</p><button type="button" data-action="choose-service">Choose service</button></div><aside class="product-note"><p class="eyebrow">In scope</p><ul>{scope_items or "<li>Core outcome flow</li>"}</ul></aside></section>
<section id="service" class="screen" aria-labelledby="service-title"><p class="eyebrow">01 / SERVICE</p><h2 id="service-title">Qual serviço você precisa?</h2><label for="service-choice">Service</label><select id="service-choice"><option>Consultation</option><option>Onboarding session</option><option>Maintenance visit</option></select><a class="text-link" href="#availability">Go to availability</a></section>
<section id="availability" class="screen" aria-labelledby="availability-title"><p class="eyebrow">02 / AVAILABILITY</p><h2 id="availability-title">Escolha um horário</h2><div class="choice-grid"><button type="button" data-action="choose-time">Today · 14:00</button><button type="button" data-action="choose-time">Tomorrow · 09:30</button></div></section>
<section id="confirmation" class="screen" aria-labelledby="confirmation-title"><p class="eyebrow">03 / CONFIRM</p><h2 id="confirmation-title">Pronto para confirmar?</h2><p>Esta é uma simulação local. Nenhuma reserva real será criada.</p><button type="button" data-action="confirm">Confirm demo booking</button></section>
<section id="complete" class="screen result" aria-labelledby="complete-title" hidden><p class="eyebrow">COMPLETE</p><h2 id="complete-title">Caminho concluído.</h2><p id="result-copy">Sua reserva de demonstração foi registrada localmente.</p></section>
<p id="demo-status" class="status" role="status" aria-live="polite">Choose a service to begin</p>
</main><script src="app.js"></script></body></html>"""
        css = """
:root{color-scheme:light;--ink:#0b2530;--muted:#557078;--teal:#00b3a6;--mint:#a8efe1;--orange:#ffa229;--paper:#f3faf8;--line:#cde5df;font-family:Inter,ui-sans-serif,system-ui,sans-serif}*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;background:var(--paper);color:var(--ink);line-height:1.55}.site-header,.demo-shell{max-width:1100px;margin:auto;padding:2rem 1.25rem}.site-header{padding-top:4rem}.eyebrow{font-size:.72rem;letter-spacing:.16em;font-weight:800;color:var(--teal)}h1,h2{line-height:1.05;letter-spacing:-.04em}h1{font-size:clamp(2.4rem,8vw,6rem);max-width:9ch;margin:.5rem 0}.lede{max-width:38rem;color:var(--muted);font-size:1.1rem}.journey{display:flex;gap:1rem;flex-wrap:wrap;border-block:1px solid var(--line);padding:1rem 0}.journey a,.text-link{color:var(--ink);font-weight:700}.screen{background:white;border:1px solid var(--line);border-radius:1.25rem;padding:clamp(1.25rem,4vw,3rem);margin:1.25rem 0;box-shadow:0 12px 35px #0b25300d}.screen-hero{display:grid;grid-template-columns:1.5fr 1fr;gap:2rem;background:var(--ink);color:white}.screen-hero .eyebrow{color:var(--mint)}.screen-hero p{color:#d6eeea}.product-note{border-left:2px solid var(--orange);padding-left:1rem}.product-note li{margin:.5rem 0}.choice-grid{display:flex;gap:.75rem;flex-wrap:wrap}button{border:0;border-radius:999px;background:var(--teal);color:var(--ink);font-weight:800;padding:.8rem 1.1rem;cursor:pointer}button:hover{background:var(--mint)}button:focus-visible,a:focus-visible,select:focus-visible{outline:3px solid var(--orange);outline-offset:3px}select{display:block;margin:.5rem 0 1rem;padding:.7rem;border:1px solid var(--line);border-radius:.5rem;font:inherit}.status{position:sticky;bottom:1rem;background:var(--orange);padding:.7rem 1rem;border-radius:.6rem;font-weight:800}.skip-link{position:absolute;left:-10000px}.skip-link:focus{left:1rem;top:1rem;background:white;padding:.5rem;z-index:2}.result{border-color:var(--teal)}@media(max-width:700px){.screen-hero{grid-template-columns:1fr}.site-header{padding-top:2rem}}@media(prefers-reduced-motion:reduce){*{animation-duration:.01ms!important;transition-duration:.01ms!important;scroll-behavior:auto!important}}
"""
        js = """
const status = document.querySelector("#demo-status");
const service = document.querySelector("#service");
function move(target, message) { target.hidden = false; status.textContent = message; target.scrollIntoView({behavior: "smooth", block: "start"}); }
document.querySelector('[data-action="choose-service"]').addEventListener("click", () => move(service, "Service selected. Choose a time."));
document.querySelectorAll('[data-action="choose-time"]').forEach((button) => button.addEventListener("click", () => move(document.querySelector("#confirmation"), "Time selected. Confirm your demo booking.")));
document.querySelector('[data-action="confirm"]').addEventListener("click", () => { document.querySelector("#complete").hidden = false; status.textContent = "Demo booking confirmed"; document.querySelector("#complete").scrollIntoView({behavior: "smooth"}); });
"""
        _write(root / "index.html", html)
        _write(root / "styles.css", css)
        _write(root / "app.js", js)
        screens = ("home", "service", "availability", "confirmation", "complete")
        traceability = {
            "run_id": run_id,
            "synthetic": True,
            "product_name": title,
            "source_artifacts": list(source_artifacts),
            "screens": list(screens),
            "interactions": ["choose-service", "choose-time", "confirm"],
        }
        _write(
            root / "traceability.json",
            json.dumps(traceability, ensure_ascii=False, indent=2) + "\n",
        )
        return DemoResult(
            root,
            root / "index.html",
            tuple(root.iterdir()),
            screens,
            source_artifacts,
        )
