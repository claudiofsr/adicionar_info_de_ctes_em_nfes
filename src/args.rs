use clap::Parser;
use std::{borrow::Cow, path::PathBuf};

use crate::{SpedError, SpedResult};

// Estrutura para o Clap processar os argumentos da linha de comando
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Atualizar arquivo CSV original
    #[arg(short, long, default_value_t = false)]
    atualizar_origem: bool,

    /// Clear screen
    #[arg(short, long, default_value_t = false)]
    clear: bool,

    /// Arquivo de Documentos Fiscais.
    ///
    /// Exemplo de arquivo esperado:
    ///
    /// - `ZZZ-874918-Info da Receita sobre o Contribuinte.csv`
    #[arg(short, long, required = true)]
    doc_path: Option<PathBuf>,

    /// Imprimir configuração
    #[arg(short, long, default_value_t = false)]
    exibir_config: bool,

    /// Máximo de caracteres por coluna
    #[arg(long, default_value_t = 3000)]
    max_char: usize,

    /// Máximo de informações de docs fiscais adicionados
    #[arg(long, default_value_t = 10)]
    max_info: usize,

    /// Não perguntar se deseja sobrescrever o original (apenas gera o arquivo modificado)
    #[arg(short, long, default_value_t = false)]
    no_prompt: bool,

    /// Ativar modo detalhado (verbose)
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug, Default)]
pub struct Config {
    pub atualizar_origem: bool,
    pub clear: bool,
    pub doc_path: PathBuf,
    pub exibir_config: bool,
    pub max_char: usize,
    pub max_info: usize,
    pub no_prompt: bool,
    pub verbose: bool,
}

impl Config {
    /// Adiciona informações a um campo de texto respeitando o limite de caracteres.
    ///
    /// - `field`: Referência mutável para a coluna que receberá o texto.
    /// - `value`: O dado a ser injetado (ignora se estiver vazio).
    /// - `label`: O prefixo da informação (ex: "CT-e" ou "NF-e").
    #[inline]
    pub fn append<'a>(&self, field: &mut Cow<'a, str>, value: &str, label: &str) {
        // Otimização: se o valor de origem for vazio, não há o que adicionar
        let value = value.trim();
        if value.is_empty() {
            return;
        }

        // Definimos o artigo ("da" para NF-e, "do" para o restante)
        let artigo = if label == "NF-e" { 'a' } else { 'o' };
        let sufixo = format!(" [Info d{} {}: {}]", artigo, label, value);

        // Cálculo de tamanho Unicode-aware sem alocar String.
        // O sufixo segue o padrão: " [Info dX YYY: ZZZ]"
        // Constantes: " [Info d" (9) + " " (1) + ": " (2) + "]" (1) = 13 caracteres fixos
        // Variáveis: artigo (1) + label.len + value.len
        let tamanho_atual = field.chars().count();
        let tamanho_sufixo = 14 + label.chars().count() + value.chars().count();

        if tamanho_atual + tamanho_sufixo < self.max_char {
            // Se o campo for Borrowed, to_mut() faz o clone para String apenas aqui
            field.to_mut().push_str(&sufixo);
        }
    }
}

pub fn get_config() -> SpedResult<Config> {
    let args = Arguments::parse();

    // 1. Extração funcional: Converte Option<PathBuf> em PathBuf ou retorna erro
    // Como o Clap já exige 'required = true', este erro só ocorreria em casos extremos.
    let doc_path = args.doc_path.ok_or(SpedError::EfdFileNotFound)?;

    Ok(Config {
        atualizar_origem: args.atualizar_origem,
        clear: args.clear,
        doc_path,
        exibir_config: args.exibir_config,
        max_char: args.max_char,
        max_info: args.max_info,
        no_prompt: args.no_prompt,
        verbose: args.verbose,
    })
}
