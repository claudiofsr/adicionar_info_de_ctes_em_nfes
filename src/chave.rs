use serde::Serialize;
use std::fmt;
const NUN_DIGITOS: usize = 44;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Chave([u8; NUN_DIGITOS]);

impl Default for Chave {
    fn default() -> Self {
        // Inicializa o array de 44 bytes com zeros
        Chave([0u8; NUN_DIGITOS])
    }
}

impl Chave {
    // Dica de Performance: Essa implementação de Chave::new é Zero-Allocation
    // no sentido de que ela não cria uma String intermediária (Heap).
    // Ela lê a string original e preenche o array da stack.

    /// Cria uma nova Chave limpando ruídos (aspas, colchetes, espaços).
    /// Retorna None se a string resultante não tiver exatamente 44 dígitos.
    ///
    /// `#[inline]`: Essencial para funções pequenas como essa.
    /// Como você a chamará dentro de loops de milhares de linhas do CSV,
    /// o custo de "pular" para a função new e voltar pode ser maior que o
    /// processamento da chave em si. O inline remove esse custo.
    #[inline]
    pub fn new(s: &str) -> Option<Self> {
        // Otimização de Atalho (Pre-flight check):
        // Se a string tem menos de 44 bytes, é matematicamente impossível
        // conter 44 dígitos. Saímos sem processar nada.
        if s.len() < NUN_DIGITOS {
            return None;
        }

        let mut arr = [0u8; NUN_DIGITOS];
        let mut count = 0;

        // s.bytes() é um iterador de u8, extremamente eficiente.
        for b in s.bytes() {
            // Sub "if b.is_ascii_digit()" por "if b.wrapping_sub(b'0') < 10"
            // Truque de performance: valida se é dígito ASCII (0-9) com uma única operação

            // O truque wrapping_sub transforma 2 comparações em 1.
            // É a forma mais rápida de validar dígitos ASCII.

            // Truque de performance: transforma b >= b'0' && b <= b'9' em uma única comparação.
            // Se b for um dígito, o resultado da subtração será entre 0 e 9.
            // Se b for menor que '0' ou maior que '9', o wrapping_sub resultará em um número grande (u8).
            if b.wrapping_sub(b'0') < 10 {
                // O check 'count < 44' permite ao compilador remover o bounds check de arr[count]
                if count < NUN_DIGITOS {
                    // Otimização de índice: o compilador consegue provar que count não estoura
                    // porque checamos count < 44 logo acima.
                    arr[count] = b;
                    count += 1;
                } else {
                    // Encontrou o 45º dígito: chave inválida, sair imediatamente!
                    return None;
                }
            }
        }

        // Validação final: precisa ter exatamente 44
        if count == NUN_DIGITOS {
            Some(Chave(arr))
        } else {
            None
        }
    }

    /// Atalho para verificar se é NF-e
    #[inline]
    pub fn is_nfe(&self) -> bool {
        &self.0[20..22] == b"55"
    }

    /// Atalho para verificar se é CT-e
    #[inline]
    pub fn is_cte(&self) -> bool {
        &self.0[20..22] == b"57"
    }

    /// Retorna a chave como string slice (&str) para uso em logs ou formatação
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: O construtor `new` garante que apenas dígitos ASCII (0-9) entrem no array.
        // Dígitos ASCII são UTF-8 válidos.
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

// Implementação de Display para facilitar o print (println!("{}", chave))
impl fmt::Display for Chave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Implementação de Debug para visualização técnica
impl fmt::Debug for Chave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chave({})", self.as_str())
    }
}

impl Serialize for Chave {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 44 dígitos + 2 aspas simples = 46 bytes
        let mut buf = [0u8; 46];
        buf[0] = b'\''; // Aspa inicial
        buf[1..45].copy_from_slice(&self.0); // Copia os 44 bytes da chave
        buf[45] = b'\''; // Aspa final

        // Converte o buffer da stack para string slice (&str)
        // Usamos unchecked porque sabemos que são dígitos ASCII e aspas
        let s = unsafe { std::str::from_utf8_unchecked(&buf) };
        serializer.serialize_str(s)
    }
}

impl<'de> serde::Deserialize<'de> for Chave {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Visitor};
        struct ChaveVisitor;

        impl<'v> Visitor<'v> for ChaveVisitor {
            type Value = Chave;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("uma string de 44 dígitos (podendo conter aspas ou espaços)")
            }
            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Chave::new(v).ok_or_else(|| E::custom("Chave de acesso inválida"))
            }
        }
        deserializer.deserialize_str(ChaveVisitor)
    }
}
