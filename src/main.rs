use adicionar_info_de_ctes_em_nfes::{
    Informacoes, SpedResult, clear_screen, enriquecer_arquivo, get_config, get_summaries,
    imprimir_versao_do_programa, sobrescrever_arquivo,
};
use execution_time::ExecutionTime;
use std::{fs, process};

/*
05.adicionar_info_de_CTes_em_NFes.pl -i 'ZZZ-874918-Info da Receita sobre o Contribuinte.csv'
cargo run -- -e 'ZZZ-874918-Info da Receita sobre o Contribuinte.csv'

 b3sum *modificado*

Comparar dois arquivos utilizando codium.
Mais eficiente que meld?

# 1. Definir as variáveis (use aspas simples para evitar expansões indesejadas na atribuição)
arquivo1='ZZZ-874918-Info da Receita sobre o Contribuinte-modificado.csv'
arquivo2='ZZZ-874918-Info da Receita sobre o Contribuinte.modificado.csv'

# 2. Executar o diff com substituição de processo
# Importante: "$arquivo1" com $ e entre aspas dentro do head
codium --diff =(head -n 100 "$arquivo1") =(head -n 100 "$arquivo2") &
codium --diff =(head -n 31850 "$arquivo1" | tail -n 20) =(head -n 31850 "$arquivo2" | tail -n 20) &

meld =(head -n 100 "$arquivo1") =(head -n 100 "$arquivo2") &
b3sum =(head -n 100 "$arquivo1") =(head -n 100 "$arquivo2")
*/

fn main() {
    if let Err(err) = run() {
        eprintln!("\n[ERRO CRÍTICO]: {err}");
        process::exit(1);
    }
}

fn run() -> SpedResult<()> {
    let timer = ExecutionTime::start();

    // 1. Configurações (Parâmetros da CLI) (O "O QUE" fazer)
    let config = get_config()?;

    clear_screen(config.clear)?;
    imprimir_versao_do_programa();

    if config.exibir_config {
        println!("{:#?}\n", config);
    }

    // 2. Informações (O "COM O QUE" trabalhar)
    // Toda a complexidade de arquivos texto e transitividade está escondida aqui
    let mut info = Informacoes::from_files(
        "cte_nfes.txt",
        "transporte_subcontratado-chaves_complementares_dos_CTes.txt",
    )?;

    // 3. Processamento (A execução propriamente dita)
    println!("--- Passagem 1: Coletando resumos de documentos ---");
    let (cte_info, nfe_info) = get_summaries(&config.doc_path, &config)?;

    if config.verbose {
        println!("\n--- Primeiros 10 CTes encontrados ---\n");
        for (chave, doc_summary) in cte_info.iter().take(10) {
            println!("chave_cte: {chave} ; doc_summary: {doc_summary:?}\n");
        }

        println!("\n--- Primeiras 10 NFes encontradas ---\n");
        for (chave, doc_summary) in nfe_info.iter().take(10) {
            println!("chave_nfe: {chave} ; doc_summary: {doc_summary:?}\n");
        }
    }

    // 8. Passagem 2: Enriquecimento
    let (output_path, alteracoes) = enriquecer_arquivo(&config, &mut info, &cte_info, &nfe_info)?;

    println!("Arquivo: {:?}", output_path.display());
    println!("Número total de linhas: {}\n", info.numero_total_de_linhas);

    // 9. Finalização
    timer.print_elapsed_time();
    println!();

    if alteracoes == 0 {
        println!(" -> ATENÇÃO: Nenhuma correspondência encontrada. Removendo arquivo temporário.");
        fs::remove_file(&output_path)?;
    } else if config.atualizar_origem {
        fs::rename(&output_path, &config.doc_path)?;
        println!(" -> Arquivo original atualizado automaticamente.");
    } else if config.no_prompt {
        println!(
            " -> Arquivo modificado gerado com sucesso em: '{}'",
            output_path.display()
        );
        println!(" -> Encerrando sem sobrescrever o original (--no-prompt ativado).");
    } else {
        // Se não houver flag de atualizar nem de no-prompt, pergunta ao usuário
        sobrescrever_arquivo(&config.doc_path, &output_path)?;
    }

    Ok(())
}
