use std::{io, path::PathBuf};
use thiserror::Error;

/// Tipo de retorno conveniente para todo o projeto
pub type SpedResult<T> = Result<T, SpedError>;

#[derive(Error, Debug)]
pub enum SpedError {
    #[error("Erro de configuração: {0}")]
    Config(String),

    #[error("Erro no processamento CSV: {0}")]
    Csv(#[from] csv::Error),

    #[error(
        "Erro estrutural no CSV\n\
        Arquivo: <{arquivo}>\n\
        Linha: {linha_numero}\n\
        Erro: {erro}\nConteúdo: {conteudo}"
    )]
    CsvDetailed {
        arquivo: PathBuf,
        linha_numero: u64,
        conteudo: String,
        erro: String,
    },

    #[error("Arquivo <{arquivo}> contém colunas repetidas: <{coluna}> no arquivo <{arquivo}>")]
    DuplicateColumnName { arquivo: PathBuf, coluna: String },

    #[error(
        "Arquivo EFD não definido ou inválido!\n\
        Exemplo:\n\
        reter_linhas_com_info_das_chaves -n 15 -e 'Info do Contribuinte EFD Contribuicoes.csv'"
    )]
    EfdFileNotFound,

    #[error("Arquivo <{arquivo}> contém colunas com nome em branco!")]
    EmptyColumnName { arquivo: PathBuf },

    #[error("Erro de I/O: {0}")]
    Io(#[from] io::Error),

    #[error(
        "Arquivo EFD não encontrado!\n\
        Arquivo: {arquivo:?}\n\
        {source}"
    )]
    IoReader {
        #[source] // Indica que este é o erro original
        source: io::Error,
        arquivo: PathBuf,
    },

    #[error("Regex Error: {0}")]
    Regex(#[from] regex::Error),
}
