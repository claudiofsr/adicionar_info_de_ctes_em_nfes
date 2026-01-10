use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    Chave, Colunas, Config, DocSummary, Informacoes, SpedError, SpedResult,
    adicionar_info_de_ctes_em_nfe, adicionar_info_de_nfes_em_cte,
};

/// Tipo alias para representar o mapa de relações entre chaves de CTe.
pub type KeyMap = HashMap<Chave, HashSet<Chave>>;

/// Limpar a tela.
pub fn clear_screen(clear_screen: bool) -> SpedResult<()> {
    if clear_screen {
        if cfg!(target_os = "windows") {
            // No Windows, 'cls' é um comando interno do 'cmd'.
            // Precisamos chamar o interpretador para executá-lo.
            Command::new("cmd").args(["/c", "cls"]).status()?;
        } else {
            // Em Linux/macOS, o comando 'clear' costuma ser um executável independente.
            Command::new("clear").status()?;
        }
    }

    Ok(())
}

/// Exibe a descrição, autoria e versão do programa.
/// Equivalente a Imprimir_Versao_do_Programa em Perl.
pub fn imprimir_versao_do_programa() {
    let descr = [
        "Este programa adiciona informações de CTes em NFes (e vice-versa).",
        "Chaves dispostas em ordem decrescente dos valores na coluna <Chave de acesso>.",
        "As opções seguintes podem ser alteradas/adicionadas na linha de comando:\n",
        " --max_char: máximo de caracter por coluna (default: 3000)",
        " --max_info: máximo de informações de docs fiscais adicionado (default: 10)",
    ];

    let author = "Claudio Fernandes de Souza Rodrigues (claudiofsr@yahoo.com)";
    let date = "8 de Janeiro de 2026 (inicio: 15 de Agosto de 2021)";
    let version = "0.51";

    // Loop de impressão da descrição (semelhante ao foreach do Perl)
    for line in &descr {
        println!(" {}", line);
    }

    // Impressão do rodapé utilizando interpolação de strings
    println!("\n {}\n {}\n versão: {}\n", author, date, version);
}

pub fn ler_todas_as_nfes_deste_cte<P>(path: P) -> SpedResult<KeyMap>
where
    P: AsRef<Path> + Clone + Display,
{
    let file = File::open(&path).map_err(|e| SpedError::IoReader {
        source: e,
        arquivo: path.as_ref().to_path_buf(),
    })?;

    let reader = BufReader::new(file);

    // Compila o regex apenas uma vez.
    // \b garante que pegamos apenas sequências de 44 dígitos isoladas.
    let re = Regex::new(r"\b\d{44}\b")?;

    let hash: KeyMap = reader
        .lines()
        .par_bridge() // Transforma o iterador sequencial em paralelo
        .filter_map(|line_result| {
            let line = line_result.ok()?;
            let mut chaves = re.find_iter(&line).filter_map(|m| Chave::new(m.as_str()));

            let cte = chaves.next()?;
            if !cte.is_cte() {
                return None;
            }

            let nfes: HashSet<Chave> = chaves.filter(|c| c.is_nfe()).collect();

            if nfes.is_empty() {
                None
            } else {
                Some((cte, nfes))
            }
        })
        .collect();

    // Estatísticas usando funcional
    let num_cte = hash.len();
    let num_nfe = hash.values().map(|v| v.len()).sum::<usize>();

    println!(
        "Encontrado {:>6} CTes contendo no total {:>6} NFes no arquivo <{}>.",
        fmt_milhares(num_cte),
        fmt_milhares(num_nfe),
        path
    );

    Ok(hash)
}

pub fn ler_chave_complementar_deste_cte<P>(path: P) -> SpedResult<KeyMap>
where
    P: AsRef<Path> + Clone + std::fmt::Display,
{
    let file = File::open(&path).map_err(|e| SpedError::IoReader {
        source: e,
        arquivo: path.as_ref().to_path_buf(),
    })?;

    let reader = BufReader::new(file);
    let re = Regex::new(r"\b\d{44}\b")?;

    // Utilizamos try_fold para construir sub-mapas em cada thread
    // e try_reduce para mesclá-los de forma eficiente.
    let hash: KeyMap = reader
        .lines()
        .par_bridge() // Paraleliza o iterador de linhas
        .try_fold(
            HashMap::new,
            |mut acc: KeyMap, line_result| -> SpedResult<KeyMap> {
                let line = line_result?;

                // Extrai chaves e converte para a struct Chave (ignora as inválidas)
                let mut matches = re.find_iter(&line).filter_map(|m| Chave::new(m.as_str()));

                // Esperamos pelo menos duas chaves válidas na linha
                if let (Some(cte), Some(comp)) = (matches.next(), matches.next()) {
                    // Regra de negócio: Ambos devem ser CT-e (57) e não podem ser iguais
                    if cte.is_cte() && comp.is_cte() && cte != comp {
                        // Inserção bidirecional: Chave é Copy, então não precisamos de .clone()
                        acc.entry(cte).or_default().insert(comp);
                        acc.entry(comp).or_default().insert(cte);
                    }
                }
                Ok(acc)
            },
        )
        .try_reduce(HashMap::new, |mut map_a, map_b| {
            // Mescla os mapas das threads. extend() em HashSets é otimizado.
            for (key, values) in map_b {
                map_a.entry(key).or_default().extend(values);
            }
            Ok(map_a)
        })?;

    // Estatísticas finais
    let num_cte = hash.len();
    let num_com = hash.values().map(|v| v.len()).sum::<usize>();

    println!(
        "Encontrado {:>6} CTes contendo no total {:>6} CTes Complementares no arquivo <{}>.",
        fmt_milhares(num_cte),
        fmt_milhares(num_com),
        path
    );

    Ok(hash)
}

/// Expande as relações de transitividade entre CTes Complementares.
///
/// Esta função resolve o problema de encontrar "componentes conectados" em um grafo de documentos.
/// Se um CTe **A** referencia **B**, e **B** referencia **C**, a função entende que todos
/// pertencem ao mesmo grupo e atualiza o mapa para que todos apontem para todos.
///
/// ### Lógica de Negócio (Transitividade)
/// Em termos práticos, se houver uma cadeia de complementos (A -> B -> C), o algoritmo
/// garante que o resultado final contenha:
/// - A conhece {B, C}
/// - B conhece {A, C}
/// - C conhece {A, B}
///
/// ### Algoritmo
/// O processo é realizado em três etapas principais:
/// 1. **Simetrização**: Garante que se A aponta para B, B também aponte para A no grafo inicial.
/// 2. **Busca de Componentes**: Utiliza uma Busca em Profundidade (DFS) para agrupar todos os
///    CTes que possuem qualquer ligação entre si (direta ou indireta).
/// 3. **Clique (Expansão Total)**: Para cada grupo encontrado, reconstrói o mapa original
///    onde cada membro do grupo possui como vizinhos todos os outros integrantes.
///
/// ### Performance
/// Esta implementação utiliza a identificação de componentes conectados,
/// resultando em uma complexidade **O(V + E)**, onde:
/// - **V** é o número de chaves (vértices).
/// - **E** é o número de relações (arestas).
///
/// ### Exemplo
/// ```
/// use adicionar_info_de_ctes_em_nfes::{expand_cte_complementar, KeyMap, Chave};
/// use std::collections::HashMap;
///
/// // Criando chaves de exemplo (44 dígitos numéricos)
/// let c1 = Chave::new("11111111111111111111571111111111111111111111").unwrap();
/// let c2 = Chave::new("22222222222222222222572222222222222222222222").unwrap();
/// let c3 = Chave::new("33333333333333333333573333333333333333333333").unwrap();
///
/// let mut mapa: KeyMap = HashMap::new();
/// // CTe 1 referencia o 2
/// mapa.entry(c1).or_default().insert(c2);
/// // CTe 2 referencia o 3
/// mapa.entry(c2).or_default().insert(c3);
///
/// expand_cte_complementar(&mut mapa);
///
/// // Graças à transitividade e simetria:
/// // 1 agora conhece 3 e 3 agora conhece 1
/// assert!(mapa.get(&c1).unwrap().contains(&c3));
/// assert!(mapa.get(&c3).unwrap().contains(&c1));
///
/// // Todos conhecem todos (exceto a si mesmos)
/// assert_eq!(mapa.get(&c1).unwrap().len(), 2);
/// ```
pub fn expand_cte_complementar(map: &mut KeyMap) {
    // 1. Simetrização: Criar um grafo de adjacência para garantir bidirecionalidade.
    // Usamos drain() para consumir o mapa original sem alocações extras.
    let mut adj: HashMap<Chave, HashSet<Chave>> = HashMap::new();
    for (u, neighbors) in map.drain() {
        for v in neighbors {
            // Chave é Copy, então u e v são copiados como valores simples (44 bytes)
            adj.entry(u).or_default().insert(v);
            adj.entry(v).or_default().insert(u);
        }
    }

    let mut visited = HashSet::new();
    // Coleta as chaves do grafo de adjacência; muito rápido com Chave (Copy)
    let keys: Vec<Chave> = adj.keys().copied().collect();

    for node in keys {
        if visited.contains(&node) {
            continue;
        }

        // 2. Identificar todos os membros da "ilha" (componente conectado) via DFS
        let mut group = Vec::new();
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            if visited.insert(current) {
                group.push(current);
                if let Some(neighbors) = adj.get(&current) {
                    // Adiciona vizinhos à pilha
                    stack.extend(neighbors.iter().copied());
                }
            }
        }

        // 3. Criar a relação "clique" (todos com todos) para este grupo
        // Se o grupo só tem 1 elemento, ele não é complementar de ninguém
        if group.len() > 1 {
            let full_group_set: HashSet<Chave> = group.iter().copied().collect();

            for member in group {
                // Criamos o conjunto de "outros" removendo apenas o membro atual
                let mut others = full_group_set.clone();
                others.remove(&member);

                if !others.is_empty() {
                    map.insert(member, others);
                }
            }
        }
    }
}

/// Expande a associação de NFEs para CTes complementares.
///
/// ### Lógica de Negócio
/// No transporte de cargas (SPED), um CTe Complementar herda as notas fiscais (NFEs)
/// do seu CTe de referência. Esta função garante que se o **CTe A** possui as
/// **Notas 1 e 2**, e o **CTe B** é complementar de **A**, então **B** também
/// passará a listar as **Notas 1 e 2**.
///
/// ### Otimização de Performance
/// Diferente da abordagem com `Vec<(String, String)>`, esta versão:
/// 1. Usa um `HashMap<String, HashSet<String>>` temporário para agrupar notas por CTe.
/// 2. Reduz a pressão sobre o alocador de memória ao evitar a criação de milhões de tuplas.
/// 3. Utiliza `extend` para mesclar conjuntos de dados de uma só vez, o que é mais
///    eficiente em Rust do que inserções individuais em loops.
///
/// Se um CTe de origem existe em ambos os mapas,
/// todos os seus "alvos" complementares recebem todas as suas NFEs.
///
/// ### Exemplo
/// ```
/// use adicionar_info_de_ctes_em_nfes::{expand_cte_nfes, KeyMap, Chave};
/// use std::collections::{HashMap, HashSet};
///
/// // 1. Criar chaves válidas (44 dígitos)
/// // CT-e de origem (modelo 57)
/// let cte_pai = Chave::new("11111111111111111111571111111111111111111111").unwrap();
/// // CT-e complementar (modelo 57)
/// let cte_comp = Chave::new("22222222222222222222572222222222222222222222").unwrap();
/// // NF-e vinculada ao CT-e pai (modelo 55)
/// let nfe = Chave::new("33333333333333333333553333333333333333333333").unwrap();
///
/// // 2. Configurar relação: CT-e Pai -> possui NF-e
/// let mut cte_nfes: KeyMap = HashMap::new();
/// cte_nfes.entry(cte_pai).or_default().insert(nfe);
///
/// // 3. Configurar relação: CT-e Pai -> é complementado por CT-e Comp
/// let mut cte_complementar: KeyMap = HashMap::new();
/// cte_complementar.entry(cte_pai).or_default().insert(cte_comp);
///
/// // 4. Executar a expansão
/// expand_cte_nfes(&mut cte_nfes, &cte_complementar);
///
/// // 5. O CT-e complementar agora deve possuir a NF-e que era do pai
/// assert!(cte_nfes.get(&cte_comp).expect("CTe complementar deve existir no mapa").contains(&nfe));
/// ```
pub fn expand_cte_nfes(cte_nfes: &mut KeyMap, cte_complementar: &KeyMap) {
    // 1. Acumulador temporário para evitar conflitos de empréstimo (borrow checker).
    // Como Chave é Copy, este HashMap é muito denso e rápido.
    let mut updates: HashMap<Chave, HashSet<Chave>> = HashMap::new();

    // 2. Itera sobre os CTes que possuem NFEs
    for (cte, nfes) in cte_nfes.iter() {
        // Se este CTe possui complementares associados no grafo de transitividade
        if let Some(complements) = cte_complementar.get(cte) {
            for &comp in complements {
                // Adiciona todas as NFEs do CTe pai ao CTe complementar no acumulador.
                // .copied() transforma o iterador de &Chave em Chave (Copy).
                updates
                    .entry(comp)
                    .or_default()
                    .extend(nfes.iter().copied());
            }
        }
    }

    // 3. Mescla os novos dados acumulados de volta no mapa original.
    for (target_cte, new_nfes) in updates {
        // O uso de 'extend' em um HashSet é otimizado para evitar re-hashing desnecessário.
        cte_nfes.entry(target_cte).or_default().extend(new_nfes);
    }
}

pub fn get_nfe_ctes(cte_nfes: &KeyMap) -> KeyMap {
    let mut nfe_ctes: KeyMap = HashMap::new();
    for (&cte, nfes) in cte_nfes {
        for &nfe in nfes {
            nfe_ctes.entry(nfe).or_default().insert(cte);
        }
    }
    nfe_ctes
}

pub fn fmt_milhares(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + len / 3);

    bytes.iter().enumerate().for_each(|(i, &b)| {
        // Adiciona o ponto se:
        // 1. Não for o primeiro caractere (i > 0)
        // 2. A distância até o fim for múltipla de 3
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push('.');
        }
        result.push(b as char);
    });

    result
}

/// Equivalente ao Sobrescrever_Arquivo do Perl
pub fn sobrescrever_arquivo(original: &Path, alterado: &Path) -> SpedResult<()> {
    if original.exists() && alterado.exists() {
        println!();
        println!("Arquivo Original: '{}'", original.display());
        println!("Arquivo Alterado: '{}'", alterado.display());

        loop {
            println!("\nSobrescrever o Arquivo Original pelo Arquivo Alterado?");
            println!("\t'{}' --> '{}'", alterado.display(), original.display());
            print!("Digite s ou n (sim ou não): ");
            io::stdout().flush()?; // Garante que o print apareça antes do input

            let mut resposta = String::new();
            io::stdin().read_line(&mut resposta)?;
            let resposta = resposta.trim().to_lowercase();

            if resposta == "s" || resposta == "y" {
                println!("\n\tmv '{}' '{}'", alterado.display(), original.display());
                fs::rename(alterado, original)?;
                break;
            } else if resposta == "n" {
                break;
            }
        }
        println!();
    }
    Ok(())
}

/// Processa o enriquecimento do arquivo CSV (Passagem 2).
/// Utiliza a Abordagem 1: Deserialização direta para a struct Colunas.
pub fn enriquecer_arquivo(
    config: &Config,
    info: &mut Informacoes,
    cte_info: &HashMap<Chave, DocSummary>,
    nfe_info: &HashMap<Chave, DocSummary>,
) -> SpedResult<(PathBuf, usize)> {
    println!("--- Passagem 2: Gravando arquivo enriquecido ---");

    let input_path = &config.doc_path;
    let output_path = input_path.with_extension("modificado.csv");

    // 1. Configurar Reader com buffer otimizado
    let file_in = File::open(input_path).map_err(|e| SpedError::IoReader {
        source: e,
        arquivo: input_path.clone(),
    })?;

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .trim(csv::Trim::All)
        .buffer_capacity(1024 * 1024) // 1MB Buffer de leitura
        .from_reader(BufReader::new(file_in));

    // 2. Configurar Writer com buffer otimizado
    let file_out = File::create(&output_path)?;
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .quote_style(csv::QuoteStyle::Necessary)
        .double_quote(true)
        .buffer_capacity(1024 * 1024) // 1MB Buffer de escrita
        .from_writer(BufWriter::new(file_out));

    let mut alteracoes_realizadas = 0;

    // Reutilizamos o buffer do StringRecord para evitar alocações a cada linha
    let mut record = csv::StringRecord::new();

    while rdr.read_record(&mut record)? {
        info.numero_total_de_linhas += 1;

        // Deserialização "Zero-Copy": os campos da struct Colunas aponta para dentro do 'record'
        let mut row: Colunas = record
            .deserialize(None)
            .map_err(|e| SpedError::Config(format!("Falha ao processar linha do CSV: {}", e)))?;

        let mut mudou = false;

        // Notas canceladas são apenas escritas de volta sem alteração
        if !row.chave_cancelada() {
            let chave = row.chave;

            if chave.is_nfe() {
                adicionar_info_de_ctes_em_nfe(&mut row, config, info, cte_info);
                // Se a função adicionar_info mexeu em algum Cow via to_mut(),
                // ele agora é Cow::Owned.
                mudou = true;
            } else if chave.is_cte() {
                adicionar_info_de_nfes_em_cte(&mut row, config, info, nfe_info);
                mudou = true;
            }
        }

        if mudou {
            // Serializa a struct modificada
            wtr.serialize(row)?;
            alteracoes_realizadas += 1;
        } else {
            // Performance Máxima: Escreve o buffer original sem re-serializar
            // Se não mudou nada, escrevemos o buffer original diretamente.
            // Isso evita converter String -> Struct -> String.
            wtr.write_record(&record)?;
        }
    }

    // Garante que tudo foi gravado no disco
    wtr.flush()?;

    println!(
        " -> Total de linhas enriquecidas: {}",
        fmt_milhares(alteracoes_realizadas)
    );

    Ok((output_path, alteracoes_realizadas))
}
