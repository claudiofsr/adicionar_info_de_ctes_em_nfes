use crate::{
    Chave, SpedResult, expand_cte_complementar, expand_cte_nfes, fmt_milhares, get_nfe_ctes,
    ler_chave_complementar_deste_cte, ler_todas_as_nfes_deste_cte,
};
use std::collections::{HashMap, HashSet};

// O estado (os HashMaps) deve ser uma struct separada ou variáveis no main
pub struct Informacoes {
    pub nfe_ctes: HashMap<Chave, HashSet<Chave>>,
    pub cte_nfes: HashMap<Chave, HashSet<Chave>>,
    pub cte_complementar: HashMap<Chave, HashSet<Chave>>,
}

impl Informacoes {
    /// Construtor que carrega, expande e processa todas as relações de documentos.
    pub fn carregar<P: AsRef<std::path::Path> + std::fmt::Display + Clone>(
        path_cte_nfes: P,
        path_complementares: P,
    ) -> SpedResult<Self> {
        println!("--- Carregando Tabelas de Relacionamento ---");

        // 1. Carregamento inicial (IO)
        let mut cte_nfes = ler_todas_as_nfes_deste_cte(path_cte_nfes)?;
        let mut cte_complementar = ler_chave_complementar_deste_cte(path_complementares)?;

        // 2. Expansão das relações (Transitividade)
        expand_cte_complementar(&mut cte_complementar);

        // 3. Propagação de NFes para CTes complementares
        expand_cte_nfes(&mut cte_nfes, &cte_complementar);

        // 4. Geração do índice invertido (NFe -> CTes)
        let nfe_ctes = get_nfe_ctes(&cte_nfes);

        println!(
            " -> Relações NFe -> CTes carregadas: {}",
            fmt_milhares(nfe_ctes.len())
        );
        println!(
            " -> Relações CTe -> NFes carregadas: {}",
            fmt_milhares(cte_nfes.len())
        );

        Ok(Self {
            nfe_ctes,
            cte_nfes,
            cte_complementar,
        })
    }
}
