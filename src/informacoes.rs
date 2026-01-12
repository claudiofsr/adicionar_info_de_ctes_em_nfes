use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{Chave, KeyMap, SpedError, SpedResult, fmt_milhares};

// O estado (os HashMaps) deve ser uma struct separada ou variáveis no main
#[derive(Debug, Default)]
pub struct Informacoes {
    pub nfe_ctes: HashMap<Chave, HashSet<Chave>>,
    pub cte_nfes: HashMap<Chave, HashSet<Chave>>,
    pub cte_complementar: HashMap<Chave, HashSet<Chave>>,
    pub numero_total_de_linhas: usize,
}

impl Informacoes {
    /// Carrega as tabelas de relacionamento em paralelo e processa a transitividade.
    pub fn from_files<P>(path_cte_nfes: P, path_complementares: P) -> SpedResult<Self>
    where
        P: AsRef<Path> + Display + Clone + Send + Sync + 'static,
    {
        println!("--- Carregando Tabelas de Relacionamento ---");

        // 1. Carregamento inicial (IO)
        // Use join do rayon para carregar os dois arquivos em paralelo!
        // rayon::join executa as duas closures em threads diferentes.
        // Capturamos os dois resultados.
        let (cte_nfes, cte_complementar) = {
            let (res1, res2) = rayon::join(
                || Self::ler_todas_as_nfes_deste_cte(path_cte_nfes),
                || Self::ler_chave_complementar_deste_cte(path_complementares),
            );
            (res1?, res2?)
        };

        let mut info = Self {
            cte_nfes,
            cte_complementar,
            ..Default::default()
        };

        // 2. Expansão das relações (Transitividade)
        info.expandir_cte_complementar();

        // 3. Propagação de NFes para CTes complementares
        info.propagar_nfes_para_cte_complementares();

        // 4. Geração do índice invertido (NFe -> CTes)
        info.get_nfe_ctes();

        println!(
            " -> Relações NFe -> CTes carregadas: {}",
            fmt_milhares(info.nfe_ctes.len())
        );
        println!(
            " -> Relações CTe -> NFes carregadas: {}",
            fmt_milhares(info.cte_nfes.len())
        );

        Ok(info)
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
                // Coleta todas as chaves da linha
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
            // Combina os resultados, se houver o mesmo CT-e em linhas diferentes.
            .fold(HashMap::new, |mut acc: KeyMap, (cte, nfes)| {
                acc.entry(cte).or_default().extend(nfes);
                acc
            })
            .reduce(HashMap::new, |mut map1, map2| {
                for (k, v) in map2 {
                    map1.entry(k).or_default().extend(v);
                }
                map1
            });

        Self::print_log("CTe -> NFes", &hash, path);
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

        Self::print_log("CTe <-> CTe Complementar", &hash, path);
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
    /// use adicionar_info_de_ctes_em_nfes::{Informacoes, KeyMap, Chave};
    /// use std::collections::HashMap;
    ///
    /// // Criando chaves de exemplo (44 dígitos numéricos)
    /// let c1 = Chave::new("11111111111111111111571111111111111111111111").unwrap();
    /// let c2 = Chave::new("22222222222222222222572222222222222222222222").unwrap();
    /// let c3 = Chave::new("33333333333333333333573333333333333333333333").unwrap();
    ///
    /// let mut info = Informacoes::default();
    ///
    /// // Simula A refere B, B refere C
    ///
    /// // CTe 1 referencia o 2
    /// info.cte_complementar.entry(c1).or_default().insert(c2);
    /// // CTe 2 referencia o 3
    /// info.cte_complementar.entry(c2).or_default().insert(c3);
    ///
    /// info.expandir_cte_complementar();
    ///
    /// // Graças à transitividade e simetria:
    /// // 1 agora conhece 3 e 3 agora conhece 1
    /// assert!(info.cte_complementar.get(&c1).unwrap().contains(&c3));
    /// assert!(info.cte_complementar.get(&c3).unwrap().contains(&c1));
    ///
    /// // Todos conhecem todos (exceto a si mesmos)
    /// assert_eq!(info.cte_complementar.get(&c1).unwrap().len(), 2);
    /// ```
    pub fn expandir_cte_complementar(&mut self) {
        // 1. Simetrização: Criar um grafo de adjacência para garantir bidirecionalidade.
        // Usamos drain() para consumir o mapa original sem alocações extras.
        let mut adj: HashMap<Chave, HashSet<Chave>> = HashMap::new();
        for (u, neighbors) in self.cte_complementar.drain() {
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
                        self.cte_complementar.insert(member, others);
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
    /// use adicionar_info_de_ctes_em_nfes::{Informacoes, KeyMap, Chave};
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
    /// let mut info = Informacoes::default();
    ///
    /// // 2. Configurar relação: CT-e Pai -> possui NF-e
    /// info.cte_nfes.entry(cte_pai).or_default().insert(nfe);
    ///
    /// // 3. Configurar relação: CT-e Pai -> é complementado por CT-e Comp
    /// info.cte_complementar.entry(cte_pai).or_default().insert(cte_comp);
    ///
    /// // 4. Executar a expansão
    /// info.propagar_nfes_para_cte_complementares();
    ///
    /// // 5. O CT-e complementar agora deve possuir a NF-e que era do pai
    /// assert!(info.cte_nfes.get(&cte_comp).expect("CTe complementar deve existir no mapa").contains(&nfe));
    /// ```
    pub fn propagar_nfes_para_cte_complementares(&mut self) {
        // 1. Acumulador temporário para evitar conflitos de empréstimo (borrow checker).
        // Como Chave é Copy, este HashMap é muito denso e rápido.
        let mut updates: HashMap<Chave, HashSet<Chave>> = HashMap::new();

        // 2. Itera sobre os CTes que possuem NFEs
        for (cte, nfes) in self.cte_nfes.iter() {
            // Se este CTe possui complementares associados no grafo de transitividade
            if let Some(complements) = self.cte_complementar.get(cte) {
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
            self.cte_nfes
                .entry(target_cte)
                .or_default()
                .extend(new_nfes);
        }
    }

    pub fn get_nfe_ctes(&mut self) {
        // Limpa o mapa caso a função seja chamada mais de uma vez.
        // Adicionado por segurança defensiva.
        self.nfe_ctes.clear();

        for (&cte, nfes) in &self.cte_nfes {
            for &nfe in nfes {
                self.nfe_ctes.entry(nfe).or_default().insert(cte);
            }
        }
    }

    #[inline]
    fn print_log<P>(label: &str, map: &KeyMap, path: P)
    where
        P: Display,
    {
        let num_de_items = map.values().map(|v| v.len()).sum::<usize>();
        println!(
            "Encontrado {:>6} chaves ({:>6} relações {}) no arquivo <{}>.",
            fmt_milhares(map.len()),
            fmt_milhares(num_de_items),
            label,
            path
        );
    }
}
