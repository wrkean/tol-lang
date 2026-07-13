use phf::phf_map;

/// This language's types
#[derive(Debug, Clone)]
pub enum TolType {
    Lutang,
    Numero,

    DiAlam,
}

pub static TYPES: phf::Map<&'static str, TolType> = phf_map! {
    "lutang" => TolType::Lutang,
    "numero" => TolType::Numero,
};

pub fn type_list() -> Vec<String> {
    TYPES.keys().map(|k| k.to_string()).collect()
}
