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
    pub fn new(s: &str) -> Option<Self> {
        let mut arr = [0u8; NUN_DIGITOS];
        let mut count = 0;

        // Itera pelos bytes da string original
        for b in s.bytes() {
            if b.is_ascii_digit() {
                if count < NUN_DIGITOS {
                    arr[count] = b;
                    count += 1;
                } else {
                    // Encontrou o 45º dígito: chave inválida, sai na hora!
                    return None;
                }
            }
        }

        // Validação: precisa ter exatamente 44 dígitos.
        // Se houver menos de 44 dígitos, a chave é inválida.
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
