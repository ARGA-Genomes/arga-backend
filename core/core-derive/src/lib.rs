use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


#[proc_macro_derive(OperationLog)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let atom = match data {
        syn::Data::Struct(st) => {
            // find the field named 'atom'
            st.fields
                .iter()
                .find(|field| match &field.ident {
                    Some(field) => field == "atom",
                    None => false,
                })
                .expect("operation log structs must have a field named 'atom' with an associated atom type")
                .clone()
        }
        _ => panic!("Only structs are supported as operation logs"),
    };

    let atom_type = match atom.ty {
        syn::Type::Path(ty) => ty
            .path
            .get_ident()
            .expect("Failed to get operation log atom type path")
            .clone(),
        _ => panic!("Only type paths to enums are supported as operation log atoms"),
    };

    let from_data_frame_operation = quote! {
        impl From<DataFrameOperation<#atom_type>> for #ident {
            fn from(op: DataFrameOperation<#atom_type>) -> Self {
                Self {
                    operation_id: op.operation_id,
                    parent_id: op.parent_id,
                    entity_id: op.entity_id,
                    dataset_version_id: op.dataset_version_id,
                    action: op.action,
                    atom: op.atom,
                }
            }
        }
    };

    let log_operation = quote! {
        impl LogOperation<#atom_type> for #ident {
            fn id(&self) -> &BigDecimal { &self.operation_id }
            fn entity_id(&self) -> &String { &self.entity_id }
            fn action(&self) -> &Action { &self.action }
            fn atom(&self) -> &#atom_type { &self.atom }
        }
    };

    quote! {
        #from_data_frame_operation
        #log_operation
    }
    .into()
}


#[proc_macro_derive(Atom)]
pub fn derive_atom(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    quote! {
        impl FromSql<Jsonb, Pg> for #ident {
            fn from_sql(value: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
                serde_json::from_value(FromSql::<Jsonb, Pg>::from_sql(value)?).map_err(|e| e.into())
            }
        }

        impl ToSql<Jsonb, Pg> for #ident {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
                let json = serde_json::to_value(self)?;
                <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&json, &mut out.reborrow())
            }
        }
    }
    .into()
}
