use super::*;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

// Helper para criar uma Chave válida rapidamente
fn mock_chave(prefixo: &str) -> Chave {
    let s = format!("{:0<44}", prefixo);
    Chave::new(&s).expect("Falha ao criar chave de teste")
}

fn mock_config_padrao() -> Config {
    Config {
        atualizar_origem: false,
        clear: false,
        doc_path: PathBuf::new(),
        exibir_config: false,
        max_char: 1000,
        max_info: 10,
        no_prompt: false,
        verbose: false,
    }
}

// Helper para criar uma struct Colunas padrão (Owned/'static)
fn mock_colunas(chave: Chave) -> Colunas<'static> {
    Colunas {
        chave,
        contribuinte_cnpj: "".into(),
        contribuinte_nome: "".into(),
        entrada_ou_saida: "".into(),
        participante_cnpj: "".into(),
        participante_nome: "".into(),
        regime_tributario: "".into(),
        observacoes: "".into(),
        remetente_cnpj1: "".into(),
        remetente_cnpj2: "".into(),
        remetente_nome: "".into(),
        remetente_municipio: "".into(),
        tomador_papel1: "".into(),
        tomador_papel2: "".into(),
        tomador_cnpj1: "".into(),
        tomador_cnpj2: "".into(),
        inicio_estado: "".into(),
        inicio_municipio: "".into(),
        termino_estado: "".into(),
        termino_municipio: "".into(),
        destinatario_cnpj: "".into(),
        destinatario_nome: "".into(),
        local_entrega: "".into(),
        descricao_natureza: "".into(),
        cancelada: "Não".into(),
        origem: "".into(),
        natureza_bc: "".into(),
        modelo: "".into(),
        num_doc: "".into(),
        chave_de_acesso: "".into(),
        observacoes_gerais: "".into(),
        dia_emissao: "".into(),
        numero_di: "".into(),
        numero_item: "".into(),
        cfop: "".into(),
        descricao_cfop: "".into(),
        descricao_mercadoria: "".into(),
        ncm: "00000000".into(),
        descricao_ncm: "".into(),
        aliq_cofins: "".into(),
        aliq_pis: "".into(),
        cst_descricao_cofins: "".into(),
        cst_descricao_pis: "".into(),
        valor_total: "0".into(),
        valor_item: "0".into(),
        valor_desconto: "0".into(),
        valor_seguro: "0".into(),
        valor_cofins: "0".into(),
        valor_pis: "0".into(),
        valor_ipi: "0".into(),
        valor_bc_iss: "0".into(),
        valor_iss: "0".into(),
        aliq_icms: "0".into(),
        valor_bc_icms: "0".into(),
        valor_icms: "0".into(),
        valor_icms_sub: "0".into(),
    }
}

#[test]
fn teste_enriquecimento_nfe_com_cte() {
    let config = Config {
        doc_path: PathBuf::from("teste.csv"),
        max_info: 2,
        ..mock_config_padrao()
    };

    let chave_nfe = mock_chave("1111111111111111111155");
    let chave_cte = mock_chave("2222222222222222222257");

    let mut nfe_ctes = HashMap::new();
    let mut ctes = HashSet::new();
    ctes.insert(chave_cte);
    nfe_ctes.insert(chave_nfe, ctes);

    let info = Informacoes {
        nfe_ctes,
        cte_nfes: HashMap::new(),
        cte_complementar: HashMap::new(),
    };

    let mut cte_resumo_map = HashMap::new();
    let mut colunas_cte = mock_colunas(chave_cte);

    // Atribuição direta funciona poisliterais string são convertidos para Cow
    colunas_cte.remetente_cnpj1 = "12.345.678/0001-99".into();
    colunas_cte.inicio_municipio = "SÃO PAULO".into();

    cte_resumo_map.insert(
        chave_cte,
        DocSummary {
            num_de_itens: 1,
            item_valor_total: 500.0,
            item_valor_maximo: 500.0,
            // colunas_max espera Box<Colunas<'static>>, que é o que mock_colunas retorna
            colunas_max: Some(Box::new(colunas_cte)),
        },
    );

    let mut row_nfe = mock_colunas(chave_nfe);
    adicionar_info_de_ctes_em_nfe(&mut row_nfe, &config, &info, &cte_resumo_map);

    assert!(row_nfe.chave_de_acesso.contains("1 CTe"));
    assert!(row_nfe.chave_de_acesso.contains("500.00"));
    assert!(row_nfe.remetente_cnpj1.contains("12.345.678/0001-99"));
    assert!(row_nfe.inicio_municipio.contains("SÃO PAULO"));
}

#[test]
fn teste_sobreposicao_ncm_no_cte() {
    let config = Config {
        max_info: 1,
        ..mock_config_padrao()
    };

    let chave_cte = mock_chave("2222222222222222222257");
    let chave_nfe = mock_chave("1111111111111111111155");

    let mut cte_nfes = HashMap::new();
    let mut nfes = HashSet::new();
    nfes.insert(chave_nfe);
    cte_nfes.insert(chave_cte, nfes);

    let info = Informacoes {
        nfe_ctes: HashMap::new(),
        cte_nfes,
        cte_complementar: HashMap::new(),
    };

    let mut colunas_nfe = mock_colunas(chave_nfe);
    colunas_nfe.ncm = "84713012".into();
    colunas_nfe.contribuinte_nome = "FORNECEDOR LTDA".into();

    let mut nfe_resumo_map = HashMap::new();
    nfe_resumo_map.insert(
        chave_nfe,
        DocSummary {
            num_de_itens: 1,
            item_valor_total: 1000.0,
            item_valor_maximo: 1000.0,
            colunas_max: Some(Box::new(colunas_nfe)),
        },
    );

    let mut row_cte = mock_colunas(chave_cte);
    row_cte.ncm = "00000000".into();

    adicionar_info_de_nfes_em_cte(&mut row_cte, &config, &info, &nfe_resumo_map);

    // Verificação: O NCM do CT-e foi sobrescrito pelo NCM válido da NF-e
    assert_eq!(row_cte.ncm, "84713012");
    assert!(row_cte.contribuinte_nome.contains("FORNECEDOR LTDA"));
}
