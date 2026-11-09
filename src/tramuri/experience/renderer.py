"""Self-contained HTML renderers for product-specific offline simulations."""

from __future__ import annotations

from collections.abc import Mapping
from html import escape
from typing import Any

from tramuri.experience.registry import DemoTemplate, DemoTemplateRegistry

DEMO_CSS = """
:root{color-scheme:light;--bg:#f4f6f5;--surface:#fff;--ink:#1c2927;--muted:#576965;--accent:#376c62;--soft:#dcebe6;--line:#d9e2df;--signal:#d9e6a7;--radius:18px;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;font-synthesis:none;text-rendering:optimizeLegibility}*{box-sizing:border-box}html{scroll-behavior:smooth;scroll-padding-top:1.5rem}body{margin:0;background:var(--bg);color:var(--ink);line-height:1.55}button,input,select,textarea{font:inherit}button,a,input,select,textarea{touch-action:manipulation}button,a,select,input,textarea{min-height:44px}a{color:inherit;text-underline-offset:.2em}button{border:1px solid transparent;border-radius:999px;background:var(--accent);color:#fff;padding:.68rem 1.05rem;font-weight:700;cursor:pointer;transition:background-color .16s ease,transform .16s ease}button:hover{background:var(--ink)}button:active{transform:translateY(1px)}button:disabled{cursor:not-allowed;opacity:.55}button:focus-visible,a:focus-visible,input:focus-visible,select:focus-visible,textarea:focus-visible{outline:3px solid var(--signal);outline-offset:3px}.skip-link{position:absolute;top:.5rem;left:-10000px;background:var(--surface);padding:.7rem 1rem;z-index:9}.skip-link:focus{left:.5rem}.page-head,.demo-main{width:min(1120px,100% - 2.5rem);margin:0 auto}.page-head{padding:2.6rem 0 1.2rem}.page-head h1{font-size:clamp(2rem,5vw,4.2rem);line-height:1.02;letter-spacing:-.035em;text-wrap:balance;max-width:16ch;margin:.4rem 0 1rem}.page-head p{max-width:68ch;color:var(--muted);margin:.4rem 0}.eyebrow{font-size:.78rem;font-weight:750;letter-spacing:.08em;color:var(--accent)}.simulation-note{display:inline-flex;align-items:center;gap:.5rem;border:1px solid var(--line);border-radius:999px;background:var(--surface);padding:.45rem .75rem;font-size:.84rem;color:var(--muted)}.simulation-dot{width:.5rem;height:.5rem;border-radius:50%;background:var(--accent)}.demo-main{padding:1rem 0 4rem}.surface-nav{display:flex;flex-wrap:wrap;gap:.5rem;border-bottom:1px solid var(--line);padding:.5rem 0 1rem;margin-bottom:1.2rem}.surface-nav a{min-height:44px;display:inline-flex;align-items:center;padding:0 .75rem;border-radius:999px;font-weight:650}.surface-nav a:hover{background:var(--soft)}.screen{background:var(--surface);border:1px solid var(--line);border-radius:var(--radius);padding:clamp(1rem,3vw,2rem);margin:1rem 0}.screen h2,.screen h3{line-height:1.15;text-wrap:balance}.lede{font-size:1.05rem;color:var(--muted);max-width:65ch}.split{display:grid;grid-template-columns:minmax(0,1.2fr) minmax(240px,.8fr);gap:1rem;align-items:start}.eyeline{height:1px;background:var(--line);margin:1.2rem 0}.muted{color:var(--muted)}.tag{display:inline-flex;align-items:center;border-radius:999px;background:var(--soft);color:var(--ink);padding:.35rem .65rem;font-size:.84rem;font-weight:700}.state{color:var(--muted);min-height:1.6em;margin:.7rem 0 0}.state[aria-live]{font-weight:600}.list-clean{list-style:none;padding:0;margin:0}.list-clean li{padding:.8rem 0;border-bottom:1px solid var(--line)}.field{display:grid;gap:.35rem;margin:.7rem 0}.field label{font-weight:700}.field input,.field select,.field textarea{width:100%;border:1px solid var(--line);border-radius:10px;background:var(--surface);color:var(--ink);padding:.7rem .8rem;font-size:1rem}.field textarea{min-height:8rem;resize:vertical;font-family:ui-monospace,SFMono-Regular,Consolas,monospace}.action-row{display:flex;flex-wrap:wrap;gap:.6rem;align-items:center}.quiet-button{background:var(--soft);color:var(--ink);border-color:var(--line)}.quiet-button:hover{background:var(--line)}.numeric{font-variant-numeric:tabular-nums}.project-board{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:.8rem}.board-lane{background:color-mix(in srgb,var(--soft) 52%,var(--surface));border-radius:14px;padding:.8rem;min-width:0}.task-row{padding:.8rem;background:var(--surface);border:1px solid var(--line);border-radius:10px;margin:.65rem 0;overflow-wrap:anywhere}.device-layout{display:grid;grid-template-columns:minmax(260px,390px) minmax(220px,1fr);gap:clamp(1rem,5vw,4rem);align-items:center;justify-content:center}.device-shell{width:min(100%,390px);margin:auto;background:#111714;border:7px solid #111714;border-radius:42px;padding:8px;box-shadow:0 24px 58px #15241b24}.device-screen{position:relative;min-height:690px;overflow:hidden;border-radius:32px;background:var(--bg);padding:env(safe-area-inset-top,12px) 16px calc(24px + env(safe-area-inset-bottom,0px))}.device-island{height:26px;width:86px;background:#111714;border-radius:999px;margin:0 auto 14px}.device-topbar{display:flex;justify-content:space-between;align-items:center;font-size:.82rem;font-weight:750}.device-tabs{position:sticky;bottom:0;display:flex;justify-content:space-around;background:var(--surface);border-top:1px solid var(--line);margin:1rem -16px -1rem;padding:.4rem 0 calc(.4rem + env(safe-area-inset-bottom,0px))}.device-tabs a{display:grid;place-items:center;text-decoration:none;font-size:.78rem;font-weight:700;padding:0 .4rem}.money-total{font-size:clamp(2rem,8vw,3rem);letter-spacing:-.045em;font-weight:780}.terminal-window{overflow:hidden;background:#101619;color:#e9f0ee;border:1px solid #384944;border-radius:16px;box-shadow:0 16px 34px #1016191f}.terminal-bar{display:flex;align-items:center;gap:.45rem;padding:.8rem 1rem;background:#1b2629;border-bottom:1px solid #384944}.terminal-light{width:.65rem;height:.65rem;border-radius:50%;background:#7da99c}.terminal-title{margin-left:.5rem;color:#b7c7c0;font-size:.86rem}.terminal-output{min-height:230px;max-height:390px;overflow:auto;padding:1.2rem;font: .94rem/1.7 ui-monospace,SFMono-Regular,Consolas,monospace;white-space:pre-wrap;overflow-wrap:anywhere}.terminal-output .prompt{color:#a5e071}.terminal-form{padding:1rem;border-top:1px solid #384944}.terminal-form label{display:block;color:#c1d0ca;margin-bottom:.4rem}.terminal-form .action-row{align-items:stretch}.terminal-form input{flex:1 1 14rem;min-width:0;border:1px solid #50645c;border-radius:9px;background:#182226;color:#fff;padding:.7rem;font:1rem ui-monospace,SFMono-Regular,Consolas,monospace}.api-layout{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:1rem}.endpoint{display:flex;gap:.65rem;align-items:center;font:700 .92rem ui-monospace,SFMono-Regular,Consolas,monospace}.method{color:#fff;background:var(--accent);border-radius:6px;padding:.32rem .5rem}.pipeline{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:1.6rem;position:relative}.pipeline-node{position:relative;border:1px solid var(--line);border-radius:14px;background:var(--surface);padding:1rem;min-height:116px}.pipeline-node:not(:last-child)::after{content:"";position:absolute;left:calc(100% + .1rem);top:50%;width:1.4rem;height:2px;background:var(--accent)}.pipeline-node[data-state="running"]{border-color:var(--accent);background:var(--soft)}.pipeline-node[data-state="complete"]{border-color:var(--accent)}.device-panel{display:grid;grid-template-columns:minmax(0,1.1fr) minmax(220px,.9fr);gap:1rem}.reading-value{font-size:clamp(2.4rem,7vw,4rem);font-weight:780;letter-spacing:-.045em}.window-frame{background:var(--surface);border:1px solid var(--line);border-radius:14px;overflow:hidden;box-shadow:0 15px 36px #172d3a14}.window-titlebar{display:flex;align-items:center;gap:.5rem;padding:.75rem 1rem;border-bottom:1px solid var(--line);background:var(--soft)}.window-content{padding:1.25rem}.hybrid-surfaces{display:grid;grid-template-columns:minmax(0,.85fr) minmax(0,1.15fr);gap:1rem;align-items:start}.compact-device{border:5px solid #15201d;border-radius:26px;background:var(--bg);padding:.7rem;max-width:330px}.compact-device .screen{padding:1rem}.concept-flow{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem;align-items:stretch}.concept-step{border-top:3px solid var(--accent);padding:1rem .2rem}.table-wrap{overflow-x:auto}table{width:100%;border-collapse:collapse}th,td{text-align:left;padding:.7rem;border-bottom:1px solid var(--line)}th{color:var(--muted);font-size:.86rem}footer{color:var(--muted);font-size:.86rem;padding:1rem 0}.scroll-area{max-width:100%;overflow-wrap:anywhere}::selection{background:var(--soft);color:var(--ink)}@media(max-width:760px){.page-head,.demo-main{width:min(100% - 1.25rem,1120px)}.page-head{padding-top:1.6rem}.split,.device-layout,.api-layout,.device-panel,.hybrid-surfaces{grid-template-columns:1fr}.project-board{grid-template-columns:1fr}.pipeline{grid-template-columns:1fr;gap:.8rem}.pipeline-node:not(:last-child)::after{left:1.25rem;top:100%;width:2px;height:.8rem}.device-screen{min-height:650px}.surface-nav{gap:.2rem}.surface-nav a{padding:0 .55rem}}@media(prefers-reduced-motion:reduce){*,*::before,*::after{scroll-behavior:auto!important;animation-duration:.01ms!important;transition-duration:.01ms!important}}
"""

MARKETPLACE_CSS = """
.marketplace-intro{display:grid;grid-template-columns:minmax(0,1fr) minmax(220px,.62fr);gap:1rem;align-items:end;border-bottom:1px solid var(--line);padding:1rem 0 2rem}.marketplace-intro h2{font-size:clamp(2rem,5vw,3.5rem);max-width:14ch;line-height:1.04;letter-spacing:-.035em}.catalog-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:1rem}.listing-card{background:var(--surface);border:1px solid var(--line);border-radius:14px;overflow:hidden}.listing-art{min-height:150px;display:grid;place-items:center;background:var(--soft);font-size:2.6rem}.listing-copy{padding:1rem}.listing-copy h3{margin:.2rem 0}.price{font-weight:750;font-variant-numeric:tabular-nums}.basket-summary{display:flex;justify-content:space-between;align-items:center;gap:1rem;border:1px solid var(--line);border-radius:12px;padding:1rem;background:var(--surface)}.seller-workspace{display:grid;grid-template-columns:minmax(0,1fr) minmax(250px,.6fr);gap:1rem;align-items:center}@media(max-width:760px){.marketplace-intro,.seller-workspace{grid-template-columns:1fr}.catalog-grid{grid-template-columns:1fr}.listing-art{min-height:120px}}
"""

MOBILE_PLATFORM_CSS = """
.android-device{border-radius:30px;padding:7px}.android-device .device-screen{border-radius:22px}.android-device .device-island{display:none}
.device-tabs button{display:grid;place-items:center;min-width:88px;min-height:44px;background:transparent;color:var(--muted);border:0;border-radius:8px;padding:.4rem;font-size:.78rem;font-weight:700}.device-tabs button[aria-selected="true"]{color:var(--accent);box-shadow:inset 0 2px 0 var(--accent)}
"""


class DemoRenderer:
    """Render a stored demo specification through its registered template."""

    def __init__(self, registry: DemoTemplateRegistry) -> None:
        self.registry = registry

    def render(self, specification: Mapping[str, Any]) -> dict[str, str]:
        template = self.registry.by_id(str(specification["template_id"]))
        rendered = template.render(specification)
        rendered.setdefault("styles.css", self.stylesheet(specification))
        rendered.setdefault(
            "app.js", _script(template.template_id, str(specification.get("domain", "")))
        )
        return rendered

    @staticmethod
    def stylesheet(specification: Mapping[str, Any]) -> str:
        colors = specification.get("visual_direction", {}).get("colors", {})
        palette = {
            "bg": colors.get("background", "#f4f6f5"),
            "surface": colors.get("surface", "#ffffff"),
            "ink": colors.get("ink", "#1c2927"),
            "muted": colors.get("muted", "#576965"),
            "accent": colors.get("accent", "#376c62"),
            "soft": colors.get("accent_soft", "#dcebe6"),
            "line": colors.get("line", "#d9e2df"),
            "signal": colors.get("highlight", "#d9e6a7"),
        }
        variables = (
            ":root{" + "".join(f"--{key}:{_e(value)};" for key, value in palette.items()) + "}"
        )
        template_id = _e(str(specification.get("template_id", "general")))
        button_color = ".terminal-window button{color:#101619}" if template_id == "cli" else ""
        return (
            DEMO_CSS
            + MOBILE_PLATFORM_CSS
            + MARKETPLACE_CSS
            + "\n"
            + variables
            + f'\nbody[data-template="{template_id}"]{{color:var(--ink)}}\n'
            + button_color
            + "\n"
        )


def default_template_registry() -> DemoTemplateRegistry:
    registry = DemoTemplateRegistry()
    definitions = (
        (
            "mobile",
            ("mobile_app",),
            _mobile,
            (
                ("home", "Overview", "Review the primary mobile outcome"),
                ("activity", "Activity", "Inspect local simulated changes"),
            ),
            (("add-expense", "Add simulated expense", "Update local mobile state"),),
        ),
        (
            "cli",
            ("cli_developer_tool",),
            _cli,
            (
                ("terminal", "Terminal", "Choose a safe predefined command scenario"),
                ("terminal-output", "Command result", "Review simulated terminal output"),
            ),
            (("run-scenario", "Run scenario", "Run only an allowlisted browser scenario"),),
        ),
        (
            "web",
            ("web_saas",),
            _web,
            (
                ("workspace", "Workspace", "Work with the product's domain objects"),
                ("board", "Main workflow", "Move through the primary work state"),
                ("complete", "Result", "Review the locally updated result"),
            ),
            (("move-task", "Move task to In progress", "Update the local project board"),),
        ),
        (
            "desktop",
            ("desktop_app",),
            _desktop,
            (
                ("workspace", "Workspace", "Work in a desktop application window"),
                ("result", "Result", "Review the saved local state"),
            ),
            (("save-settings", "Save settings", "Update the visible local application state"),),
        ),
        (
            "api",
            ("api_backend",),
            _api,
            (
                ("request", "Request", "Edit a local example request"),
                ("response", "Response", "Inspect a simulated response"),
            ),
            (
                (
                    "send-request",
                    "Send simulated request",
                    "Validate local JSON and render a synthetic response",
                ),
            ),
        ),
        (
            "pipeline",
            ("data_ai_pipeline",),
            _pipeline,
            (
                ("source", "Source", "Inspect source data and pipeline steps"),
                ("execution", "Execution", "Run a local pipeline simulation"),
                ("output", "Output", "Review a synthetic processing result"),
            ),
            (
                (
                    "run-pipeline",
                    "Run pipeline",
                    "Advance predefined stages and show synthetic output",
                ),
            ),
        ),
        (
            "iot",
            ("iot_hardware",),
            _iot,
            (
                ("device", "Device", "Adjust a simulated device reading"),
                ("telemetry", "Telemetry", "Review a local telemetry state"),
            ),
            (
                (
                    "change-reading",
                    "Change simulated reading",
                    "Update local telemetry and alert state",
                ),
            ),
        ),
        (
            "hybrid",
            ("hybrid",),
            _hybrid,
            (
                ("field-app", "Field app", "Capture work in the mobile surface"),
                ("operations", "Operations dashboard", "Review that work in the web surface"),
                ("complete", "Updated state", "Confirm both product surfaces changed"),
            ),
            (
                (
                    "capture-reading",
                    "Record field reading",
                    "Update the mobile record and operations status",
                ),
                ("acknowledge-alert", "Acknowledge alert", "Update the shared local alert state"),
            ),
        ),
        (
            "other",
            ("other",),
            _other,
            (
                ("concept", "Product concept", "Explore the inferred product outcome"),
                ("flow", "Primary flow", "Move through the inferred user journey"),
                ("result", "Outcome", "Review the local simulated result"),
            ),
            (("advance-flow", "Show next step", "Move to the next inferred product state"),),
        ),
    )
    for template_id, categories, render, screens, interactions in definitions:
        registry.register(DemoTemplate(template_id, categories, render, screens, interactions))
    return registry


def _document(spec: Mapping[str, Any], body: str, *, page_title: str | None = None) -> str:
    title = _e(page_title or str(spec.get("product_name", "Product concept")))
    name = _e(str(spec.get("product_name", "Product concept")))
    description = _e(str(spec.get("description", "A locally generated product simulation.")))
    category = _e(str(spec.get("category", "other")))
    template_id = _e(str(spec.get("template_id", "other")))
    visual = spec.get("visual_direction", {})
    direction = ""
    if visual.get("provenance") == "design_proposal":
        label = str(visual.get("label", "proposed product direction")).removeprefix("proposed ")
        direction = f'<span class="tag">Proposed design direction · {_e(label)}</span>'
    return f'''<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="theme-color" content="{_e(spec.get("visual_direction", {}).get("colors", {}).get("background", "#f4f6f5"))}"><title>{title} · Product simulation</title><link rel="stylesheet" href="styles.css"></head><body data-template="{template_id}" data-demo-category="{category}" data-simulation-level="interactive_simulation"><a class="skip-link" href="#main">Skip to main content</a><header class="page-head"><span class="simulation-note"><span class="simulation-dot" aria-hidden="true"></span>Interactive simulation · illustrative local data</span>{direction}<h1>{name}</h1><p>{description}</p></header><main id="main" class="demo-main">{body}</main><footer class="page-head"><p>This prototype is a local simulation. No live integrations, accounts, transactions, or user validation are represented.</p></footer><script src="app.js"></script></body></html>'''


def _features(spec: Mapping[str, Any], limit: int = 5) -> list[str]:
    values = spec.get("features", [])
    return (
        [str(value) for value in values[:limit] if str(value).strip()]
        if isinstance(values, list)
        else []
    )


def _feature_list(spec: Mapping[str, Any], limit: int = 5) -> str:
    values = _features(spec, limit) or ["Primary product outcome", "Local result state"]
    return "".join(f"<li>{_e(value)}</li>" for value in values)


def _nav(*items: tuple[str, str]) -> str:
    return (
        '<nav class="surface-nav" aria-label="Product surfaces">'
        + "".join(f'<a href="{_e(href)}">{_e(label)}</a>' for href, label in items)
        + "</nav>"
    )


def _mobile(spec: Mapping[str, Any]) -> dict[str, str]:
    platform = str(spec.get("platform", ""))
    is_android = platform.casefold() == "android"
    device_class = "device-shell android-device" if is_android else "device-shell"
    device_label = (
        "Android smartphone product simulation" if is_android else "Smartphone product simulation"
    )
    system_bar = (
        '<div class="device-topbar"><span>9:41</span><span aria-hidden="true">▮ ▮ ▰</span></div>'
        if is_android
        else '<div class="device-topbar"><span>9:41</span><span aria-hidden="true">● ● ●</span></div>'
    )
    island = "" if is_android else '<div class="device-island" aria-hidden="true"></div>'
    platform_note = (
        "This Android proposal uses Android device chrome and a phone-first layout. "
        "Product behavior remains simulated."
        if is_android
        else "The device frame and touch layout reflect the supplied mobile platform context. "
        "Product behavior remains simulated."
    )
    if spec.get("domain") == "personal_finance":
        screen = """<section id="home" class="screen" role="tabpanel" aria-labelledby="tab-overview" data-mobile-screen="overview"><p class="eyebrow">MONTHLY SPENDING</p><h2>Room in your budget</h2><p class="money-total numeric" id="spending-total">$1,240</p><p class="muted">$760 remaining · illustrative budget</p><div class="eyeline"></div><div class="action-row"><span class="tag">Everyday · 42%</span><span class="tag">Travel · 18%</span></div><p class="muted">Your full transaction history is on the Activity screen.</p><button type="button" data-action="add-expense">Add $8 coffee expense</button><p id="demo-status" class="state" role="status" aria-live="polite">Your sample budget is ready.</p></section>"""
        activity_screen = """<section id="activity" class="screen" role="tabpanel" aria-labelledby="tab-activity" data-mobile-screen="activity" hidden><p class="eyebrow">ACTIVITY</p><h2>Recent transactions</h2><p class="muted">Illustrative entries from this local budget.</p><ul id="activity-list" class="list-clean"><li>Market groceries <span class="numeric">−$48</span></li><li>Local transit <span class="numeric">−$12</span></li></ul></section>"""
        side = """<section class="screen"><p class="eyebrow">PROPOSED MOBILE EXPERIENCE</p><h2>Keep the next action close</h2><p class="lede">A thumb-reachable expense action updates local sample values. The amounts are illustrative and no bank connection is present.</p><ul class="list-clean"><li>Designed for a phone viewport with safe-area spacing</li><li>Cross-platform layout adapts to the supplied iOS or Android context</li><li>Large touch controls and a readable spending summary</li></ul></section>"""
    else:
        journey = _e(str(spec.get("primary_journey", "Continue your work")))
        screen = f"""<section id="home" class="screen" role="tabpanel" aria-labelledby="tab-overview" data-mobile-screen="overview"><p class="eyebrow">YOUR NEXT STEP</p><h2>{journey}</h2><p class="lede">A phone-sized view of the product's primary outcome.</p><ul class="list-clean">{_feature_list(spec, 3)}</ul><button type="button" data-action="complete-mobile-step" data-activity-label="{journey}">Complete the next step</button><p id="demo-status" class="state" role="status" aria-live="polite">Ready for a local interaction.</p></section>"""
        activity_screen = """<section id="activity" class="screen" role="tabpanel" aria-labelledby="tab-activity" data-mobile-screen="activity" hidden><p class="eyebrow">RECENT UPDATES</p><h2>Your activity</h2><p class="muted">This list records actions from the local product journey.</p><ul id="journey-activity" class="list-clean"><li>Ready · awaiting your first action</li></ul></section>"""
        side = f"""<section class="screen"><p class="eyebrow">MOBILE SURFACE</p><h2>Designed around the supplied journey</h2><p class="lede">{_e(platform_note)}</p></section>"""
    tabs = """<nav class="device-tabs" role="tablist" aria-label="App navigation"><button id="tab-overview" type="button" role="tab" aria-controls="home" aria-selected="true" data-mobile-view="overview">Overview</button><button id="tab-activity" type="button" role="tab" aria-controls="activity" aria-selected="false" tabindex="-1" data-mobile-view="activity">Activity</button></nav>"""
    body = f"""{_nav(("#mobile-app", "App preview"), ("#simulation-state", "Simulation notes"))}<div class="device-layout"><div id="mobile-app" class="{device_class}" aria-label="{device_label}"><div class="device-screen">{island}{system_bar}{screen}{activity_screen}{tabs}</div></div><div>{side}<section id="simulation-state" class="screen"><p class="eyebrow">SIMULATION STATE</p><h2>What this prototype changes</h2><p class="muted">Controls update only this local prototype. They do not create a real transaction, account or external record.</p></section></div></div>"""
    index = _document(spec, body)
    return {"index.html": index, "marketing.html": index}


def _cli(spec: Mapping[str, Any]) -> dict[str, str]:
    title = _e(str(spec.get("product_name", "Developer tool")))
    body = f"""{_nav(("#terminal", "Terminal"), ("#scenarios", "Scenarios"))}<section id="terminal" class="screen"><div class="split"><div><p class="eyebrow">COMMAND LINE EXPERIENCE</p><h2>Try a predefined workflow</h2><p class="lede">This terminal renders fixed responses in the browser. It never starts a shell or runs system commands.</p></div><div><span class="tag">No operating-system access</span></div></div><div class="terminal-window" aria-label="Interactive simulated terminal"><div class="terminal-bar"><span class="terminal-light" aria-hidden="true"></span><span class="terminal-light" aria-hidden="true"></span><span class="terminal-light" aria-hidden="true"></span><span class="terminal-title" translate="no">{title} · local demo</span></div><pre class="terminal-output" id="terminal-output" role="log" aria-live="polite"><span class="prompt">$</span> tramuri status\nRun a predefined scenario or enter a supported demo command.</pre><form id="command-form" class="terminal-form"><label for="demo-command">Try a simulated command</label><div class="action-row"><input id="demo-command" name="command" autocomplete="off" spellcheck="false" value="tramuri status"><button type="submit" data-action="run-scenario">Run scenario</button></div></form></div><p id="demo-status" class="state" role="status" aria-live="polite">Choose a command scenario below.</p></section><section id="scenarios" class="screen"><h2>Available scenarios</h2><div class="action-row"><button type="button" class="quiet-button" data-scenario="tramuri --help">Show help</button><button type="button" class="quiet-button" data-scenario="tramuri init --starter api">Initialize a project</button><button type="button" class="quiet-button" data-scenario="tramuri status">Inspect status</button></div><p class="muted">Only these bundled examples have responses. Other input receives a safe “not available” message.</p><ul class="list-clean">{_feature_list(spec, 4)}</ul></section>"""
    return {"index.html": _document(spec, body)}


def _web(spec: Mapping[str, Any]) -> dict[str, str]:
    domain = str(spec.get("domain", "general"))
    if domain == "service_booking":
        return _web_appointment(spec)
    if domain == "marketplace":
        return _web_marketplace(spec)
    if domain == "education":
        return _web_learning(spec)
    return _web_workspace(spec)


def _web_marketplace(spec: Mapping[str, Any]) -> dict[str, str]:
    body = f"""{_nav(("#home", "Storefront"), ("#catalog", "Catalog"), ("#cart", "Basket"), ("#seller", "Seller workspace"))}<section id="home" class="marketplace-intro"><div><p class="eyebrow">INDEPENDENT MAKERS · LOCAL DEMO</p><h2>Useful objects, thoughtfully made.</h2><p class="lede">A marketplace journey connects discovery, product detail and a sample order.</p><button type="button" data-action="browse-catalog">Shop by collection</button></div><aside><span class="tag">Sample listings · synthetic</span><p class="muted">Items and maker names are illustrative. This demo does not represent a live inventory.</p></aside></section><section id="catalog" class="screen"><div class="split"><div><p class="eyebrow">THE MARKET</p><h2>Shop by collection</h2><p class="lede">Explore a small example catalog from independent makers.</p></div><p class="tag">Local sample catalog</p></div><div class="catalog-grid"><article class="listing-card"><div class="listing-art" aria-hidden="true">◒</div><div class="listing-copy"><p class="eyebrow">HOME · SAMPLE MAKER</p><h3>Stoneware table vase</h3><p class="muted">Hand-finished ceramic · illustrative listing</p><p class="price">$38.00 · sample price</p><button type="button" class="quiet-button" data-action="view-product">View details</button></div></article><article class="listing-card"><div class="listing-art" aria-hidden="true">▧</div><div class="listing-copy"><p class="eyebrow">PRINT · SAMPLE MAKER</p><h3>Garden study print</h3><p class="muted">Small-batch linocut · illustrative listing</p><p class="price">$24.00 · sample price</p><button type="button" class="quiet-button" data-action="view-product">View details</button></div></article><article class="listing-card"><div class="listing-art" aria-hidden="true">⌑</div><div class="listing-copy"><p class="eyebrow">TEXTILE · SAMPLE MAKER</p><h3>Everyday linen wrap</h3><p class="muted">Natural fiber · illustrative listing</p><p class="price">$31.00 · sample price</p><button type="button" class="quiet-button" data-action="view-product">View details</button></div></article></div></section><section id="product" class="screen"><p class="eyebrow">PRODUCT DETAIL</p><h2>Stoneware table vase</h2><p class="lede">A clear detail view helps a shopper understand the item before deciding.</p><ul class="list-clean"><li>Material: sample stoneware</li><li>Maker: illustrative profile</li><li>Availability: not connected to inventory</li></ul><button type="button" data-action="add-to-cart">Add to basket</button><p id="demo-status" class="state" role="status" aria-live="polite">Choose a sample listing to continue.</p></section><section id="cart" class="screen"><div class="split"><div><p class="eyebrow">YOUR BASKET</p><h2>Review your selection</h2><p id="cart-line" class="muted">No sample items yet.</p></div><div class="basket-summary"><span>Items in basket</span><strong id="cart-count" class="numeric">0</strong></div></div><button type="button" data-action="checkout">Complete sample checkout</button><p class="muted">No payment or order is created.</p></section><section id="complete" class="screen" hidden><p class="eyebrow">SIMULATED ORDER</p><h2>Sample checkout complete</h2><p>Your local basket state is now complete. No seller or payment service was contacted.</p></section><section id="seller" class="screen seller-workspace"><div><p class="eyebrow">SELLER WORKSPACE</p><h2>One storefront, two useful perspectives.</h2><p class="lede">A separate seller view can manage listings while the shopper journey remains focused on discovery.</p></div><aside class="basket-summary"><span>Sample listings</span><strong>3 · illustrative</strong></aside></section>"""
    return {"index.html": _document(spec, body)}


def _web_appointment(spec: Mapping[str, Any]) -> dict[str, str]:
    scope = _feature_list(spec)
    body = f"""{_nav(("#home", "Overview"), ("#service", "Choose service"), ("admin.html", "Operations"))}<section id="home" class="screen"><div class="split"><div><p class="eyebrow">CUSTOMER JOURNEY</p><h2>Choose a service, then find a time that works.</h2><p class="lede">A local booking path shows how service selection, availability and confirmation connect.</p><button type="button" data-action="choose-service">Choose service</button></div><aside><h3>Product scope</h3><ul class="list-clean">{scope}</ul></aside></div></section><section id="service" class="screen"><h2>Select a service</h2><div class="field"><label for="service-choice">Service</label><select id="service-choice" name="service"><option>Consultation</option><option>Onboarding session</option><option>Maintenance visit</option></select></div><a href="#availability">Go to availability</a></section><section id="availability" class="screen"><h2>Available times</h2><p>Choose a sample slot to continue.</p><div class="action-row"><button type="button" class="quiet-button" data-action="choose-time">Today · 14:00</button><button type="button" class="quiet-button" data-action="choose-time">Tomorrow · 09:30</button></div></section><section id="confirmation" class="screen"><h2>Confirm this demo booking</h2><p>No real appointment is created.</p><button type="button" data-action="confirm">Confirm demo booking</button></section><section id="complete" class="screen" hidden><h2>Booking recorded locally</h2><p>The simulated schedule now reflects this choice.</p></section><p id="demo-status" class="state" role="status" aria-live="polite">Choose a service to begin.</p>"""
    admin_body = """<nav class="surface-nav" aria-label="Operations navigation"><a href="index.html">Customer booking</a><a href="#admin">Schedule</a></nav><section id="admin" class="screen"><h2>Manage availability</h2><table><thead><tr><th>Service</th><th>Local sample state</th></tr></thead><tbody><tr><td>Consultation</td><td>Available · illustrative</td></tr><tr><td>Onboarding session</td><td>Pending validation</td></tr></tbody></table></section>"""
    app_body = """<nav class="surface-nav" aria-label="Booking navigation"><a href="index.html#service">Select service</a><a href="index.html#availability">Choose time</a><a href="index.html#confirmation">Confirm</a><a href="admin.html">Operations</a></nav><section class="screen"><h2>Your appointment</h2><p>Service and time choices from the local simulation appear here.</p></section>"""
    return _multi_page(spec, body, app_body, admin_body)


def _web_learning(spec: Mapping[str, Any]) -> dict[str, str]:
    scope = _feature_list(spec)
    body = f"""{_nav(("#home", "Overview"), ("#catalog", "Catalog"), ("app.html", "Learner space"), ("admin.html", "Creator"))}<section id="home" class="screen"><div class="split"><div><p class="eyebrow">LEARNER JOURNEY</p><h2>Find a useful course and see progress take shape.</h2><p class="lede">Explore a focused catalog, begin a lesson, and update a local progress state.</p><button type="button" data-action="browse-catalog">Explore catalog</button></div><aside><h3>Product scope</h3><ul class="list-clean">{scope}</ul></aside></div></section><section id="catalog" class="screen"><h2>Choose a practical path</h2><div class="split"><div><h3>Applied AI Foundations</h3><p>Short lessons with a visible practice outcome.</p><button type="button" data-action="enroll">View course</button></div><div><h3>Creator preview</h3><p>Course structure and learner-facing progress.</p><a href="admin.html#course-editor">Open creator workspace</a></div></div></section><section id="checkout" class="screen" hidden><h2>Confirm simulated enrollment</h2><p>No payment is processed.</p><button type="button" data-action="confirm-enrollment">Start course</button></section><section id="progress" class="screen" hidden><h2>Your learning path</h2><label for="lesson">Current lesson</label><select id="lesson" name="lesson"><option>Lesson 1 · Foundations</option><option>Lesson 2 · Practice</option></select><button type="button" data-action="mark-progress">Mark lesson complete</button></section><section id="complete" class="screen" hidden><h2>Progress updated locally</h2></section><p id="demo-status" class="state" role="status" aria-live="polite">Explore the catalog to begin.</p>"""
    app_body = """<nav class="surface-nav" aria-label="Learner navigation"><a href="index.html#catalog">Catalog</a><a href="#course">Course</a><a href="#progress">Progress</a><a href="admin.html">Creator</a></nav><section id="course" class="screen"><h2>Applied AI Foundations</h2><p>Three sample lessons and a visible completion state.</p><button type="button" data-action="mark-progress">Complete current lesson</button></section><section id="progress" class="screen"><h2>1 of 3 lessons complete</h2><ul class="list-clean"><li>Foundations · Complete</li><li>Practice · Next</li><li>Capstone · Not started</li></ul></section><p id="demo-status" class="state" role="status" aria-live="polite">Continue your course.</p>"""
    admin_body = """<nav class="surface-nav" aria-label="Creator admin navigation"><a href="index.html">Course catalog</a><a href="app.html">Learner space</a><a href="#course-editor">Course editor</a></nav><section id="course-editor" class="screen"><p class="eyebrow">CREATOR ADMIN</p><h2>Applied AI Foundations</h2><div class="field"><label for="course-status">Course status</label><select id="course-status" name="status"><option>Draft</option><option>Published</option></select></div><button type="button" data-action="publish-course">Publish simulated course</button></section><section id="admin" class="screen"><h2>Creator signals</h2><p>Illustrative course state · learner validation pending</p></section><p id="demo-status" class="state" role="status" aria-live="polite">Draft changes stay local.</p>"""
    return _multi_page(spec, body, app_body, admin_body)


def _web_workspace(spec: Mapping[str, Any]) -> dict[str, str]:
    tasks = _features(spec, 4) or ["Define a milestone", "Review delivery", "Share an update"]
    task_rows = "".join(
        f'<li class="task-row" data-task="{index}">{_e(task)}</li>'
        for index, task in enumerate(tasks[:3])
    )
    first_task = tasks[0]
    body = f"""{_nav(("#workspace", "Workspace"), ("#board", "Project board"), ("#complete", "Delivery"))}<section id="workspace" class="screen"><div class="split"><div><p class="eyebrow">TEAM WORKSPACE</p><h2>Move the project forward together.</h2><p class="lede">A shared work view connects project setup, task ownership and delivery status.</p></div><aside><span class="tag">{_e(str(spec.get("platform", "web browser")))}</span><p class="muted">The example tasks are synthetic.</p></aside></div><div class="eyeline"></div><ul class="list-clean">{_feature_list(spec, 4)}</ul></section><section id="board" class="screen"><h2>Launch planning board</h2><div class="project-board"><section class="board-lane"><h3>To do</h3><ul id="todo-list" class="list-clean">{task_rows}</ul></section><section class="board-lane"><h3>In progress <span class="numeric" id="progress-count">0</span></h3><ul id="progress-list" class="list-clean"></ul></section><section class="board-lane"><h3>Done</h3><ul class="list-clean"><li class="task-row">Set a project outcome</li></ul></section></div><button type="button" data-action="move-task">Move “{_e(first_task)}” to In progress</button><p id="demo-status" class="state" role="status" aria-live="polite">Choose a task to see the board update.</p></section><section id="complete" class="screen"><h2>Delivery view</h2><p>Team changes are local to this interactive prototype.</p></section>"""
    return {"index.html": _document(spec, body)}


def _multi_page(
    spec: Mapping[str, Any], index_body: str, app_body: str, admin_body: str
) -> dict[str, str]:
    if spec.get("domain") == "education":
        marketing_body = f"""{_nav(("index.html#catalog", "Explore courses"), ("app.html", "Learner space"), ("admin.html", "Creator space"))}<section id="marketing-overview" class="screen"><p class="eyebrow">LEARNING MARKETPLACE</p><h2>Find a practical path, then make progress visible.</h2><p class="lede">{_e(str(spec.get("description", "A product concept for learners and creators.")))}</p><ul class="list-clean">{_feature_list(spec, 4)}</ul><a class="primary-link" href="index.html#catalog">Explore the sample catalog</a></section><section class="screen"><h2>For learners and course creators</h2><p>Review the learner and creator workspaces in this local product simulation.</p><div class="action-row"><a href="app.html">Open learner workspace</a><a href="admin.html">Open creator space</a></div></section>"""
        marketing_title = "Learning marketplace"
    else:
        marketing_body = f"""{_nav(("index.html#service", "Choose a service"), ("index.html#availability", "Availability"), ("admin.html", "Service operations"))}<section id="marketing-overview" class="screen"><p class="eyebrow">SERVICE BOOKING</p><h2>Make the next appointment easy to arrange.</h2><p class="lede">{_e(str(spec.get("description", "A service appointment product concept.")))}</p><ul class="list-clean">{_feature_list(spec, 4)}</ul><a class="primary-link" href="index.html#service">Explore sample services</a></section><section class="screen"><h2>Customer and service-team views</h2><p>Booking choices are local examples and do not create an appointment.</p><a href="app.html">Open customer workspace</a></section>"""
        marketing_title = "Service booking"
    return {
        "index.html": _document(spec, index_body),
        "marketing.html": _document(spec, marketing_body, page_title=marketing_title),
        "app.html": _document(spec, app_body, page_title="Customer workspace"),
        "admin.html": _document(spec, admin_body, page_title="Product operations"),
    }


def _desktop(spec: Mapping[str, Any]) -> dict[str, str]:
    features = _feature_list(spec, 5)
    body = f"""{_nav(("#workspace", "Workspace"), ("#tools", "Tools"), ("#result", "Result"))}<section id="workspace" class="window-frame"><div class="window-titlebar"><span class="terminal-light" aria-hidden="true"></span><span>{_e(str(spec.get("product_name", "Application")))}</span><span class="muted">· local desktop concept</span></div><div class="window-content"><div class="split"><aside><h2>Workspace</h2><ul class="list-clean"><li>Overview</li><li>Recent work</li><li>Preferences</li></ul></aside><section><p class="eyebrow">DESKTOP WORKFLOW</p><h2>{_e(str(spec.get("primary_journey", "Work with the product")))}</h2><p class="lede">A proposed desktop window with a tool rail, main work area and visible save state.</p><ul class="list-clean">{features}</ul></section></div><section id="tools" class="screen"><h3>Application settings</h3><div class="field"><label for="workspace-name">Workspace name</label><input id="workspace-name" name="workspace" autocomplete="off" value="Sample workspace"></div><button type="button" data-action="save-settings">Save local settings</button><p id="demo-status" class="state" role="status" aria-live="polite">No external files or preferences are changed.</p></section></div></section><section id="result" class="screen"><h2>Local result</h2><p>The visible state belongs to this simulation.</p></section>"""
    return {"index.html": _document(spec, body)}


def _api(spec: Mapping[str, Any]) -> dict[str, str]:
    body = """<nav class="surface-nav" aria-label="API console navigation"><a href="#request">Request</a><a href="#response">Response</a></nav><div class="api-layout"><section id="request" class="screen"><div class="endpoint"><span class="method">POST</span><code translate="no">/v1/events</code></div><p class="lede">Edit the payload, then run a local transformation simulation. No network request is sent.</p><form id="api-form"><div class="field"><label for="api-payload">Request JSON</label><textarea id="api-payload" name="payload" autocomplete="off" spellcheck="false">{"events":[{"type":"page_view","source":"web"},{"type":"purchase","source":"checkout"}]}</textarea></div><button type="submit" data-action="send-request">Send simulated request</button></form><p id="demo-status" class="state" role="status" aria-live="polite">Ready to validate the sample request.</p></section><section id="response" class="screen"><p class="eyebrow">SIMULATED RESPONSE</p><h2 id="response-title">Response preview</h2><pre id="api-response" class="terminal-output scroll-area" aria-live="polite">No request has been sent.</pre></section></div><section class="screen"><h2>Processing contract</h2><ul class="list-clean"><li>Input is parsed as local JSON</li><li>Validation and output are illustrative</li><li>No API, database, or external service is contacted</li></ul></section>"""
    return {"index.html": _document(spec, body)}


def _pipeline(spec: Mapping[str, Any]) -> dict[str, str]:
    features = _features(spec, 3) or ["Read source data", "Transform records", "Write result"]
    labels = ["Ingest", "Transform", "Publish"]
    nodes = "".join(
        f'<article class="pipeline-node" data-step="{index}" data-state="pending"><p class="eyebrow">STEP {index + 1}</p><h3>{_e(label)}</h3><p>{_e(features[index % len(features)])}</p><span class="tag">Waiting</span></article>'
        for index, label in enumerate(labels)
    )
    body = f"""{_nav(("#source", "Source"), ("#execution", "Run"), ("#output", "Output"))}<section id="source" class="screen"><div class="split"><div><p class="eyebrow">DATA FLOW</p><h2>Follow the work from input to useful output.</h2><p class="lede">Each stage is explicit so a processing result can be traced through the pipeline.</p></div><aside><span class="tag">Synthetic execution data</span><p class="muted">No source files or model endpoints are accessed.</p></aside></div></section><section id="execution" class="screen"><h2>Processing path</h2><div class="pipeline" aria-label="Three-stage data pipeline">{nodes}</div><button type="button" data-action="run-pipeline">Run local simulation</button><p id="demo-status" class="state" role="status" aria-live="polite">Pipeline is waiting to run.</p></section><section id="output" class="screen"><h2>Sample output</h2><div class="table-wrap"><table><thead><tr><th>Stage</th><th>Illustrative status</th><th>Source</th></tr></thead><tbody><tr><td>Transform</td><td id="output-state">Not run</td><td>Synthetic fixture</td></tr></tbody></table></div></section>"""
    return {"index.html": _document(spec, body)}


def _iot(spec: Mapping[str, Any]) -> dict[str, str]:
    body = """<nav class="surface-nav" aria-label="Device navigation"><a href="#device">Device</a><a href="#telemetry">Telemetry</a></nav><section id="device" class="screen"><div class="device-panel"><div><p class="eyebrow">DEVICE STATUS · SIMULATED</p><h2>Cooling unit 04</h2><p class="lede">Change a local sensor value to see a corresponding device state.</p><div class="field"><label for="temperature">Temperature reading</label><input id="temperature" name="temperature" type="range" min="15" max="35" value="22" aria-describedby="reading-help"><span id="reading-help" class="muted">Adjust the sample between 15 °C and 35 °C.</span></div><button type="button" class="quiet-button" data-action="change-reading">Simulate next reading</button></div><aside id="telemetry" class="screen"><p class="eyebrow">LIVE SAMPLE</p><p id="reading-value" class="reading-value numeric">22 °C</p><p id="reading-state" class="tag">Within sample range</p><p id="demo-status" class="state" role="status" aria-live="polite">No device is connected.</p></aside></div></section><section class="screen"><h2>Local telemetry events</h2><ul class="list-clean"><li>Reading changed · simulated</li><li>Alert history · not connected to hardware</li></ul></section>"""
    return {"index.html": _document(spec, body)}


def _hybrid(spec: Mapping[str, Any]) -> dict[str, str]:
    body = f"""{_nav(("#field-app", "Field app"), ("#operations", "Operations"), ("#complete", "Shared state"))}<section class="screen"><div class="split"><div><p class="eyebrow">CONNECTED PRODUCT SURFACES</p><h2>Field work and operations stay in the same journey.</h2><p class="lede">The phone captures a reading; the operations view reflects that local change.</p></div><aside><span class="tag">Mobile + web concept</span></aside></div></section><div class="hybrid-surfaces"><section id="field-app" class="compact-device"><div class="screen"><p class="eyebrow">TECHNICIAN APP</p><h2>Equipment visit</h2><p>Unit 04 · sample record</p><ul class="list-clean">{_feature_list(spec, 3)}</ul><button type="button" data-action="capture-reading">Record field reading</button></div></section><section id="operations" class="window-frame"><div class="window-titlebar"><span class="terminal-light" aria-hidden="true"></span><span>Operations dashboard · local</span></div><div class="window-content"><p class="eyebrow">FLEET STATUS</p><h2 id="operations-state">Waiting for a field update</h2><div class="table-wrap"><table><thead><tr><th>Device</th><th>Last reading</th><th>Status</th></tr></thead><tbody><tr><td>Unit 04</td><td id="hybrid-reading">Not recorded</td><td id="hybrid-alert">Awaiting technician</td></tr></tbody></table></div><button type="button" class="quiet-button" data-action="acknowledge-alert">Acknowledge sample alert</button></div></section></div><section id="complete" class="screen"><h2>Shared prototype state</h2><p id="demo-status" class="state" role="status" aria-live="polite">Each update is local and synthetic; no hardware is connected.</p></section>"""
    return {"index.html": _document(spec, body)}


def _other(spec: Mapping[str, Any]) -> dict[str, str]:
    steps = [
        str(item.get("title", "")) for item in spec.get("screens", []) if isinstance(item, Mapping)
    ]
    steps = steps[:3] or ["Start", "Complete the main action", "Review the result"]
    content = "".join(
        f'<article class="concept-step"><h2>{_e(step)}</h2><p>{_e(str(spec.get("primary_journey", "A proposed product flow")))}</p></article>'
        for step in steps
    )
    body = f"""{_nav(("#concept", "Concept"), ("#flow", "Primary flow"), ("#result", "Outcome"))}<section id="concept" class="screen"><span class="tag">Conceptual representation · classification uncertain</span><h2>Explore the outcome implied by the product brief.</h2><p class="lede">This composition uses the supplied audience, features and journey. Correct the category if another environment better represents the product.</p><ul class="list-clean">{_feature_list(spec, 5)}</ul></section><section id="flow" class="screen"><h2>Proposed product flow</h2><div class="concept-flow">{content}</div><button type="button" data-action="advance-flow">Show simulated outcome</button><section id="result" class="screen" hidden><h2>Local concept state</h2><p>The inferred outcome appears here as an interactive simulation.</p></section><p id="demo-status" class="state" role="status" aria-live="polite">This category is marked as an inference.</p></section>"""
    return {"index.html": _document(spec, body)}


def _script(template_id: str, domain: str) -> str:
    if template_id == "mobile":
        return _mobile_script(domain)
    if template_id == "cli":
        return r"""const scenarios={
"tramuri --help":["tramuri — local product workflow","","Commands:","  init      Initialize a sample project","  status    Show local workflow status","","This response is simulated."].join('\n'),
"tramuri init --starter api":"$ tramuri init --starter api\nCreated a sample API project structure.\nNo files were written.",
"tramuri status":"$ tramuri status\nRun: demo\nStage: product definition\nExperience: interactive simulation"
};
const output=document.querySelector('#terminal-output');
const status=document.querySelector('#demo-status');
function run(command){const response=scenarios[command];if(output)output.textContent='$ '+command+'\\n'+(response||'Command is not available in this simulation. Choose one of the predefined scenarios.');if(status)status.textContent=response?'Predefined scenario completed locally.':'Command is not available in this simulation.';}
document.querySelectorAll('[data-scenario]').forEach((button)=>button.addEventListener('click',()=>run(button.dataset.scenario||'')));
document.querySelector('#command-form')?.addEventListener('submit',(event)=>{event.preventDefault();const input=document.querySelector('#demo-command');run(input?.value.trim()||'');});"""
    if template_id == "web" and domain == "service_booking":
        return """const status=document.querySelector('#demo-status');function announce(message){if(status)status.textContent=message;}document.querySelector('[data-action="choose-service"]')?.addEventListener('click',()=>{announce('Service selected. Choose a sample time.');document.querySelector('#availability')?.scrollIntoView({behavior:'smooth',block:'start'});});document.querySelectorAll('[data-action="choose-time"]').forEach((button)=>button.addEventListener('click',()=>announce('Sample time selected. Confirm the demo booking.')));document.querySelector('[data-action="confirm"]')?.addEventListener('click',()=>{const complete=document.querySelector('#complete');if(complete)complete.hidden=false;announce('Demo booking confirmed');});"""
    if template_id == "web" and domain == "marketplace":
        return """const status=document.querySelector('#demo-status');
function announce(message){if(status)status.textContent=message;}
document.querySelector('[data-action="browse-catalog"]')?.addEventListener('click',()=>{document.querySelector('#catalog')?.scrollIntoView({behavior:'smooth',block:'start'});announce('Sample catalog opened.');});
document.querySelectorAll('[data-action="view-product"]').forEach((button)=>button.addEventListener('click',()=>{document.querySelector('#product')?.scrollIntoView({behavior:'smooth',block:'start'});announce('Sample product details opened.');}));
let count=0;
document.querySelector('[data-action="add-to-cart"]')?.addEventListener('click',()=>{count=1;const value=document.querySelector('#cart-count');const line=document.querySelector('#cart-line');if(value)value.textContent=String(count);if(line)line.textContent='Stoneware table vase · sample item';announce('Sample product added to the local basket.');});
document.querySelector('[data-action="checkout"]')?.addEventListener('click',()=>{const complete=document.querySelector('#complete');if(complete)complete.hidden=false;announce('Sample checkout complete. No order was created.');});"""
    if template_id == "web" and domain == "education":
        return """const status=document.querySelector('#demo-status');function show(id,message){const section=document.querySelector(id);if(section)section.hidden=false;if(status)status.textContent=message;section?.scrollIntoView({behavior:'smooth',block:'start'});}document.querySelector('[data-action="browse-catalog"]')?.addEventListener('click',()=>show('#catalog','Catalog opened. Choose a learning path.'));document.querySelector('[data-action="enroll"]')?.addEventListener('click',()=>show('#checkout','Course selected. Confirm a simulated enrollment.'));document.querySelector('[data-action="confirm-enrollment"]')?.addEventListener('click',()=>show('#progress','Enrollment complete. Continue with the first lesson.'));document.querySelectorAll('[data-action="mark-progress"]').forEach((button)=>button.addEventListener('click',()=>{const complete=document.querySelector('#complete');if(complete)complete.hidden=false;if(status)status.textContent='Lesson completed. Progress updated locally.';}));document.querySelector('[data-action="publish-course"]')?.addEventListener('click',()=>{if(status)status.textContent='Course draft published in the simulated admin.';});"""
    if template_id == "web":
        return """document.querySelector('[data-action="move-task"]')?.addEventListener('click',()=>{const task=document.querySelector('#todo-list li');const target=document.querySelector('#progress-list');if(task&&target)target.append(task);const count=document.querySelector('#progress-count');if(count)count.textContent=String((Number(count.textContent)||0)+1);const status=document.querySelector('#demo-status');if(status)status.textContent='Task moved to In progress. Project board updated locally.';});"""
    if template_id == "desktop":
        return """document.querySelector('[data-action="save-settings"]')?.addEventListener('click',()=>{const status=document.querySelector('#demo-status');if(status)status.textContent='Settings saved in this local simulation.';});"""
    if template_id == "api":
        return """document.querySelector('#api-form')?.addEventListener('submit',(event)=>{event.preventDefault();const status=document.querySelector('#demo-status');const output=document.querySelector('#api-response');const title=document.querySelector('#response-title');try{const payload=JSON.parse(document.querySelector('#api-payload').value);const events=Array.isArray(payload.events)?payload.events:[];const response={status:'simulated',accepted:events.length,normalized:events.length,source:'local fixture'};if(output)output.textContent=JSON.stringify(response,null,2);if(title)title.textContent='200 · simulated response';if(status)status.textContent='Request validated locally. No network request was sent.';}catch(error){if(output)output.textContent=JSON.stringify({status:'error',message:'Enter valid JSON to preview a response.'},null,2);if(title)title.textContent='400 · simulated validation error';if(status)status.textContent='Request JSON is invalid. Correct the payload and send it again.';}});"""
    if template_id == "pipeline":
        return """document.querySelector('[data-action="run-pipeline"]')?.addEventListener('click',()=>{document.querySelectorAll('.pipeline-node').forEach((node,index)=>{node.dataset.state='complete';const tag=node.querySelector('.tag');if(tag)tag.textContent='Complete';});const output=document.querySelector('#output-state');if(output)output.textContent='Complete · synthetic output';const status=document.querySelector('#demo-status');if(status)status.textContent='Pipeline simulation complete. No external data was processed.';});"""
    if template_id == "iot":
        return """const slider=document.querySelector('#temperature');function update(){const value=Number(slider?.value||22);const reading=document.querySelector('#reading-value');const state=document.querySelector('#reading-state');if(reading)reading.textContent=value+' °C';if(state)state.textContent=value>=30?'Sample threshold exceeded':'Within sample range';const status=document.querySelector('#demo-status');if(status)status.textContent='Simulated sensor reading updated locally.';}slider?.addEventListener('input',update);document.querySelector('[data-action="change-reading"]')?.addEventListener('click',()=>{if(slider)slider.value=String(Math.min(35,Number(slider.value)+4));update();});"""
    if template_id == "hybrid":
        return """document.querySelector('[data-action="capture-reading"]')?.addEventListener('click',()=>{const reading=document.querySelector('#hybrid-reading');const alert=document.querySelector('#hybrid-alert');const state=document.querySelector('#operations-state');const status=document.querySelector('#demo-status');if(reading)reading.textContent='22 °C · synthetic';if(alert)alert.textContent='Review reading';if(state)state.textContent='New technician reading received';if(status)status.textContent='Mobile record and operations dashboard updated locally.';});document.querySelector('[data-action="acknowledge-alert"]')?.addEventListener('click',()=>{const alert=document.querySelector('#hybrid-alert');const status=document.querySelector('#demo-status');if(alert)alert.textContent='Acknowledged · simulated';if(status)status.textContent='Sample alert acknowledged in the local prototype.';});"""
    return """document.querySelector('[data-action="advance-flow"]')?.addEventListener('click',()=>{const result=document.querySelector('#result');if(result)result.hidden=false;const status=document.querySelector('#demo-status');if(status)status.textContent='Inferred outcome shown as a local concept state.';});"""


def _mobile_script(domain: str) -> str:
    """Return local navigation and category-specific mobile interactions."""
    navigation = r"""const tabs=[...document.querySelectorAll('[role="tab"][data-mobile-view]')];
const panels=[...document.querySelectorAll('[data-mobile-screen]')];
function showView(name){tabs.forEach((tab)=>{const active=tab.dataset.mobileView===name;tab.setAttribute('aria-selected',String(active));tab.tabIndex=active?0:-1;});panels.forEach((panel)=>{panel.hidden=panel.dataset.mobileScreen!==name;});}
tabs.forEach((tab,index)=>{tab.addEventListener('click',()=>showView(tab.dataset.mobileView||'overview'));tab.addEventListener('keydown',(event)=>{if(event.key!=='ArrowRight'&&event.key!=='ArrowLeft')return;event.preventDefault();const offset=event.key==='ArrowRight'?1:-1;const next=tabs[(index+offset+tabs.length)%tabs.length];next.focus();showView(next.dataset.mobileView||'overview');});});
"""
    if domain == "personal_finance":
        interaction = r"""const total=document.querySelector('#spending-total');let added=false;
document.querySelector('[data-action="add-expense"]')?.addEventListener('click',()=>{if(total&&!added){total.textContent='$1,248';document.querySelector('#activity-list')?.insertAdjacentHTML('afterbegin','<li>Local coffee <span class="numeric">−$8</span></li>');added=true;}const status=document.querySelector('#demo-status');if(status)status.textContent='Sample expense added. Local budget updated.';});"""
    else:
        interaction = r"""document.querySelector('[data-action="complete-mobile-step"]')?.addEventListener('click',(event)=>{const button=event.currentTarget;const list=document.querySelector('#journey-activity');const entry=document.createElement('li');entry.textContent='Completed · '+(button.dataset.activityLabel||'next step');list?.prepend(entry);const status=document.querySelector('#demo-status');if(status)status.textContent='Next step completed in the local mobile simulation.';});"""
    return navigation + interaction


def _e(value: Any) -> str:
    return escape(str(value), quote=True)
