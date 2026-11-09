# Checklist de Product Experience

- [ ] A tarefa possui direção visual documentada antes da implementação.
- [ ] `Product Experience Classifier` registra categoria, evidências,
      incerteza, componentes híbridos e correções explícitas.
- [ ] A especificação `tramuri.demo.v1` registra jornada, telas, ações,
      plataformas, identidade visual e artefatos de origem para re-renderização.
- [ ] O template corresponde ao ambiente do produto: mobile, web, CLI,
      desktop, API, pipeline, IoT, híbrido ou composição contextual.
- [ ] Dados, ações, erros e respostas são identificados como sintéticos quando
      não pertencem a um sistema real; nenhum shell ou integração é executado.
- [ ] Portal e demo usam HTML semântico, foco visível, `lang`, labels e estado
      anunciado por `aria-live` quando necessário.
- [ ] A demo funciona offline, sem CDN, Node.js ou backend obrigatório.
- [ ] As superficies marketing, cliente e admin existem apenas quando justificadas
      pelo dominio e estao ligadas por navegacao real.
- [ ] A jornada principal tem botões/links funcionais e mudanças observáveis.
- [ ] Cada tela obrigatória é rastreável a um artefato de produto.
- [ ] Sensores verificam arquivos, links, telas, jornada, traceabilidade,
      acessibilidade básica, coerência de dominio, placeholders e dependências externas.
- [ ] A pesquisa, quando usada, possui fontes, datas, confiança, source IDs e
      estado explicito (`imported`, `not_run` ou `blocked`).
- [ ] O relatório mostra concorrência, sinais, motivos de compra, posicionamento,
      sizing e build/buy/partner sem apresentar hipótese como fato.
- [ ] O reviewer é read-only, produz findings específicos e não inventa escopo.
- [ ] A validação de navegador foi executada ou marcada `not_run` com causa.
- [ ] Existe evidência de viewport responsivo e `prefers-reduced-motion`.
- [ ] A entrega registra Product Definition, Product Review, Demo Generation,
      Browser Validation, Critical Review, Correction e Delivery.
- [ ] A ausência de `web-animation-design` está documentada e não é mascarada.
