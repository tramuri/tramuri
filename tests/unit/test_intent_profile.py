from tramuri.modules.pm.intent import parse_intent


def test_course_intent_preserves_the_product_domain() -> None:
    profile = parse_intent(
        """# SaaS para vender cursos de IA

        ## Intenção
        Quero vender cursos online para pessoas que querem aprender IA.

        ## Resultado desejado
        A pessoa compra um curso e acompanha seu progresso.

        ## Hipóteses iniciais
        - Alunos pagam por uma trilha prática.
        """
    )

    assert profile.archetype == "learning"
    assert profile.title == "SaaS para vender cursos de IA"
    assert "curso" in profile.intent.lower()
    assert "alunos" in " ".join(profile.audience).lower()
    assert "progresso" in profile.desired_outcome.lower()


def test_appointment_intent_uses_service_archetype() -> None:
    profile = parse_intent("Quero criar uma plataforma para agendamento de consultas.")

    assert profile.archetype == "appointment"
    assert profile.title.startswith("Plataforma")
    assert profile.desired_outcome
