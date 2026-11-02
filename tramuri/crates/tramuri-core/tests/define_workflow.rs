use serde_json::{Value, json};
use tramuri_core::{
    DefineStepId, IntentRouteSource, KnowledgeFilter, MissionPerspective, define_artifact_history,
    define_brief_assess, define_brief_assessment_get, define_next, define_question_answer,
    define_question_request, define_review, define_start, define_status, define_submit,
    knowledge_list, mission_start,
};

fn mission(path: &std::path::Path) -> String {
    mission_start(
        path,
        "Quero criar um SaaS para vender cursos online para alunos",
        None,
        Some(MissionPerspective::ProductEngineer),
    )
    .expect("exploratory Product Engineering Mission should start")
    .mission
    .id
}

fn result(step: DefineStepId) -> Value {
    match step {
        DefineStepId::ObjectiveContext => {
            json!({"original_intent":"Criar SaaS de cursos","interpreted_objective":"Explorar e definir uma plataforma de cursos","known_information":["SaaS","cursos online"],"hypotheses":["instrutores podem ser compradores"],"gaps":["mercado inicial"]})
        }
        DefineStepId::BusinessDiscovery => {
            json!({"problem":"Criadores precisam vender cursos","target_audience":["instrutores"],"business_objectives":["validar demanda"],"value_proposition":"Venda e acesso em um lugar","uncertainties":["disposição a pagar"],"references":[{"step":"objective_context","revision":1}]})
        }
        DefineStepId::Requirements => {
            json!({"functional_requirements":["publicar curso"],"non_functional_requirements":["proteger dados"],"constraints":["sem pagamentos reais no protótipo"],"dependencies":[],"acceptance_criteria":["aluno consegue abrir curso"],"success_criteria":["critério candidato: completar jornada"],"gaps":[],"references":[{"step":"business_discovery","revision":1}]})
        }
        DefineStepId::KnowledgeSynthesis => {
            json!({"classifications":[{"kind":"hypothesis","statement":"Instrutores serão os compradores"}],"references":[{"step":"requirements","revision":1}],"open_questions":["Qual mercado inicial?"]})
        }
        DefineStepId::ProductBrief => {
            json!({"objective":"Definir SaaS de cursos","problem":"Venda e acesso fragmentados","target_audience":["instrutores","alunos"],"value_proposition":"Centralizar descoberta e consumo","main_journey":["descobrir","inscrever-se","assistir"],"requirements":[{"step":"requirements","revision":1}],"mvp_scope":["catálogo","página do curso"],"exclusions":["pagamento real"],"risks":["aquisição de instrutores"],"assumptions":["web app"],"approved_decisions":[],"candidate_success_criteria":["aluno completa jornada de teste"],"next_action":"Prototipar jornada com dados fictícios","references":[{"step":"knowledge_synthesis","revision":1}]})
        }
    }
}

#[test]
fn define_is_sequential_persistent_and_completes_without_verifying_mission_outcome() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let mission_id = mission(workspace.path());
    let started = define_start(workspace.path(), &mission_id).expect("DEFINE should start");
    assert_eq!(started.current_step, Some(DefineStepId::ObjectiveContext));
    assert_eq!(started.completed, 0);
    assert!(
        define_submit(
            workspace.path(),
            &mission_id,
            DefineStepId::BusinessDiscovery,
            result(DefineStepId::BusinessDiscovery),
            None,
            None
        )
        .is_err(),
        "out-of-order step must be rejected"
    );

    let invalid = define_submit(
        workspace.path(),
        &mission_id,
        DefineStepId::ObjectiveContext,
        json!({"original_intent":"missing required fields"}),
        None,
        None,
    )
    .expect("invalid payload should return structured rejection");
    assert!(!invalid.accepted);
    assert_eq!(invalid.progress.completed, 0);
    let corrected = define_submit(
        workspace.path(),
        &mission_id,
        DefineStepId::ObjectiveContext,
        result(DefineStepId::ObjectiveContext),
        None,
        None,
    )
    .expect("corrected payload should be accepted");
    assert!(corrected.accepted);
    assert_eq!(corrected.next_step, Some(DefineStepId::BusinessDiscovery));

    for step in [
        DefineStepId::BusinessDiscovery,
        DefineStepId::Requirements,
        DefineStepId::KnowledgeSynthesis,
    ] {
        let revision = define_status(workspace.path(), &mission_id)
            .expect("status should persist")
            .revision;
        let submitted = define_submit(
            workspace.path(),
            &mission_id,
            step,
            result(step),
            Some(revision),
            Some(step_skill(step)),
        )
        .expect("step submission should return");
        assert!(submitted.accepted, "unexpected rejection: {submitted:?}");
    }
    // Reopen through a new Core call, as a new process/session would.
    let resumed = define_next(workspace.path(), &mission_id).expect("workflow should resume");
    assert_eq!(
        resumed.current_step.as_ref().map(|s| s.id),
        Some(DefineStepId::ProductBrief)
    );
    assert_eq!(resumed.progress.completed, 4);
    assert_eq!(resumed.accepted_inputs.len(), 4);
    let brief = define_submit(
        workspace.path(),
        &mission_id,
        DefineStepId::ProductBrief,
        result(DefineStepId::ProductBrief),
        None,
        None,
    )
    .expect("brief should complete DEFINE");
    assert!(brief.accepted);
    assert_eq!(brief.workflow_state, tramuri_core::DefineState::Completed);
    assert_eq!(brief.progress.completed, 5);
    assert!(
        knowledge_list(workspace.path(), KnowledgeFilter::default())
            .unwrap()
            .is_empty(),
        "DEFINE hypotheses and proposals must not be copied into Knowledge automatically"
    );
    let repeated = define_submit(
        workspace.path(),
        &mission_id,
        DefineStepId::ProductBrief,
        result(DefineStepId::ProductBrief),
        Some(0),
        None,
    )
    .expect("retry after a lost completion response should be idempotent");
    assert!(repeated.accepted);
    assert_eq!(repeated.accepted_artifact_id, brief.accepted_artifact_id);
    assert_eq!(
        define_next(workspace.path(), &mission_id)
            .expect("completed state should be retrievable")
            .next_action,
        "define_complete"
    );
}

#[test]
fn stale_revision_submission_is_rejected_without_advancing() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let id = mission(workspace.path());
    define_start(workspace.path(), &id).unwrap();
    define_submit(
        workspace.path(),
        &id,
        DefineStepId::ObjectiveContext,
        result(DefineStepId::ObjectiveContext),
        Some(0),
        None,
    )
    .unwrap();
    let stale = define_submit(
        workspace.path(),
        &id,
        DefineStepId::BusinessDiscovery,
        result(DefineStepId::BusinessDiscovery),
        Some(0),
        None,
    );
    assert!(stale.is_err());
    let status = define_status(workspace.path(), &id).unwrap();
    assert_eq!(status.current_step, Some(DefineStepId::BusinessDiscovery));
    assert_eq!(status.completed, 1);
}

#[test]
fn reviewing_accepted_brief_preserves_history_and_survives_reopen() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let id = mission(workspace.path());
    define_start(workspace.path(), &id).unwrap();
    for step in [
        DefineStepId::ObjectiveContext,
        DefineStepId::BusinessDiscovery,
        DefineStepId::Requirements,
        DefineStepId::KnowledgeSynthesis,
        DefineStepId::ProductBrief,
    ] {
        let submit = define_submit(workspace.path(), &id, step, result(step), None, None).unwrap();
        assert!(submit.accepted, "{submit:?}");
    }

    let mut revised = result(DefineStepId::ProductBrief);
    revised["next_action"] = json!("Prototipar também o painel do instrutor");
    let invalid_decision_change = {
        revised["approved_decisions"] = json!(["Alterar o modelo de negócio"]);
        define_review(
            workspace.path(),
            &id,
            DefineStepId::ProductBrief,
            revised.clone(),
            "revisão do modelo",
            "Product Engineer",
            "user_request",
            None,
            false,
            None,
        )
    };
    assert!(
        invalid_decision_change.is_err(),
        "approved decision changes require explicit authorization"
    );
    revised["approved_decisions"] = json!([]);
    let first_revision = define_status(workspace.path(), &id).unwrap().revision;
    let review = define_review(
        workspace.path(),
        &id,
        DefineStepId::ProductBrief,
        revised.clone(),
        "detalhar a jornada do instrutor",
        "Product Engineer",
        "user_request",
        Some(first_revision),
        false,
        None,
    )
    .expect("valid review should persist");
    assert!(review.accepted);
    assert_eq!(review.workflow_state, tramuri_core::DefineState::Completed);
    assert_eq!(review.progress.completed, 5);

    // A fresh Core read simulates a process restart. Both artifact revisions remain available.
    let resumed = define_next(workspace.path(), &id).unwrap();
    assert_eq!(resumed.accepted_inputs.len(), 5);
    let briefs =
        define_artifact_history(workspace.path(), &id, Some(DefineStepId::ProductBrief)).unwrap();
    let briefs = briefs
        .iter()
        .filter(|artifact| artifact.step == DefineStepId::ProductBrief)
        .collect::<Vec<_>>();
    assert_eq!(briefs.len(), 2);
    assert_eq!(briefs[0].revision, 1);
    assert_eq!(briefs[1].revision, 2);
    assert_eq!(briefs[0].payload, result(DefineStepId::ProductBrief));
    assert_eq!(briefs[1].payload, revised);
    assert_eq!(
        briefs[1].provenance["review"]["reason"],
        "detalhar a jornada do instrutor"
    );
    assert_eq!(
        briefs[1].provenance["review"]["declared_author"],
        "Product Engineer"
    );
    assert_eq!(briefs[1].provenance["review"]["origin"], "user_request");
    assert_eq!(
        resumed
            .accepted_inputs
            .iter()
            .filter(|artifact| artifact.id == review.accepted_artifact_id.as_deref().unwrap())
            .count(),
        1
    );
}

#[test]
fn upstream_review_marks_only_dependent_steps_stale_and_requires_current_references() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let id = mission(workspace.path());
    define_start(workspace.path(), &id).unwrap();
    for step in [
        DefineStepId::ObjectiveContext,
        DefineStepId::BusinessDiscovery,
        DefineStepId::Requirements,
        DefineStepId::KnowledgeSynthesis,
        DefineStepId::ProductBrief,
    ] {
        assert!(
            define_submit(workspace.path(), &id, step, result(step), None, None)
                .unwrap()
                .accepted
        );
    }

    let mut revised_business = result(DefineStepId::BusinessDiscovery);
    revised_business["target_audience"] = json!(["instrutores independentes"]);
    let reviewed = define_review(
        workspace.path(),
        &id,
        DefineStepId::BusinessDiscovery,
        revised_business,
        "corrigir público-alvo após nova informação",
        "FDE declarado",
        "user_request",
        None,
        false,
        None,
    )
    .unwrap();
    assert!(reviewed.accepted);
    assert_eq!(reviewed.workflow_state, tramuri_core::DefineState::Stale);
    assert_eq!(reviewed.next_step, Some(DefineStepId::Requirements));
    let status = define_status(workspace.path(), &id).unwrap();
    assert_eq!(status.completed, 2);
    assert_eq!(
        status.steps[0].state,
        tramuri_core::DefineStepState::Completed
    );
    assert_eq!(
        status.steps[1].state,
        tramuri_core::DefineStepState::Completed
    );
    assert_eq!(status.steps[2].state, tramuri_core::DefineStepState::Stale);
    assert_eq!(status.steps[3].state, tramuri_core::DefineStepState::Stale);
    assert_eq!(status.steps[4].state, tramuri_core::DefineStepState::Stale);

    let next = define_next(workspace.path(), &id).unwrap();
    assert_eq!(
        next.current_step.as_ref().unwrap().id,
        DefineStepId::Requirements
    );
    assert_eq!(
        next.accepted_inputs.len(),
        2,
        "stale artifacts must not be given as current context"
    );
    let old_reference = define_submit(
        workspace.path(),
        &id,
        DefineStepId::Requirements,
        result(DefineStepId::Requirements),
        None,
        None,
    )
    .unwrap();
    assert!(
        !old_reference.accepted,
        "revalidation must point at revision 2 of business discovery"
    );
    assert!(
        old_reference
            .errors
            .iter()
            .any(|error| error.code == "stale_reference")
    );

    let mut requirements = result(DefineStepId::Requirements);
    requirements["references"] = json!([{"step":"business_discovery","revision":2}]);
    assert!(
        define_submit(
            workspace.path(),
            &id,
            DefineStepId::Requirements,
            requirements,
            None,
            None
        )
        .unwrap()
        .accepted
    );
    assert_eq!(
        define_status(workspace.path(), &id).unwrap().current_step,
        Some(DefineStepId::KnowledgeSynthesis)
    );
    let mut synthesis = result(DefineStepId::KnowledgeSynthesis);
    synthesis["references"] = json!([{"step":"requirements","revision":2}]);
    assert!(
        define_submit(
            workspace.path(),
            &id,
            DefineStepId::KnowledgeSynthesis,
            synthesis,
            None,
            None
        )
        .unwrap()
        .accepted
    );
    let mut brief = result(DefineStepId::ProductBrief);
    brief["references"] = json!([{"step":"knowledge_synthesis","revision":2}]);
    brief["requirements"] = json!([{"step":"requirements","revision":2}]);
    assert!(
        define_submit(
            workspace.path(),
            &id,
            DefineStepId::ProductBrief,
            brief,
            None,
            None
        )
        .unwrap()
        .accepted
    );
    let final_status = define_status(workspace.path(), &id).unwrap();
    assert_eq!(final_status.state, tramuri_core::DefineState::Completed);
    assert_eq!(final_status.completed, 5);
}

#[test]
fn semantic_brief_assessment_is_host_declared_traced_and_does_not_complete_domain_outcomes() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let id = mission(workspace.path());
    define_start(workspace.path(), &id).unwrap();
    for step in [
        DefineStepId::ObjectiveContext,
        DefineStepId::BusinessDiscovery,
        DefineStepId::Requirements,
        DefineStepId::KnowledgeSynthesis,
        DefineStepId::ProductBrief,
    ] {
        assert!(
            define_submit(workspace.path(), &id, step, result(step), None, None)
                .unwrap()
                .accepted
        );
    }
    let brief = define_artifact_history(workspace.path(), &id, Some(DefineStepId::ProductBrief))
        .unwrap()
        .remove(0);
    let criteria = [
        ("goal_alignment", "pass", "objective"),
        ("target_consistency", "concern", "target_audience"),
        ("evidence_assumption_separation", "pass", "assumptions"),
        ("success_criteria_fit", "pass", "candidate_success_criteria"),
        ("requirements_traceability", "pass", "requirements"),
        ("decision_authorization", "pass", "approved_decisions"),
        ("internal_consistency", "pass", "main_journey"),
    ];
    let assessment = json!({
        "schema":"tramuri.product-brief-assessment.v1",
        "brief_artifact_id":brief.id,
        "brief_revision":brief.revision,
        "evaluator":"synthetic host evaluator",
        "rubric_version":"tramuri.product-brief-rubric.v1",
        "overall":"concerns",
        "criteria":criteria.iter().map(|(criterion,result,field)| json!({
            "criterion":criterion,"result":result,"rationale":format!("synthetic rubric rationale for {criterion}"),
            "references": if *criterion == "target_consistency" {
                vec![json!({"step":"business_discovery","revision":1,"field":"target_audience"}),json!({"step":"product_brief","revision":1,"field":field})]
            } else { vec![json!({"step":"product_brief","revision":1,"field":field})] }
        })).collect::<Vec<_>>(),
        "limitations":["No customer or market validation was performed."],
        "evaluation_boundary":"host_semantic_assessment_not_core_verified"
    });
    let recorded = define_brief_assess(workspace.path(), &id, assessment.clone()).unwrap();
    assert_eq!(recorded.overall, "concerns");
    assert_eq!(
        recorded.evaluation_boundary,
        "host_semantic_assessment_not_core_verified"
    );
    assert_eq!(
        recorded
            .criteria
            .iter()
            .find(|item| item.criterion == "target_consistency")
            .unwrap()
            .result,
        "concern"
    );
    assert_eq!(
        define_brief_assessment_get(workspace.path(), &id).unwrap(),
        Some(recorded)
    );
    assert_eq!(define_status(workspace.path(), &id).unwrap().completed, 5);

    let mut falsely_positive = assessment;
    falsely_positive["overall"] = json!("pass");
    assert!(define_brief_assess(workspace.path(), &id, falsely_positive).is_err());
}

#[test]
fn clarification_budget_is_persisted_and_third_request_is_rejected() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let id = mission(workspace.path());
    define_start(workspace.path(), &id).expect("DEFINE should start");
    let first = define_question_request(
        workspace.path(),
        &id,
        DefineStepId::ObjectiveContext,
        "Does this assume a two-sided marketplace?",
        &["yes".into(), "no".into()],
        false,
    )
    .expect("first clarification should be recorded");
    define_question_answer(
        workspace.path(),
        &id,
        &first.id,
        "Treat it as an initial hypothesis",
        true,
    )
    .expect("answer should persist");
    define_question_request(
        workspace.path(),
        &id,
        DefineStepId::ObjectiveContext,
        "Should that remain an assumption?",
        &[],
        false,
    )
    .expect("second clarification should be recorded");
    assert!(
        define_question_request(
            workspace.path(),
            &id,
            DefineStepId::ObjectiveContext,
            "One more question",
            &[],
            false
        )
        .is_err()
    );
    assert_eq!(
        define_status(workspace.path(), &id)
            .expect("status should persist")
            .clarification_count,
        2
    );
}

#[test]
fn a_material_decision_blocks_submission_until_answered() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let id = mission(workspace.path());
    let question = define_question_request(
        workspace.path(),
        &id,
        DefineStepId::ObjectiveContext,
        "Confirm whether use of customer data is authorized",
        &[],
        true,
    )
    .expect("blocking question should persist");
    assert_eq!(
        define_status(workspace.path(), &id).unwrap().state,
        tramuri_core::DefineState::Blocked
    );
    assert!(
        define_submit(
            workspace.path(),
            &id,
            DefineStepId::ObjectiveContext,
            result(DefineStepId::ObjectiveContext),
            None,
            None
        )
        .is_err()
    );
    define_question_answer(
        workspace.path(),
        &id,
        &question.id,
        "Use synthetic data only",
        true,
    )
    .expect("decision answer should unblock");
    assert!(
        define_submit(
            workspace.path(),
            &id,
            DefineStepId::ObjectiveContext,
            result(DefineStepId::ObjectiveContext),
            None,
            None
        )
        .unwrap()
        .accepted
    );
}

#[test]
fn blocking_decisions_require_explicit_answers_and_all_must_be_resolved() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let id = mission(workspace.path());
    let first = define_question_request(
        workspace.path(),
        &id,
        DefineStepId::ObjectiveContext,
        "Is customer data authorized for this discovery?",
        &[],
        true,
    )
    .expect("first blocking decision should persist");
    let second = define_question_request(
        workspace.path(),
        &id,
        DefineStepId::ObjectiveContext,
        "May the discovery use production credentials?",
        &[],
        true,
    )
    .expect("second blocking decision should persist");

    assert!(
        define_question_answer(
            workspace.path(),
            &id,
            &first.id,
            "Use synthetic data",
            false,
        )
        .is_err()
    );

    define_question_answer(workspace.path(), &id, &first.id, "Use synthetic data", true)
        .expect("explicit answer should be recorded");
    assert_eq!(
        define_status(workspace.path(), &id).unwrap().state,
        tramuri_core::DefineState::Blocked,
        "the other unresolved material decision must keep the step blocked"
    );

    define_question_answer(
        workspace.path(),
        &id,
        &second.id,
        "No production credentials",
        true,
    )
    .expect("second explicit answer should be recorded");
    assert_eq!(
        define_status(workspace.path(), &id).unwrap().state,
        tramuri_core::DefineState::InProgress
    );
}

#[test]
fn later_step_contracts_require_typed_references_to_accepted_predecessors() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let id = mission(workspace.path());
    define_start(workspace.path(), &id).unwrap();
    define_submit(
        workspace.path(),
        &id,
        DefineStepId::ObjectiveContext,
        result(DefineStepId::ObjectiveContext),
        None,
        None,
    )
    .unwrap();

    let without_reference = define_submit(
        workspace.path(),
        &id,
        DefineStepId::BusinessDiscovery,
        json!({"problem":"A problem","target_audience":[],"business_objectives":[],"value_proposition":"A value","uncertainties":[],"references":[]}),
        None,
        None,
    )
    .unwrap();
    assert!(!without_reference.accepted);
    assert!(
        without_reference
            .errors
            .iter()
            .any(|error| { error.code == "missing_previous_step_reference" })
    );

    let invalid_reference_type = define_submit(
        workspace.path(),
        &id,
        DefineStepId::BusinessDiscovery,
        json!({"problem":"A problem","target_audience":[],"business_objectives":[],"value_proposition":"A value","uncertainties":[],"references":"objective_context"}),
        None,
        None,
    )
    .unwrap();
    assert!(!invalid_reference_type.accepted);
    assert!(
        invalid_reference_type
            .errors
            .iter()
            .any(|error| { error.code == "invalid_field_type" && error.field == "references" })
    );
}

#[test]
fn define_requires_product_engineering_perspective() {
    let workspace = tempfile::tempdir().expect("workspace should be created");
    let started = mission_start(
        workspace.path(),
        "Preciso integrar a API de um cliente",
        None,
        Some(MissionPerspective::Fde),
    )
    .expect("FDE Mission should start");
    assert_eq!(started.route.source, IntentRouteSource::Explicit);
    assert!(define_start(workspace.path(), &started.mission.id).is_err());
}

fn step_skill(step: DefineStepId) -> &'static str {
    match step {
        DefineStepId::ObjectiveContext => "tramuri-pe-objective-context",
        DefineStepId::BusinessDiscovery => "tramuri-pe-business-discovery",
        DefineStepId::Requirements => "tramuri-pe-requirements",
        DefineStepId::KnowledgeSynthesis => "tramuri-pe-evidence-hypotheses",
        DefineStepId::ProductBrief => "tramuri-pe-product-brief",
    }
}
