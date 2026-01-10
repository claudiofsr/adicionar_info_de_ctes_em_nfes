use crate::{Chave, RE_MULTISPACE};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Colunas<'a> {
    // --- Identificação do Contribuinte e Participante ---
    #[serde(rename = "CNPJ do Contribuinte : NF Item (Todos)")]
    pub contribuinte_cnpj: Cow<'a, str>,

    #[serde(rename = "Nome do Contribuinte : NF Item (Todos)")]
    pub contribuinte_nome: Cow<'a, str>,

    #[serde(rename = "Entrada/Saída : NF (Todos)")]
    pub entrada_ou_saida: Cow<'a, str>,

    #[serde(rename = "CPF/CNPJ do Participante : NF (Todos)")]
    pub participante_cnpj: Cow<'a, str>,

    #[serde(rename = "Nome do Participante : NF (Todos)")]
    pub participante_nome: Cow<'a, str>,

    #[serde(rename = "CRT : NF (Todos)")]
    pub regime_tributario: Cow<'a, str>,

    #[serde(rename = "Observações : NF (Todos)")]
    pub observacoes: Cow<'a, str>,

    // --- CTe: Remetente ---
    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: CNPJ/CPF de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub remetente_cnpj1: Cow<'a, str>,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: CNPJ/CPF de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub remetente_cnpj2: Cow<'a, str>,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: Nome de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub remetente_nome: Cow<'a, str>,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: Município de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub remetente_municipio: Cow<'a, str>,

    // --- CTe: Tomador ---
    #[serde(
        rename = "Descrição CTe - Indicador do 'papel' do tomador do serviço de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub tomador_papel1: Cow<'a, str>,

    #[serde(
        rename = "Descrição CTe - Indicador do 'papel' do tomador do serviço de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub tomador_papel2: Cow<'a, str>,

    #[serde(
        rename = "CTe - Outro tipo de Tomador: CNPJ/CPF de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub tomador_cnpj1: Cow<'a, str>,

    #[serde(
        rename = "CTe - Outro tipo de Tomador: CNPJ/CPF de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub tomador_cnpj2: Cow<'a, str>,

    // --- CTe: Percurso ---
    #[serde(
        rename = "CTe - UF do início da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub inicio_estado: Cow<'a, str>,

    #[serde(
        rename = "CTe - Nome do Município do início da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub inicio_municipio: Cow<'a, str>,

    #[serde(
        rename = "CTe - UF do término da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub termino_estado: Cow<'a, str>,

    #[serde(
        rename = "CTe - Nome do Município do término da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub termino_municipio: Cow<'a, str>,

    // --- CTe: Destinatário e Entrega ---
    #[serde(
        rename = "CTe - Informações do Destinatário do CT-e: CNPJ/CPF de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub destinatario_cnpj: Cow<'a, str>,

    #[serde(
        rename = "CTe - Informações do Destinatário do CT-e: Nome de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub destinatario_nome: Cow<'a, str>,

    #[serde(
        rename = "CTe - Local de Entrega constante na Nota Fiscal: Nome de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub local_entrega: Cow<'a, str>,

    // --- Dados do Documento Fiscal ---
    #[serde(rename = "Descrição da Natureza da Operação : NF Item (Todos)")]
    pub descricao_natureza: Cow<'a, str>,

    #[serde(rename = "Cancelada : NF (Todos)")]
    pub cancelada: Cow<'a, str>,

    #[serde(rename = "Registro de Origem do Item : NF Item (Todos)")]
    pub origem: Cow<'a, str>,

    #[serde(rename = "Natureza da Base de Cálculo do Crédito Descrição : NF Item (Todos)")]
    pub natureza_bc: Cow<'a, str>,

    #[serde(rename = "Modelo - Descrição : NF Item (Todos)")]
    pub modelo: Cow<'a, str>,

    #[serde(rename = "Número da Nota : NF Item (Todos)")]
    pub num_doc: Cow<'a, str>,

    #[serde(rename = "Chave da Nota Fiscal Eletrônica : NF Item (Todos)")]
    pub chave: Chave, // Chave é Copy (array fixo), não precisa de Cow

    #[serde(rename = "Inf. NFe - Chave de acesso da NF-e : ConhecimentoInformacaoNFe")]
    pub chave_de_acesso: Cow<'a, str>,

    #[serde(rename = "CTe - Observações Gerais de Conhecimento : ConhecimentoInformacaoNFe")]
    pub observacoes_gerais: Cow<'a, str>,

    // --- Detalhes do Item ---
    #[serde(rename = "Dia da Emissão : NF Item (Todos)")]
    pub dia_emissao: Cow<'a, str>,

    #[serde(rename = "Número da DI : NF Item (Todos)")]
    pub numero_di: Cow<'a, str>,

    #[serde(rename = "Número do Item : NF Item (Todos)")]
    pub numero_item: Cow<'a, str>,

    #[serde(rename = "Código CFOP : NF Item (Todos)")]
    pub cfop: Cow<'a, str>,

    #[serde(rename = "Descrição CFOP : NF Item (Todos)")]
    pub descricao_cfop: Cow<'a, str>,

    #[serde(rename = "Descrição da Mercadoria/Serviço : NF Item (Todos)")]
    pub descricao_mercadoria: Cow<'a, str>,

    #[serde(rename = "Código NCM : NF Item (Todos)")]
    pub ncm: Cow<'a, str>,

    #[serde(rename = "Descrição NCM : NF Item (Todos)")]
    pub descricao_ncm: Cow<'a, str>,

    // --- Alíquotas e CST ---
    #[serde(rename = "COFINS: Alíquota ad valorem - Atributo : NF Item (Todos)")]
    pub aliq_cofins: Cow<'a, str>,

    #[serde(rename = "PIS: Alíquota ad valorem - Atributo : NF Item (Todos)")]
    pub aliq_pis: Cow<'a, str>,

    #[serde(rename = "CST COFINS Descrição : NF Item (Todos)")]
    pub cst_descricao_cofins: Cow<'a, str>,

    #[serde(rename = "CST PIS Descrição : NF Item (Todos)")]
    pub cst_descricao_pis: Cow<'a, str>,

    // --- Valores Monetários (SOMA) ---
    #[serde(rename = "Valor Total : NF (Todos) SOMA")]
    pub valor_total: Cow<'a, str>,

    #[serde(rename = "Valor da Nota Proporcional : NF Item (Todos) SOMA")]
    pub valor_item: Cow<'a, str>,

    #[serde(rename = "Valor dos Descontos : NF Item (Todos) SOMA")]
    pub valor_desconto: Cow<'a, str>,

    #[serde(rename = "Valor Seguro : NF (Todos) SOMA")]
    pub valor_seguro: Cow<'a, str>,

    #[serde(rename = "COFINS: Valor do Tributo : NF Item (Todos) SOMA")]
    pub valor_cofins: Cow<'a, str>,

    #[serde(rename = "PIS: Valor do Tributo : NF Item (Todos) SOMA")]
    pub valor_pis: Cow<'a, str>,

    #[serde(rename = "IPI: Valor do Tributo : NF Item (Todos) SOMA")]
    pub valor_ipi: Cow<'a, str>,

    #[serde(rename = "ISS: Base de Cálculo : NF Item (Todos) SOMA")]
    pub valor_bc_iss: Cow<'a, str>,

    #[serde(rename = "ISS: Valor do Tributo : NF Item (Todos) SOMA")]
    pub valor_iss: Cow<'a, str>,

    #[serde(rename = "ICMS: Alíquota : NF Item (Todos) NOISE OR")]
    pub aliq_icms: Cow<'a, str>,

    #[serde(rename = "ICMS: Base de Cálculo : NF Item (Todos) SOMA")]
    pub valor_bc_icms: Cow<'a, str>,

    #[serde(rename = "ICMS: Valor do Tributo : NF Item (Todos) SOMA")]
    pub valor_icms: Cow<'a, str>,

    #[serde(rename = "ICMS por Substituição: Valor do Tributo : NF Item (Todos) SOMA")]
    pub valor_icms_sub: Cow<'a, str>,
}

impl<'a> Colunas<'a> {
    pub fn ncm_valido(&self) -> bool {
        self.ncm.bytes().any(|b| matches!(b, b'1'..=b'9'))
    }

    pub fn get_valor_do_item(&self) -> Option<f64> {
        self.valor_item
            .replace('.', "")
            .replace(',', ".")
            .parse::<f64>()
            .ok()
    }

    pub fn chave_cancelada(&self) -> bool {
        matches!(
            self.cancelada.to_lowercase().as_str(),
            "sim" | "s" | "true" | "1"
        )
    }

    /// Performance Máxima (Zero Allocation):
    ///
    /// Se a regex não encontrar espaços múltiplos, o replace_all retorna Cow::Borrowed.
    ///
    /// Nesse caso, o if let falha, e nada acontece.
    ///
    /// Você não aloca memória e não sobrescreve o campo desnecessariamente.       
    pub fn sanitizar_campo(field: &mut Cow<'a, str>) {
        if field.contains("  ") {
            // 1. Fazemos o replace_all. O resultado é um Cow temporário.
            let replaced = RE_MULTISPACE.replace_all(field, " ");

            // 2. Verificamos se houve mudança (se é Owned).
            // Se for Owned, movemos a nova String para dentro do campo da struct.
            if let Cow::Owned(nova_string) = replaced {
                *field = Cow::Owned(nova_string);
            }
        }
    }

    /// Transforma todos os campos Borrowed em Owned para que a struct possa
    /// ser armazenada de forma independente (ex: no DocSummary).
    pub fn into_owned(self) -> Colunas<'static> {
        Colunas {
            contribuinte_cnpj: Cow::Owned(self.contribuinte_cnpj.into_owned()),
            contribuinte_nome: Cow::Owned(self.contribuinte_nome.into_owned()),
            entrada_ou_saida: Cow::Owned(self.entrada_ou_saida.into_owned()),
            participante_cnpj: Cow::Owned(self.participante_cnpj.into_owned()),
            participante_nome: Cow::Owned(self.participante_nome.into_owned()),
            regime_tributario: Cow::Owned(self.regime_tributario.into_owned()),
            observacoes: Cow::Owned(self.observacoes.into_owned()),
            remetente_cnpj1: Cow::Owned(self.remetente_cnpj1.into_owned()),
            remetente_cnpj2: Cow::Owned(self.remetente_cnpj2.into_owned()),
            remetente_nome: Cow::Owned(self.remetente_nome.into_owned()),
            remetente_municipio: Cow::Owned(self.remetente_municipio.into_owned()),
            tomador_papel1: Cow::Owned(self.tomador_papel1.into_owned()),
            tomador_papel2: Cow::Owned(self.tomador_papel2.into_owned()),
            tomador_cnpj1: Cow::Owned(self.tomador_cnpj1.into_owned()),
            tomador_cnpj2: Cow::Owned(self.tomador_cnpj2.into_owned()),
            inicio_estado: Cow::Owned(self.inicio_estado.into_owned()),
            inicio_municipio: Cow::Owned(self.inicio_municipio.into_owned()),
            termino_estado: Cow::Owned(self.termino_estado.into_owned()),
            termino_municipio: Cow::Owned(self.termino_municipio.into_owned()),
            destinatario_cnpj: Cow::Owned(self.destinatario_cnpj.into_owned()),
            destinatario_nome: Cow::Owned(self.destinatario_nome.into_owned()),
            local_entrega: Cow::Owned(self.local_entrega.into_owned()),
            descricao_natureza: Cow::Owned(self.descricao_natureza.into_owned()),
            cancelada: Cow::Owned(self.cancelada.into_owned()),
            origem: Cow::Owned(self.origem.into_owned()),
            natureza_bc: Cow::Owned(self.natureza_bc.into_owned()),
            modelo: Cow::Owned(self.modelo.into_owned()),
            num_doc: Cow::Owned(self.num_doc.into_owned()),
            chave: self.chave,
            chave_de_acesso: Cow::Owned(self.chave_de_acesso.into_owned()),
            observacoes_gerais: Cow::Owned(self.observacoes_gerais.into_owned()),
            dia_emissao: Cow::Owned(self.dia_emissao.into_owned()),
            numero_di: Cow::Owned(self.numero_di.into_owned()),
            numero_item: Cow::Owned(self.numero_item.into_owned()),
            cfop: Cow::Owned(self.cfop.into_owned()),
            descricao_cfop: Cow::Owned(self.descricao_cfop.into_owned()),
            descricao_mercadoria: Cow::Owned(self.descricao_mercadoria.into_owned()),
            ncm: Cow::Owned(self.ncm.into_owned()),
            descricao_ncm: Cow::Owned(self.descricao_ncm.into_owned()),
            aliq_cofins: Cow::Owned(self.aliq_cofins.into_owned()),
            aliq_pis: Cow::Owned(self.aliq_pis.into_owned()),
            cst_descricao_cofins: Cow::Owned(self.cst_descricao_cofins.into_owned()),
            cst_descricao_pis: Cow::Owned(self.cst_descricao_pis.into_owned()),
            valor_total: Cow::Owned(self.valor_total.into_owned()),
            valor_item: Cow::Owned(self.valor_item.into_owned()),
            valor_desconto: Cow::Owned(self.valor_desconto.into_owned()),
            valor_seguro: Cow::Owned(self.valor_seguro.into_owned()),
            valor_cofins: Cow::Owned(self.valor_cofins.into_owned()),
            valor_pis: Cow::Owned(self.valor_pis.into_owned()),
            valor_ipi: Cow::Owned(self.valor_ipi.into_owned()),
            valor_bc_iss: Cow::Owned(self.valor_bc_iss.into_owned()),
            valor_iss: Cow::Owned(self.valor_iss.into_owned()),
            aliq_icms: Cow::Owned(self.aliq_icms.into_owned()),
            valor_bc_icms: Cow::Owned(self.valor_bc_icms.into_owned()),
            valor_icms: Cow::Owned(self.valor_icms.into_owned()),
            valor_icms_sub: Cow::Owned(self.valor_icms_sub.into_owned()),
        }
    }
}
