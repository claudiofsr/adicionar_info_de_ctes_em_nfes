use super::*;
use std::collections::{HashMap, HashSet};

// Helper para criar uma Chave válida rapidamente
fn mock_chave(prefixo: &str) -> Chave {
    let s = format!("{:0<44}", prefixo);
    Chave::new(&s).expect("Falha ao criar chave de teste")
}

// Config padrão simplificada com Default
fn mock_config_padrao() -> Config {
    Config {
        max_char: 1000,
        max_info: 10,
        ..Default::default()
    }
}

// Helper para criar uma struct Colunas plana básica
fn mock_colunas(chave: Chave) -> Colunas<'static> {
    Colunas {
        chave,
        cancelada: "Não".into(),
        ncm: "00000000".into(),
        ..Default::default()
    }
}

#[test]
fn teste_enriquecimento_nfe_com_cte() {
    let config = Config {
        max_info: 2,
        ..mock_config_padrao()
    };

    let chave_nfe = mock_chave("1111111111111111111155");
    let chave_cte = mock_chave("2222222222222222222257");

    // 1. Configura as relações (Índice de transitividade)
    let mut nfe_ctes = HashMap::new();
    let mut ctes = HashSet::new();
    ctes.insert(chave_cte);
    nfe_ctes.insert(chave_nfe, ctes);

    let info = Informacoes {
        nfe_ctes,
        ..Default::default()
    };

    // 2. Prepara os metadados do CT-e (O documento que fornece info)
    let colunas_cte = Colunas {
        remetente_cnpj1: "12.345.678/0001-99".into(),
        inicio_municipio: "SÃO PAULO".into(),
        ..mock_colunas(chave_cte)
    };

    let mut cte_resumo_map = HashMap::new();
    cte_resumo_map.insert(
        chave_cte,
        DocSummary {
            num_de_itens: 1,
            item_valor_total: 500.0,
            item_valor_maximo: 500.0,
            metadata: Some(DocMetadata::Cte(Box::new(
                colunas_cte.extrair_cte_metadata(),
            ))),
        },
    );

    // 3. Executa o enriquecimento na NF-e
    let mut row_nfe = mock_colunas(chave_nfe);
    adicionar_info_de_ctes_em_nfe(&mut row_nfe, &config, &info, &cte_resumo_map);

    // 4. Asserções
    assert!(row_nfe.chave_de_acesso.contains("1 CTe"));
    assert!(row_nfe.chave_de_acesso.contains("500.00"));
    assert!(row_nfe.remetente_cnpj1.contains("12.345.678/0001-99"));
    assert!(row_nfe.inicio_municipio.contains("SÃO PAULO"));
}

#[test]
fn teste_sobreposicao_ncm_no_cte() {
    let config = mock_config_padrao();

    let chave_cte = mock_chave("2222222222222222222257");
    let chave_nfe = mock_chave("1111111111111111111155");

    // 1. Configura as relações
    let mut cte_nfes = HashMap::new();
    let mut nfes = HashSet::new();
    nfes.insert(chave_nfe);
    cte_nfes.insert(chave_cte, nfes);

    let info = Informacoes {
        cte_nfes,
        ..Default::default()
    };

    // 2. Prepara os metadados da NF-e (O documento que fornece o NCM)
    let colunas_nfe = Colunas {
        ncm: "84713012".into(),
        contribuinte_nome: "FORNECEDOR LTDA".into(),
        ..mock_colunas(chave_nfe)
    };

    let mut nfe_resumo_map = HashMap::new();
    nfe_resumo_map.insert(
        chave_nfe,
        DocSummary {
            num_de_itens: 1,
            item_valor_total: 1000.0,
            item_valor_maximo: 1000.0,
            metadata: Some(DocMetadata::Nfe(Box::new(
                colunas_nfe.extrair_nfe_metadata(),
            ))),
        },
    );

    // 3. Executa o enriquecimento no CT-e
    let mut row_cte = mock_colunas(chave_cte);
    row_cte.ncm = "00000000".into(); // NCM inicial "inválido"

    adicionar_info_de_nfes_em_cte(&mut row_cte, &config, &info, &nfe_resumo_map);

    // 4. Verificação: O NCM do CT-e foi sobrescrito pelo NCM válido da NF-e
    assert_eq!(row_cte.ncm, "84713012");
    assert!(row_cte.contribuinte_nome.contains("FORNECEDOR LTDA"));
}
