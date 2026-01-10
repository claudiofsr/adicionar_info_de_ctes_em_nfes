use super::*;

// Helper para criar uma struct Colunas mínima para testes
fn mock_colunas_com_valor(valor: &str) -> Colunas<'static> {
    Colunas {
        // .to_string() cria uma String (Owned)
        // .into() converte String para Cow::Owned, que é 'static
        valor_item: valor.to_string().into(),
        ..Default::default()
    }
}
#[test]
fn test_formatos_comuns() {
    // Padrão Brasileiro com ponto de milhar
    assert_eq!(
        mock_colunas_com_valor("1.234,56").get_valor_do_item(),
        Some(1234.56)
    );
    // Padrão Brasileiro sem milhar
    assert_eq!(
        mock_colunas_com_valor("1234,56").get_valor_do_item(),
        Some(1234.56)
    );
    // Padrão Internacional
    assert_eq!(
        mock_colunas_com_valor("1234.56").get_valor_do_item(),
        Some(1234.56)
    );
    // Inteiro
    assert_eq!(
        mock_colunas_com_valor("1000").get_valor_do_item(),
        Some(1000.0)
    );
}

#[test]
fn test_valores_pequenos_e_negativos() {
    assert_eq!(
        mock_colunas_com_valor("0,05").get_valor_do_item(),
        Some(0.05)
    );
    assert_eq!(
        mock_colunas_com_valor("-10,50").get_valor_do_item(),
        Some(-10.5)
    );
    assert_eq!(
        mock_colunas_com_valor("-1.500,00").get_valor_do_item(),
        Some(-1500.0)
    );
}

#[test]
fn test_limpeza_de_ruido() {
    // Espaços e símbolos de moeda
    assert_eq!(
        mock_colunas_com_valor(" R$ 1.234,56 ").get_valor_do_item(),
        Some(1234.56)
    );
    // Texto misturado (comum em campos mal preenchidos)
    assert_eq!(
        mock_colunas_com_valor("valor: 100,00").get_valor_do_item(),
        Some(100.0)
    );
}

#[test]
fn test_casos_vazios_e_invalidos() {
    assert_eq!(mock_colunas_com_valor("").get_valor_do_item(), None);
    assert_eq!(mock_colunas_com_valor("abc").get_valor_do_item(), None);
    assert_eq!(mock_colunas_com_valor("...").get_valor_do_item(), None);
}

#[test]
fn test_limite_do_buffer_64_bytes() {
    // Caso Extremo: Valor dentro do limite (exatamente 64 chars de dígitos)
    let longo_valido = "0".repeat(64);
    assert!(
        mock_colunas_com_valor(&longo_valido)
            .get_valor_do_item()
            .is_some()
    );

    // Caso Extremo: Estouro do buffer (65 caracteres)
    // Deve imprimir a mensagem de erro no stderr e retornar None
    let estouro = "1".repeat(65);
    assert_eq!(mock_colunas_com_valor(&estouro).get_valor_do_item(), None);
}

#[test]
fn test_multiplos_pontos_milhar() {
    // 1 milhão com pontos de milhar
    assert_eq!(
        mock_colunas_com_valor("1.000.000,00").get_valor_do_item(),
        Some(1000000.0)
    );
}

#[test]
fn test_notacao_cientifica() {
    // Embora raro no SPED, o parse do f64 do Rust suporta
    assert_eq!(
        mock_colunas_com_valor("1.23e4").get_valor_do_item(),
        Some(12300.0)
    );

    assert_eq!(
        mock_colunas_com_valor("4.3e10").get_valor_do_item(),
        Some(43000000000.0)
    );

    assert_eq!(
        mock_colunas_com_valor("-3.6e2").get_valor_do_item(),
        Some(-360.0)
    );
}
