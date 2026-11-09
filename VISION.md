# Tramuri — visão do produto

**From intent to verified outcomes.**

Tramuri é uma ferramenta open source, local-first e agent-native de Product
Engineering. Ela transforma uma intenção incompleta em uma definição de produto
estruturada, com hipóteses, decisões, escopo, critérios de sucesso e artefatos
que podem ser verificados.

## Primeira versão

A primeira versão implementa capacidades para transformar uma intenção em uma
definição de produto verificável. Ela ajuda Product Engineers, FDEs e equipes
de produto a organizar:

- problema, público e proposta de valor;
- lacunas de descoberta, hipóteses e evidências pendentes;
- escopo, priorização e incrementos de entrega;
- métricas de sucesso, decisões, riscos e revisão crítica;
- relatório HTML offline e protótipo navegável delimitado pelo escopo.

UX e Engineering possuem contratos de extensão, mas não são simulados como
capacidades presentes.

Tramuri organiza o trabalho do produto por capacidades e artefatos. O produto
não cria uma persona profissional de Product Manager nem exige que a pessoa
usuária conheça uma profissão ou metodologia específica.

## Arquitetura

O pacote `tramuri` é Python 3.12+, distribuído como wheel e sdist com `src/`
layout. O CLI e o core compartilham os mesmos contratos. SQLite local guarda
projetos, execuções, etapas, artefatos, decisões, hipóteses, evidências,
validações e eventos.

Provider adapters isolam as diferenças entre OpenAI Codex, Anthropic Claude Code
e Google Gemini CLI. Cada adapter detecta apenas capacidades observáveis,
instala componentes gerenciados com manifest/hash/backup e preserva arquivos que
não pertencem ao Tramuri. Nenhum MCP, servidor externo, runtime JavaScript,
compilador ou chave de API duplicada é requisito.

## Limites de confiança

Documentos do projeto, código e saídas externas são dados não confiáveis. O
Tramuri não os trata como instruções superiores, não executa scripts encontrados
durante descoberta e não inventa entrevistas, dados de mercado ou validações.
Afirmações permanecem classificadas como fato, inferência, hipótese, decisão ou
pendência de validação.

## Direção futura

Novos módulos e providers podem ser adicionados através de contratos explícitos.
Cloud, billing, RBAC, orquestração externa e módulos UX/Engineering continuam
fora do MVP até que exista requisito e decisão documentada.
