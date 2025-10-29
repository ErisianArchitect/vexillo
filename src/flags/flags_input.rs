use syn::{Attribute, Ident, Type, Visibility};



pub struct FlagsInput {
    pub cfg_attrs: Vec<Attribute>,
    pub attrs: Vec<Attribute>,
    pub type_vis: Visibility,
    pub type_name: Ident,
    pub mask_type: Type,
}