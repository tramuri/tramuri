(() => {
  const copy = {
    en: {
      "skip.main": "Skip to report",
      "aria.navigation": "Report navigation",
      "aria.displaySettings": "Report display settings",
      "aria.language": "Report language",
      "aria.workflowProgress": "Persisted workflow progress",
      "aria.stages": "Product development stages",
      "aria.graphScroll": "Scrollable decision and evidence map",
      "aria.graph": "Recorded product relationships",
      "aria.legend": "Relationship map legend",
      "brand.report": "Product report",
      "nav.contents": "Report contents",
      "nav.overview": "Overview",
      "nav.journey": "PDLC journey",
      "nav.intelligence": "Product intelligence",
      "nav.market": "Market research",
      "nav.scope": "Scope & experience",
      "nav.traceability": "Decision & evidence",
      "nav.execution": "Execution & outcomes",
      "nav.demo": "Product experience",
      "run.identifier": "Run identifier",
      "topbar.context": "Product intelligence",
      "controls.theme": "Theme",
      "controls.themeLight": "Switch to light theme",
      "controls.themeDark": "Switch to dark theme",
      "controls.print": "Print",
      "controls.printTitle": "Print or save as PDF",
      "footer.offline": "Generated locally · Product content reflects the recorded run",
      "overview.label": "Tramuri / Product report",
      "overview.problemLabel": "Product intent",
      "overview.summaryFallback": "The run records the current product framing, evidence and decisions.",
      "meta.mode": "Mode",
      "meta.provider": "Provider",
      "meta.archetype": "Product shape",
      "metric.progress": "Workflow progress",
      "metric.stages": "Stages completed",
      "metric.evidence": "Evidence sources",
      "metric.decisions": "Recorded decisions",
      "metric.progressNote": "Persisted stage state",
      "metric.evidenceNote": "External sources in the research pack",
      "metric.decisionsNote": "Decisions in this run",
      "next.title": "Next recorded action",
      "section.journey.title": "The product journey",
      "section.journey.description": "A stage-by-stage view of this run, taken from persisted workflow state.",
      "journey.progress": "completed",
      "journey.owner": "Owner",
      "journey.attempts": "attempts",
      "journey.dependencies": "Depends on",
      "journey.noDependencies": "No recorded dependency",
      "section.intelligence.title": "Product intelligence",
      "section.intelligence.description": "The product's problem, audience and value proposition, with uncertainty kept visible.",
      "label.problem": "Problem",
      "label.users": "Target users",
      "label.needs": "User needs",
      "label.value": "Value proposition",
      "label.archetype": "Product shape",
      "label.constraints": "Constraints",
      "label.discoveryGaps": "Discovery gaps",
      "label.notRecorded": "Not recorded",
      "notice.evidence.title": "Evidence boundary",
      "notice.evidence.body": "Interviews, market numbers and user validation are presented as facts only when supporting evidence exists.",
      "section.market.title": "Market context & opportunity",
      "section.market.description": "Competitive context, buying motivations and market signals are included only from a validated research pack.",
      "market.status": "Research status",
      "market.query": "Research question",
      "market.date": "Research date",
      "market.competitors": "Competitive landscape",
      "market.product": "Product",
      "market.positioning": "Positioning",
      "market.strengths": "Strengths",
      "market.gaps": "Observed gaps",
      "market.signals": "Market signals",
      "market.buying": "Buying motivations",
      "market.tradeoffs": "Build · buy · partner",
      "market.sizing": "Audience sizing",
      "market.pending": "No market evidence",
      "market.pendingDetail": "This run has no imported, source-backed market research. The report leaves competitors, market size and buying signals unasserted.",
      "market.sources": "Sources & traceability",
      "market.publisher": "Publisher",
      "market.confidence": "Confidence",
      "market.sourceNote": "Claims retain their source identifiers from the research pack.",
      "section.hypotheses.title": "Assumptions to test",
      "section.hypotheses.description": "Hypotheses remain distinct from validated evidence and keep their risk and test signal attached.",
      "hypothesis.statement": "Hypothesis",
      "hypothesis.risk": "Risk",
      "hypothesis.test": "Test method",
      "hypothesis.signal": "Success signal",
      "hypothesis.state": "Evidence state",
      "hypothesis.empty": "No hypotheses recorded",
      "hypothesis.emptyDetail": "This run has not persisted any product hypotheses.",
      "section.scope.title": "Scope & product experience",
      "section.scope.description": "The defined product scope and generated experience are shown separately from production engineering.",
      "scope.in": "Included in scope",
      "scope.out": "Outside this increment",
      "scope.features": "Prioritized capabilities",
      "scope.empty": "No scope details recorded",
      "scope.emptyDetail": "The current run has not persisted a product scope artifact.",
      "experience.disclaimer": "The generated experience is an offline product exploration, not a production application. Synthetic data is labeled in the demo.",
      "experience.surfaces": "Generated surfaces",
      "experience.marketing": "Marketing site",
      "experience.customer": "Customer experience",
      "experience.admin": "Admin experience",
      "experience.open": "Open experience",
      "experience.traceability": "Screen and interaction traceability",
      "section.graph.title": "Decision & evidence map",
      "section.graph.description": "Recorded dependencies and source references connect the product reasoning. Unlinked records remain unconnected.",
      "graph.relationshipRule": "Lines appear only for persisted stage dependencies, artifact lineage or exact evidence references.",
      "graph.scrollHint": "Scroll horizontally to explore the full map.",
      "graph.stage": "Workflow stage",
      "graph.artifact": "Artifact",
      "graph.evidence": "Evidence",
      "graph.hypothesis": "Hypothesis",
      "graph.decision": "Decision",
      "graph.empty": "No explicit relationships recorded",
      "graph.emptyDetail": "The run has no stored dependencies or source references to connect. Individual records remain available below.",
      "graph.accessible": "Recorded relationships",
      "section.evidence.title": "Evidence & decisions",
      "section.evidence.description": "Claims, decisions and sources stay traceable to their stored classifications and references.",
      "evidence.claim": "Claim",
      "evidence.classification": "Classification",
      "evidence.source": "Source reference",
      "evidence.freshness": "Freshness",
      "evidence.verifier": "Verified by",
      "evidence.empty": "No evidence records",
      "evidence.emptyDetail": "The run has not persisted evidence records. This does not imply validation.",
      "decision.title": "Decision",
      "decision.outcome": "Selected direction",
      "decision.rationale": "Rationale",
      "decision.classification": "Classification",
      "decision.empty": "No decisions recorded",
      "decision.emptyDetail": "The run has not persisted product decisions.",
      "knowledge.title": "Knowledge notes",
      "knowledge.classification": "Classification",
      "knowledge.content": "Recorded information",
      "knowledge.freshness": "Freshness",
      "knowledge.status": "Status",
      "knowledge.empty": "No knowledge entries",
      "knowledge.emptyDetail": "No reusable knowledge was recorded for this run.",
      "section.execution.title": "Execution & outcomes",
      "section.execution.description": "Progress, validation receipts, checkpoints and risks from this run's actual execution.",
      "execution.metrics": "Success criteria",
      "execution.outcomes": "Desired outcomes",
      "execution.validations": "Validation receipts",
      "execution.events": "Workflow events",
      "execution.sensor": "Sensor",
      "execution.severity": "Severity",
      "execution.reviewFindings": "Review findings",
      "execution.status": "Status",
      "execution.checkedAt": "Checked at",
      "execution.runEvent": "Run-level event",
      "execution.metricsName": "Metric",
      "execution.metricsTarget": "Target",
      "execution.metricsState": "State",
      "execution.checkpoints": "Resume checkpoints",
      "execution.noNextAction": "No next action recorded.",
      "execution.sensors": "Sensor results",
      "execution.risks": "Risks & critical review",
      "execution.artifacts": "Generated artifacts",
      "execution.stage": "Stage",
      "execution.check": "Check",
      "execution.result": "Result",
      "execution.details": "Details",
      "execution.checkpoint": "Checkpoint",
      "execution.created": "Created",
      "execution.next": "Next action",
      "execution.artifact": "Artifact",
      "execution.empty": "No records in this section",
      "execution.emptyDetail": "This run has not persisted data for this section.",
      "footer.validation": "Validate open hypotheses and research before making delivery decisions.",
      "overview.decisions": "Key decisions",
      "overview.risks": "Risks to watch",
      "overview.noRationale": "No rationale recorded",
      "evidence.records": "Evidence records",
      "stage.intent": "Intent",
      "stage.framing": "Product framing",
      "stage.discovery": "Discovery",
      "stage.research": "Market research",
      "stage.hypothesis": "Hypotheses",
      "stage.shaping": "Product scope",
      "stage.experiment": "Experiment",
      "stage.success": "Success criteria",
      "stage.governance": "Governance",
      "stage.traceability": "Traceability",
      "stage.delivery": "Delivery plan",
      "stage.risk_review": "Risk review",
      "stage.critical_review": "Critical review",
      "stage.report": "Report generation",
      "stage.navigation": "Product navigation",
      "classification.fact": "Fact",
      "classification.evidence": "Evidence",
      "classification.inference": "Inference",
      "classification.hypothesis": "Hypothesis",
      "classification.assumption": "Assumption",
      "classification.decision": "Decision",
      "classification.pending_validation": "Pending validation",
      "freshness.current": "Current",
      "freshness.stale": "Stale",
      "freshness.unknown": "Unknown",
      "risk.high": "High",
      "risk.medium": "Medium",
      "risk.low": "Low",
      "risk.critical": "Critical",
      "status.completed": "Completed",
      "status.running": "In progress",
      "status.pending": "Pending",
      "status.blocked": "Blocked",
      "status.failed": "Failed",
      "status.skipped": "Skipped",
      "status.valid": "Valid",
      "status.draft": "Draft",
      "status.stale": "Stale",
      "status.superseded": "Superseded",
      "status.passed": "Passed",
      "status.not_run": "Not run",
      "status.not_started": "Not started",
      "status.required": "Required",
      "status.waiting": "Waiting",
      "status.could_not_run": "Could not run",
      "status.unknown": "Unknown"
    },
    pt: {
      "skip.main": "Pular para o relatório",
      "aria.navigation": "Navegação do relatório",
      "aria.displaySettings": "Configurações de exibição do relatório",
      "aria.language": "Idioma do relatório",
      "aria.workflowProgress": "Progresso persistido do fluxo",
      "aria.stages": "Etapas do ciclo de produto",
      "aria.graphScroll": "Mapa de decisões e evidências com rolagem horizontal",
      "aria.graph": "Relações de produto registradas",
      "aria.legend": "Legenda do mapa de relações",
      "brand.report": "Relatório de produto",
      "nav.contents": "Conteúdo do relatório",
      "nav.overview": "Visão geral",
      "nav.journey": "Jornada do ciclo de produto",
      "nav.intelligence": "Inteligência de produto",
      "nav.market": "Pesquisa de mercado",
      "nav.scope": "Escopo e experiência",
      "nav.traceability": "Decisões e evidências",
      "nav.execution": "Execução e resultados",
      "nav.demo": "Experiência do produto",
      "run.identifier": "Identificador da execução",
      "topbar.context": "Inteligência de produto",
      "controls.theme": "Tema",
      "controls.themeLight": "Alternar para tema claro",
      "controls.themeDark": "Alternar para tema escuro",
      "controls.print": "Imprimir",
      "controls.printTitle": "Imprimir ou salvar como PDF",
      "footer.offline": "Gerado localmente · O conteúdo reflete os registros desta execução",
      "overview.label": "Tramuri / Relatório de produto",
      "overview.problemLabel": "Intenção do produto",
      "overview.summaryFallback": "A execução registra o enquadramento, as evidências e as decisões atuais do produto.",
      "meta.mode": "Modo",
      "meta.provider": "Provedor",
      "meta.archetype": "Formato do produto",
      "metric.progress": "Progresso do fluxo",
      "metric.stages": "Etapas concluídas",
      "metric.evidence": "Fontes de evidência",
      "metric.decisions": "Decisões registradas",
      "metric.progressNote": "Estado persistido das etapas",
      "metric.evidenceNote": "Fontes externas no pacote de pesquisa",
      "metric.decisionsNote": "Decisões desta execução",
      "next.title": "Próxima ação registrada",
      "section.journey.title": "Jornada do produto",
      "section.journey.description": "Visão por etapa desta execução, baseada no estado persistido do fluxo.",
      "journey.progress": "concluídas",
      "journey.owner": "Responsável",
      "journey.attempts": "tentativas",
      "journey.dependencies": "Depende de",
      "journey.noDependencies": "Nenhuma dependência registrada",
      "section.intelligence.title": "Inteligência de produto",
      "section.intelligence.description": "Problema, público e proposta de valor, mantendo as incertezas visíveis.",
      "label.problem": "Problema",
      "label.users": "Público-alvo",
      "label.needs": "Necessidades dos usuários",
      "label.value": "Proposta de valor",
      "label.archetype": "Formato do produto",
      "label.constraints": "Restrições",
      "label.discoveryGaps": "Lacunas de descoberta",
      "label.notRecorded": "Não registrado",
      "notice.evidence.title": "Limite das evidências",
      "notice.evidence.body": "Entrevistas, dados de mercado e validação com usuários só aparecem como fatos quando há evidências de suporte.",
      "section.market.title": "Contexto de mercado e oportunidade",
      "section.market.description": "Concorrência, motivos de compra e sinais de mercado aparecem apenas quando há um pacote de pesquisa validado.",
      "market.status": "Estado da pesquisa",
      "market.query": "Pergunta de pesquisa",
      "market.date": "Data da pesquisa",
      "market.competitors": "Cenário competitivo",
      "market.product": "Produto",
      "market.positioning": "Posicionamento",
      "market.strengths": "Pontos fortes",
      "market.gaps": "Lacunas observadas",
      "market.signals": "Sinais de mercado",
      "market.buying": "Motivos de compra",
      "market.tradeoffs": "Construir · comprar · parceria",
      "market.sizing": "Dimensionamento do público",
      "market.pending": "Sem evidências de mercado",
      "market.pendingDetail": "Esta execução não possui pesquisa de mercado importada com fontes. O relatório não afirma concorrentes, tamanho de mercado ou sinais de compra.",
      "market.sources": "Fontes e rastreabilidade",
      "market.publisher": "Publicador",
      "market.confidence": "Confiança",
      "market.sourceNote": "As afirmações mantêm os identificadores de fonte do pacote de pesquisa.",
      "section.hypotheses.title": "Hipóteses a testar",
      "section.hypotheses.description": "Hipóteses permanecem distintas de evidências validadas e mantêm seus riscos e sinais de teste.",
      "hypothesis.statement": "Hipótese",
      "hypothesis.risk": "Risco",
      "hypothesis.test": "Método de teste",
      "hypothesis.signal": "Sinal de sucesso",
      "hypothesis.state": "Estado da evidência",
      "hypothesis.empty": "Nenhuma hipótese registrada",
      "hypothesis.emptyDetail": "Esta execução não persistiu hipóteses de produto.",
      "section.scope.title": "Escopo e experiência do produto",
      "section.scope.description": "O escopo definido e a experiência gerada aparecem separados de uma aplicação de produção.",
      "scope.in": "Incluído no escopo",
      "scope.out": "Fora deste incremento",
      "scope.features": "Capacidades priorizadas",
      "scope.empty": "Nenhum detalhe de escopo registrado",
      "scope.emptyDetail": "Esta execução não persistiu um artefato de escopo do produto.",
      "experience.disclaimer": "A experiência gerada é uma exploração offline do produto, não uma aplicação de produção. Os dados sintéticos estão identificados na demo.",
      "experience.surfaces": "Superfícies geradas",
      "experience.marketing": "Site de apresentação",
      "experience.customer": "Experiência do cliente",
      "experience.admin": "Experiência administrativa",
      "experience.open": "Abrir experiência",
      "experience.traceability": "Rastreabilidade de telas e interações",
      "section.graph.title": "Mapa de decisões e evidências",
      "section.graph.description": "Dependências e referências de fonte registradas conectam o raciocínio do produto. Registros sem vínculo continuam separados.",
      "graph.relationshipRule": "As linhas representam somente dependências de etapas, linhagem de artefatos ou referências explícitas de evidência.",
      "graph.scrollHint": "Deslize ou role na horizontal para explorar o mapa completo.",
      "graph.stage": "Etapa do fluxo",
      "graph.artifact": "Artefato",
      "graph.evidence": "Evidência",
      "graph.hypothesis": "Hipótese",
      "graph.decision": "Decisão",
      "graph.empty": "Nenhuma relação explícita registrada",
      "graph.emptyDetail": "A execução não possui dependências ou referências de fonte persistidas para conectar. Os registros individuais continuam disponíveis abaixo.",
      "graph.accessible": "Relações registradas",
      "section.evidence.title": "Evidências e decisões",
      "section.evidence.description": "Afirmações, decisões e fontes mantêm suas classificações e referências registradas.",
      "evidence.claim": "Afirmação",
      "evidence.classification": "Classificação",
      "evidence.source": "Referência da fonte",
      "evidence.freshness": "Atualidade",
      "evidence.verifier": "Verificado por",
      "evidence.empty": "Nenhum registro de evidência",
      "evidence.emptyDetail": "A execução não persistiu registros de evidência. Isso não significa que houve validação.",
      "decision.title": "Decisão",
      "decision.outcome": "Direção escolhida",
      "decision.rationale": "Justificativa",
      "decision.classification": "Classificação",
      "decision.empty": "Nenhuma decisão registrada",
      "decision.emptyDetail": "A execução não persistiu decisões de produto.",
      "knowledge.title": "Notas de conhecimento",
      "knowledge.classification": "Classificação",
      "knowledge.content": "Informação registrada",
      "knowledge.freshness": "Atualidade",
      "knowledge.status": "Estado",
      "knowledge.empty": "Nenhum item de conhecimento",
      "knowledge.emptyDetail": "Nenhum conhecimento reutilizável foi registrado nesta execução.",
      "section.execution.title": "Execução e resultados",
      "section.execution.description": "Progresso, comprovantes de validação, checkpoints e riscos desta execução.",
      "execution.metrics": "Critérios de sucesso",
      "execution.outcomes": "Resultados desejados",
      "execution.validations": "Registros de validação",
      "execution.events": "Eventos do fluxo",
      "execution.sensor": "Sensor",
      "execution.severity": "Severidade",
      "execution.reviewFindings": "Achados da revisão",
      "execution.status": "Estado",
      "execution.checkedAt": "Verificado em",
      "execution.runEvent": "Evento da execução",
      "execution.metricsName": "Métrica",
      "execution.metricsTarget": "Meta",
      "execution.metricsState": "Estado",
      "execution.checkpoints": "Checkpoints de retomada",
      "execution.noNextAction": "Nenhuma próxima ação registrada.",
      "execution.sensors": "Resultados dos sensores",
      "execution.risks": "Riscos e revisão crítica",
      "execution.artifacts": "Artefatos gerados",
      "execution.stage": "Etapa",
      "execution.check": "Verificação",
      "execution.result": "Resultado",
      "execution.details": "Detalhes",
      "execution.checkpoint": "Checkpoint",
      "execution.created": "Criado em",
      "execution.next": "Próxima ação",
      "execution.artifact": "Artefato",
      "execution.empty": "Nenhum registro nesta seção",
      "execution.emptyDetail": "Esta execução não persistiu dados para esta seção.",
      "footer.validation": "Valide as hipóteses e pesquisas pendentes antes de decidir sobre a entrega.",
      "overview.decisions": "Decisões principais",
      "overview.risks": "Riscos a acompanhar",
      "overview.noRationale": "Nenhuma justificativa registrada",
      "evidence.records": "Registros de evidência",
      "stage.intent": "Intenção",
      "stage.framing": "Enquadramento do produto",
      "stage.discovery": "Descoberta",
      "stage.research": "Pesquisa de mercado",
      "stage.hypothesis": "Hipóteses",
      "stage.shaping": "Escopo do produto",
      "stage.experiment": "Experimento",
      "stage.success": "Critérios de sucesso",
      "stage.governance": "Governança",
      "stage.traceability": "Rastreabilidade",
      "stage.delivery": "Plano de entrega",
      "stage.risk_review": "Revisão de riscos",
      "stage.critical_review": "Revisão crítica",
      "stage.report": "Geração do relatório",
      "stage.navigation": "Navegação do produto",
      "classification.fact": "Fato",
      "classification.evidence": "Evidência",
      "classification.inference": "Inferência",
      "classification.hypothesis": "Hipótese",
      "classification.assumption": "Suposição",
      "classification.decision": "Decisão",
      "classification.pending_validation": "Validação pendente",
      "freshness.current": "Atual",
      "freshness.stale": "Desatualizado",
      "freshness.unknown": "Desconhecido",
      "risk.high": "Alto",
      "risk.medium": "Médio",
      "risk.low": "Baixo",
      "risk.critical": "Crítico",
      "status.completed": "Concluído",
      "status.running": "Em andamento",
      "status.pending": "Pendente",
      "status.blocked": "Bloqueado",
      "status.failed": "Falhou",
      "status.skipped": "Ignorado",
      "status.valid": "Válido",
      "status.draft": "Rascunho",
      "status.stale": "Desatualizado",
      "status.superseded": "Substituído",
      "status.passed": "Aprovado",
      "status.not_run": "Não executado",
      "status.not_started": "Não iniciado",
      "status.required": "Necessário",
      "status.waiting": "Aguardando",
      "status.could_not_run": "Não foi possível executar",
      "status.unknown": "Desconhecido"
    }
  };

  const statusAliases = {
    "stage-status": "status",
    "not-run": "not_run",
    "not-started": "not_started",
    "could-not-run": "could_not_run"
  };

  const translate = (language) => {
    const selected = language === "pt-BR" ? "pt" : "en";
    const dictionary = copy[selected];
    document.documentElement.lang = language;
    document.querySelectorAll("[data-i18n]").forEach((element) => {
      const value = dictionary[element.dataset.i18n];
      if (value) element.textContent = value;
    });
    document.querySelectorAll("[data-i18n-aria]").forEach((element) => {
      const value = dictionary[element.dataset.i18nAria];
      if (value) element.setAttribute("aria-label", value);
    });
    document.querySelectorAll("[data-i18n-title]").forEach((element) => {
      const value = dictionary[element.dataset.i18nTitle];
      if (value) element.setAttribute("title", value);
    });
    document.querySelectorAll("[data-status-label]").forEach((element) => {
      const raw = element.dataset.statusLabel || "unknown";
      const key = statusAliases[raw] || raw;
      element.textContent = dictionary[`status.${key}`] || raw.replaceAll("_", " ");
    });
    document.querySelectorAll("[data-stage-label]").forEach((element) => {
      const raw = element.dataset.stageLabel || "";
      const key = raw.replaceAll("-", "_");
      if (dictionary[`stage.${key}`]) element.textContent = dictionary[`stage.${key}`];
    });
    document.querySelectorAll("[data-classification-label]").forEach((element) => {
      const raw = element.dataset.classificationLabel || "unknown";
      element.textContent = dictionary[`classification.${raw}`] || raw.replaceAll("_", " ");
    });
    document.querySelectorAll("[data-freshness-label]").forEach((element) => {
      const raw = element.dataset.freshnessLabel || "unknown";
      element.textContent = dictionary[`freshness.${raw}`] || raw.replaceAll("_", " ");
    });
    document.querySelectorAll("[data-risk-label]").forEach((element) => {
      const raw = element.dataset.riskLabel || "unknown";
      const riskKey = `risk.${raw.toLowerCase()}`;
      if (dictionary[riskKey]) element.textContent = dictionary[riskKey];
    });
    document.querySelectorAll("[data-language]").forEach((button) => {
      button.setAttribute("aria-pressed", String(button.dataset.language === language));
    });
    const themeButton = document.querySelector("[data-theme-toggle]");
    if (themeButton) updateThemeLabel(themeButton);
  };

  const updateThemeLabel = (button) => {
    const isDark = document.documentElement.dataset.theme === "dark";
    const key = isDark ? "controls.themeLight" : "controls.themeDark";
    const language = document.documentElement.lang === "pt-BR" ? "pt" : "en";
    const label = copy[language][key];
    button.setAttribute("aria-label", label);
    button.setAttribute("title", label);
  };

  document.querySelectorAll("[data-language]").forEach((button) => {
    button.addEventListener("click", () => translate(button.dataset.language));
  });

  const themeButton = document.querySelector("[data-theme-toggle]");
  if (themeButton) {
    updateThemeLabel(themeButton);
    themeButton.addEventListener("click", () => {
      const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
      document.documentElement.dataset.theme = next;
      const themeMeta = document.querySelector('meta[name="theme-color"]');
      if (themeMeta) themeMeta.content = next === "dark" ? "#0b2028" : "#f5f7f4";
      updateThemeLabel(themeButton);
    });
  }

  const printButton = document.querySelector("[data-print]");
  if (printButton) printButton.addEventListener("click", () => window.print());

  const sections = Array.from(document.querySelectorAll("#overview, .report-section[id]"));
  let navigationFrame = 0;
  const updateActiveNavigation = () => {
    if (navigationFrame) return;
    navigationFrame = window.requestAnimationFrame(() => {
      navigationFrame = 0;
      const activationLine = Math.min(240, window.innerHeight * 0.28);
      let activeSection = sections[0];
      sections.forEach((section) => {
        if (section.getBoundingClientRect().top <= activationLine) activeSection = section;
      });
      if (window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 4) {
        activeSection = sections[sections.length - 1];
      }
      document.querySelectorAll(".report-nav a[aria-current]").forEach((item) => item.removeAttribute("aria-current"));
      const link = activeSection && document.querySelector(`.report-nav a[href="#${activeSection.id}"]`);
      if (link) link.setAttribute("aria-current", "location");
    });
  };
  window.addEventListener("scroll", updateActiveNavigation, { passive: true });
  window.addEventListener("hashchange", updateActiveNavigation);
  window.addEventListener("resize", updateActiveNavigation, { passive: true });
  updateActiveNavigation();

  translate("en");
})();
