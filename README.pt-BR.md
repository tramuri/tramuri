# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Logo do Tramuri" width="220">
</p>

<p align="center"><a href="https://github.com/tramuri/tramuri/blob/main/README.md">English</a> · <a href="https://github.com/tramuri/tramuri/blob/main/README.pt-BR.md">Português (Brasil)</a></p>

<p align="center"><strong>Da intenção a resultados verificados.</strong></p>

<p align="center">
  <a href="https://github.com/tramuri/tramuri/actions/workflows/ci.yml"><img src="https://github.com/tramuri/tramuri/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://pypi.org/project/tramuri/"><img src="https://img.shields.io/pypi/v/tramuri" alt="Versão no PyPI"></a>
</p>

O Tramuri é uma infraestrutura open source e local-first para transformar objetivos de produto e
negócio em trabalho coordenado e resultados verificáveis. Ele ajuda pessoas, agentes e ferramentas
a preservar contexto entre sessões e a distinguir trabalho concluído de resultados que foram
realmente avaliados.

O Tramuri foi pensado para Product Engineers, Forward Deployed Engineers (FDEs) e equipes de
produto. Ele não exige um repositório de código, um LLM nem um framework específico de agentes.

## O que o Tramuri oferece

- **Missões e Contratos de Sucesso** para descrever objetivos e critérios de aceitação.
- **Conhecimento persistente** com referências às fontes e histórico.
- **Registros de execução e evidências** que mantêm separados o trabalho, suas referências de apoio
  e sua avaliação.
- **Resultados verificados** com base na avaliação explícita dos critérios. Concluir uma execução
  não marca uma missão como alcançada.
- **Descoberta de projetos** como recurso opcional para inventariar um projeto de código. A análise
  não executa arquivos do repositório e exclui caminhos sensíveis por padrão.
- Uma CLI nativa em Rust e um SDK Python apoiados pelo mesmo core Rust.

O pacote no PyPI oferece o SDK Python pré-compilado. Ainda não há instaladores independentes
pré-compilados para a CLI nativa.

O Tramuri mantém os dados locais do projeto em SQLite. Evidências são registradas como referências;
o Tramuri não busca recursos externos nem armazena o conteúdo dos arquivos analisados como evidência.

## Comece com Python

O SDK Python requer Python 3.11 ou mais recente. Instale o pacote pré-compilado:

```bash
python -m pip install tramuri
```

A wheel inclui o core Rust compilado. Quem instala uma wheel compatível não precisa instalar Rust.

```python
from pathlib import Path

import tramuri

workspace = Path("onboarding-de-clientes")
workspace.mkdir(exist_ok=True)
tramuri.init(workspace)

mission = tramuri.mission_create(
    workspace,
    objective="Validar o fluxo de onboarding de clientes",
    criteria=[
        {
            "description": "O cliente consegue concluir o onboarding",
            "required": True,
            "evidence_expectation": "Aceite do cliente ou conclusão observada",
        }
    ],
)

print(mission["id"])
```

Criar uma missão não exige um repositório Git nem uma análise do projeto. Consulte a
[visão do produto](VISION.md) para conhecer o escopo e os princípios do Tramuri, e o
[changelog](CHANGELOG.md) para ver o histórico de versões.

## Estado do projeto

O Tramuri está em desenvolvimento inicial. Descoberta de projetos, conhecimento persistente,
missões, registros manuais de execução e evidências, e avaliações de critérios fazem parte do
produto atual. A exportação versionada de Context Packs e Evidence Packs está planejada e usará
seleção e autorização explícitas, com proteção segura por padrão.

## Comunidade

- [Como contribuir](CONTRIBUTING.md)
- [Código de Conduta](CODE_OF_CONDUCT.md)
- [Política de Segurança](SECURITY.md)
- [Suporte](SUPPORT.md)
- [Licença](LICENSE) (Apache-2.0)
