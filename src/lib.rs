mod args;
mod chave;
mod colunas;
mod error;
mod informacoes;
mod processor;
mod regex;
mod utils;

pub use self::{
    args::*, chave::*, colunas::*, error::*, informacoes::*, processor::*, regex::*, utils::*,
};

pub const BUFFER: usize = 1014 * 1024; // 1MB
