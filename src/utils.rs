use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    BUFFER, Chave, Colunas, Config, DocSummary, Informacoes, SpedError, SpedResult,
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
        .buffer_capacity(BUFFER)
        .from_reader(BufReader::new(file_in));

    // Inicializa contador (considerando header se existir)
    info.numero_total_de_linhas = if rdr.has_headers() { 1 } else { 0 };

    // 2. Configurar Writer com buffer otimizado
    let file_out = File::create(&output_path)?;
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .quote_style(csv::QuoteStyle::Necessary)
        .double_quote(true)
        .buffer_capacity(BUFFER)
        .from_writer(BufWriter::new(file_out));

    let mut alteracoes_realizadas = 0;

    // Reutilizamos o buffer do StringRecord para evitar alocações a cada linha
    let mut record = csv::StringRecord::new();

    while rdr.read_record(&mut record)? {
        info.numero_total_de_linhas += 1;

        // Deserialização "Zero-Copy": os campos da struct Colunas aponta para dentro do 'record'
        // Deserialização com captura detalhada de erro
        let mut row: Colunas = record
            .deserialize(None)
            .map_err(|e| SpedError::CsvDetailed {
                arquivo: input_path.to_path_buf(),
                linha_numero: record.position().map(|p| p.line()).unwrap_or(0),
                conteudo: record.iter().collect::<Vec<_>>().join(";"),
                erro: e.to_string(),
            })?;

        let mut mudou = false;

        // Notas canceladas são apenas escritas de volta sem alteração
        if !row.chave_cancelada() {
            let chave = row.chave;

            if chave.is_nfe() {
                // Se a função adicionar_info mexeu em algum Cow via to_mut(),
                // ele agora é Cow::Owned.
                mudou = adicionar_info_de_ctes_em_nfe(&mut row, config, info, cte_info);
            } else if chave.is_cte() {
                mudou = adicionar_info_de_nfes_em_cte(&mut row, config, info, nfe_info);
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
