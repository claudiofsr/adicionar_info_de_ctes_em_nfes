use crate::{
    BUFFER, Chave, Colunas, Config, CteMetadata, Informacoes, NfeMetadata, SpedError, SpedResult,
    fmt_milhares,
};
use csv::{ByteRecord, ReaderBuilder};
use rayon::prelude::*;
use std::{
    collections::{HashMap, hash_map::Entry},
    fs::File,
    io::BufReader,
    path::Path,
};

/// 0.00005 é a metade da quarta casa.
///
/// Qualquer valor absoluto menor será desconsiderado
/// na soma de número de itens de NFe.
const DELTA: f64 = 0.00005;

pub fn f64_to_str(valor: f64) -> String {
    // 1. Formata com 2 casas decimais e arredondamento (ex: 1234.706 -> "1234.71")
    format!("{:.2}", valor.abs())
}

// O Enum não precisa de Default porque ele é usado dentro de um Option
// Mas é boa prática manter Debug e Clone
#[derive(Debug, Clone)]
pub enum DocMetadata {
    Cte(Box<CteMetadata<'static>>),
    Nfe(Box<NfeMetadata<'static>>),
}

/// Sumarizar informações de Documentos Fiscais.
///
/// Armazena (considera apenas o itens de valores não nulos):
/// - número de itens;
/// - valor total;
/// - valor máximo do item;
/// - metadata do item de maior valor da chave.
#[derive(Debug, Default)]
pub struct DocSummary {
    pub num_de_itens: usize,
    pub item_valor_total: f64,
    pub item_valor_maximo: f64,
    pub metadata: Option<DocMetadata>,
}

impl DocSummary {
    /// Combina dois resumos. Usado para unir os resultados de diferentes threads (Rayon).
    pub fn merge(&mut self, other: Self) {
        self.num_de_itens += other.num_de_itens;
        self.item_valor_total += other.item_valor_total;

        // Se o valor do outro resumo for estritamente maior,
        // substituímos o metadado vencedor.
        if other.item_valor_maximo > self.item_valor_maximo {
            self.item_valor_maximo = other.item_valor_maximo;
            self.metadata = other.metadata;
        }
    }
}

/// Estrutura auxiliar para acumular os dois mapas durante o processamento paralelo.
#[derive(Default)]
pub struct SummaryPair {
    pub ctes: HashMap<Chave, DocSummary>,
    pub nfes: HashMap<Chave, DocSummary>,
}

impl SummaryPair {
    /// Mescla dois pares de resumos consumindo o segundo e fundindo-o no primeiro.
    /// Utiliza a Entry API para evitar buscas duplas no mapa.
    pub fn merge(mut self, other: Self) -> Self {
        // Mesclar o mapa de CT-es
        for (k, v) in other.ctes {
            match self.ctes.entry(k) {
                Entry::Occupied(mut entry) => entry.get_mut().merge(v),
                Entry::Vacant(entry) => {
                    entry.insert(v);
                }
            }
        }
        // Mesclar o mapa de NF-es
        for (k, v) in other.nfes {
            match self.nfes.entry(k) {
                Entry::Occupied(mut entry) => entry.get_mut().merge(v),
                Entry::Vacant(entry) => {
                    entry.insert(v);
                }
            }
        }
        self
    }
}

/// Reter informações (DocSummary) do item de valor máximo da chave (NF-e ou CT-e).
///
/// Uso de Processamento em Paralelo.
pub fn get_summaries_parallel(
    path: &Path,
    config: &Config,
) -> SpedResult<(HashMap<Chave, DocSummary>, HashMap<Chave, DocSummary>)> {
    // 1. Abertura do arquivo com tratamento de erro de I/O
    let file = File::open(path).map_err(|e| SpedError::IoReader {
        source: e,
        arquivo: path.to_path_buf(),
    })?;

    // 2. Configuração do Reader CSV
    // Buffer de 4MB para reduzir syscalls de leitura
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true) // O crate gerencia o cabeçalho automaticamente
        .flexible(false) // Garante integridade (erro se o num de colunas variar)
        .trim(csv::Trim::All) // Remove espaços nas extremidades
        .quoting(true)
        .double_quote(true)
        .buffer_capacity(BUFFER) // Buffer de 4MB para performance
        .from_reader(BufReader::new(file));

    // 3. Processamento Paralelo (Rayon Pipeline)
    let final_pair = rdr
        .byte_records() // Usando ByteRecords para velocidade
        .par_bridge() // Transforma o iterador sequencial em ParallelIterator
        .try_fold(
            SummaryPair::default, // Inicializador local por thread
            |mut acc, result| -> SpedResult<SummaryPair> {
                let record: ByteRecord = result.map_err(SpedError::Csv)?;

                // Deserialização com captura detalhada de erro
                let mut row: Colunas =
                    record
                        .deserialize(None)
                        .map_err(|e| SpedError::CsvDetailed {
                            arquivo: path.to_path_buf(),
                            linha_numero: record.position().map(|p| p.line()).unwrap_or(0),
                            conteudo: record
                                .iter()
                                .map(|b| String::from_utf8_lossy(b))
                                .collect::<Vec<_>>()
                                .join(";"),
                            erro: e.to_string(),
                        })?;

                // Aplicação de Filtro de Notas Canceladas
                if row.chave_cancelada() {
                    return Ok(acc);
                }

                // Valor do Item (f64 parseado) >= DELTA
                let valor = match row.get_valor_do_item() {
                    Some(v) if v.abs() >= DELTA => v.abs(),
                    _ => return Ok(acc), // Ignora ruído
                };

                // Obtenção da chave de 44 dígitos (Chave já é um tipo forte)
                let chave = row.chave;

                // Decide em qual mapa usar com base no tipo da chave
                let map = match (chave.is_cte(), chave.is_nfe()) {
                    (true, _) => &mut acc.ctes,
                    (_, true) => &mut acc.nfes,
                    // Ignora se não for documento de interesse (Modelo 55 ou 57)
                    // Pula para a próxima linha do CSV se não for nenhum dos dois
                    _ => return Ok(acc),
                };

                let doc_summary = map.entry(chave).or_default();

                // Acumulação do valor total
                doc_summary.item_valor_total += valor;

                // Contador de itens
                doc_summary.num_de_itens += 1;

                // Lógica de seleção do item de valor máximo
                // Atualiza se:
                // a) For o primeiro item encontrado (is_none)
                // b) OU (o item atual tem valor estritamente maior que o máximo anterior)
                if doc_summary.metadata.is_none() || valor > doc_summary.item_valor_maximo {
                    doc_summary.item_valor_maximo = valor;

                    // Sanitização Lazy: Limpa apenas o que vai ser guardado na RAM
                    if chave.is_nfe() {
                        Colunas::sanitizar_campo(&mut row.descricao_mercadoria);

                        // Guarda apenas os 10 campos da NF-e, descartando o resto da linha
                        doc_summary.metadata =
                            Some(DocMetadata::Nfe(Box::new(row.extrair_nfe_metadata())));
                    } else {
                        Colunas::sanitizar_campo(&mut row.descricao_natureza);
                        Colunas::sanitizar_campo(&mut row.observacoes_gerais);

                        // Guarda apenas os 16 campos do CT-e, descartando o resto da linha
                        doc_summary.metadata =
                            Some(DocMetadata::Cte(Box::new(row.extrair_cte_metadata())));
                    }
                }

                Ok(acc)
            },
        )
        // 4. Redução: Combina os SummaryPair de todas as threads em um único resultado
        .try_reduce(SummaryPair::default, |a, b| Ok(a.merge(b)))?;

    // 5. Logs e Estatísticas (se verbose estiver ativado)
    if config.verbose {
        println!("--- Resumo do Processamento Paralelo ---");
        println!(
            " -> CT-es Processados: {}",
            fmt_milhares(final_pair.ctes.len())
        );
        println!(
            " -> NF-es Processadas: {}",
            fmt_milhares(final_pair.nfes.len())
        );
    }

    // Retorna a tupla de mapas
    Ok((final_pair.ctes, final_pair.nfes))
}

/// Reter informações (DocSummary) do item de valor máximo da chave (NF-e ou CT-e).
///
/// - O arquivo é lido.
/// - Linhas inválidas/canceladas são puladas.
/// - Os dados são bifurcados em dois destinos.
/// - O "melhor" item (valor máximo) é preservado.
pub fn get_summaries(
    path: &Path,
    config: &Config,
) -> SpedResult<(HashMap<Chave, DocSummary>, HashMap<Chave, DocSummary>)> {
    let mut cte_summaries: HashMap<Chave, DocSummary> = HashMap::new();
    let mut nfe_summaries: HashMap<Chave, DocSummary> = HashMap::new();

    let file = File::open(path).map_err(|e| SpedError::IoReader {
        source: e,
        arquivo: path.to_path_buf(),
    })?;

    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true) // O crate gerencia o cabeçalho automaticamente
        .flexible(false) // Garante integridade (erro se o num de colunas variar)
        .trim(csv::Trim::All) // Remove espaços nas extremidades
        .quoting(true)
        .double_quote(true)
        .buffer_capacity(BUFFER)
        .from_reader(BufReader::new(file));

    // Usamos records() em vez de deserialize() para ter acesso à linha bruta em caso de erro
    // for result in rdr.deserialize::<Colunas>() {
    // let mut row: Colunas = result?;
    for result in rdr.records() {
        let record = result.map_err(SpedError::Csv)?;

        // Deserialização com captura detalhada de erro
        let mut row: Colunas = record
            .deserialize(None)
            .map_err(|e| SpedError::CsvDetailed {
                arquivo: path.to_path_buf(),
                linha_numero: record.position().map(|p| p.line()).unwrap_or(0),
                conteudo: record.iter().collect::<Vec<_>>().join(";"),
                erro: e.to_string(),
            })?;

        // Aplicação de Filtro de Notas Canceladas
        if row.chave_cancelada() {
            continue;
        }

        // Valor do Item (f64 parseado) >= DELTA
        let valor = match row.get_valor_do_item() {
            Some(v) if v.abs() >= DELTA => v.abs(),
            _ => continue, // Ignora ruído
        };

        // Obtenção da chave de 44 dígitos (Chave já é um tipo forte)
        let chave = row.chave;

        // Decide em qual mapa usar com base no tipo da chave
        let map = match (chave.is_cte(), chave.is_nfe()) {
            (true, _) => &mut cte_summaries,
            (_, true) => &mut nfe_summaries,
            _ => continue, // Pula para a próxima linha do CSV se não for nenhum dos dois
        };

        let doc_summary = map.entry(chave).or_default();

        // Acumulação do valor total
        doc_summary.item_valor_total += valor;

        // Contador de itens
        doc_summary.num_de_itens += 1;

        // Lógica de seleção do item de valor máximo
        // Atualiza se:
        // a) For o primeiro item encontrado (is_none)
        // b) OU (o item atual tem valor estritamente maior que o máximo anterior)
        if doc_summary.metadata.is_none() || valor > doc_summary.item_valor_maximo {
            doc_summary.item_valor_maximo = valor;

            // Sanitização Lazy: Limpa apenas o que vai ser guardado na RAM
            if chave.is_nfe() {
                Colunas::sanitizar_campo(&mut row.descricao_mercadoria);

                // Guarda apenas os 10 campos da NF-e, descartando o resto da linha
                doc_summary.metadata = Some(DocMetadata::Nfe(Box::new(row.extrair_nfe_metadata())));
            } else {
                Colunas::sanitizar_campo(&mut row.descricao_natureza);
                Colunas::sanitizar_campo(&mut row.observacoes_gerais);

                // Guarda apenas os 16 campos do CT-e, descartando o resto da linha
                doc_summary.metadata = Some(DocMetadata::Cte(Box::new(row.extrair_cte_metadata())));
            }
        }
    }

    if config.verbose {
        println!(
            " -> CT-es Processados: {}",
            fmt_milhares(cte_summaries.len()),
        );
        println!(
            " -> NF-es Processadas: {}",
            fmt_milhares(nfe_summaries.len()),
        );
    }

    Ok((cte_summaries, nfe_summaries))
}

/// Adiciona informações de CT-es relacionados diretamente na struct Colunas da NF-e.
///
/// Abordagem: Type-safe, extraindo metadados específicos do enum DocMetadata.
pub fn adicionar_info_de_ctes_em_nfe(
    row_nfe: &mut Colunas,
    config: &Config,
    info: &Informacoes,
    cte_info: &HashMap<Chave, DocSummary>,
) -> bool {
    // 1. A chave da NFe é obtida da própria struct
    let chave_nfe = row_nfe.chave;

    // 2. Busca os CT-es relacionados à esta NF-e no índice de transitividade
    let Some(ctes_relacionados) = info.nfe_ctes.get(&chave_nfe) else {
        // row_nfe.chave_de_acesso = format!("NFe: {}, 0 CTe: [] de valor total = 0", chave_nfe).into();
        return false; // Não houve alteração
    };

    // 3. Filtra CT-es que possuem resumo e mapeia para referências
    let mut valid_ctes: Vec<(&Chave, &DocSummary)> = ctes_relacionados
        .iter()
        .filter_map(|c| cte_info.get(c).map(|info| (c, info)))
        .collect();

    if valid_ctes.is_empty() {
        return false; // Não houve alteração
    }

    // 4. Ordenação:
    // 1º Valor Máximo (Desc), 2º Valor Total (Desc), 3º Chave (Asc)
    valid_ctes.sort_unstable_by(|a, b| {
        b.1.item_valor_maximo
            .partial_cmp(&a.1.item_valor_maximo)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.1.item_valor_total
                    .partial_cmp(&a.1.item_valor_total)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.0.cmp(b.0))
    });

    // 5. Formatação da string de resumo para a coluna "Chave de Acesso"
    let soma_total: f64 = valid_ctes.iter().map(|c| c.1.item_valor_total).sum();
    let lista_chaves = valid_ctes
        .iter()
        .map(|c| c.0.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let num_ctes = valid_ctes.len();
    let plural = if num_ctes > 1 { "s" } else { "" };

    // 6. Atualização do campo "chave_de_acesso" diretamente na struct
    row_nfe.chave_de_acesso = format!(
        "NFe: {}, {} CTe{}: [{}] de valor total = {}",
        chave_nfe,
        num_ctes,
        plural,
        lista_chaves,
        f64_to_str(soma_total)
    )
    .into();

    // 7. Injeção de Metadados (As 16 colunas do CT-e injetadas na NF-e)
    // take(config.max_info) limita a quantidade de documentos cujos dados serão concatenados
    for (_, summary) in valid_ctes.iter().take(config.max_info) {
        // Pattern match para garantir que estamos extraindo metadados de CT-e
        if let Some(DocMetadata::Cte(c)) = &summary.metadata {
            row_nfe.injetar_metadata_cte(config, c);
        }
    }

    true
}

/// Adiciona informações de NF-es relacionadas em um CT-e seguindo estritamente
/// as 10 colunas de interesse na ordem definida.
pub fn adicionar_info_de_nfes_em_cte(
    row_cte: &mut Colunas,
    config: &Config,
    info: &Informacoes,
    nfe_info: &HashMap<Chave, DocSummary>,
) -> bool {
    let chave_cte = row_cte.chave;

    // 1. Busca as NF-es relacionadas a este CT-e no índice de transitividade
    let Some(nfes_relacionadas) = info.cte_nfes.get(&chave_cte) else {
        return false; // Não houve alteração
    };

    // 2. Filtra NF-es que possuem resumo válido e mapeia para referências
    let mut valid_nfes: Vec<(&Chave, &DocSummary)> = nfes_relacionadas
        .iter()
        .filter_map(|n| nfe_info.get(n).map(|info| (n, info)))
        .collect();

    if valid_nfes.is_empty() {
        return false; // Não houve alteração
    }

    // 3. Ordenação (Paridade com Perl):
    // 1º Valor Máximo (Desc), 2º Valor Total (Desc), 3º Chave (Asc)
    valid_nfes.sort_unstable_by(|a, b| {
        b.1.item_valor_maximo
            .partial_cmp(&a.1.item_valor_maximo)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.1.item_valor_total
                    .partial_cmp(&a.1.item_valor_total)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.0.cmp(b.0))
    });

    // 4. Atualiza o cabeçalho da célula "Chave de Acesso"
    let soma_total: f64 = valid_nfes.iter().map(|n| n.1.item_valor_total).sum();
    let lista_chaves = valid_nfes
        .iter()
        .map(|n| n.0.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let num_nfes = valid_nfes.len();
    let plural = if num_nfes > 1 { "s" } else { "" };

    row_cte.chave_de_acesso = format!(
        "CTe: {}, {} NFe{}: [{}] de valor total = {}",
        chave_cte,
        num_nfes,
        plural,
        lista_chaves,
        f64_to_str(soma_total)
    )
    .into();

    // 5. Injeção dos metadados das NF-es (10 colunas específicas)
    for (_, summary) in valid_nfes.iter().take(config.max_info) {
        // Pattern match para extrair especificamente os metadados de NF-e
        if let Some(DocMetadata::Nfe(n)) = &summary.metadata {
            row_cte.injetar_metadata_nfe(config, n);
        }
    }

    true
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output colunas_tests
#[cfg(test)]
#[path = "tests/info_adicionadas.rs"]
mod info_adicionadas;
