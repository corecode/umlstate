use syn::{Error, Result};

use crate::parse;

pub struct Model {
    pub items: Vec<MachineModel>,
}

pub struct MachineModel {
    pub ident: syn::Ident,
}

pub fn analyze(ast: parse::UmlState) -> Result<Model> {
    Ok(Model {
        items: {
            let mut items = Vec::new();
            for item in ast.items {
                items.push(analyze_machine(item)?);
            }
            items
        },
    })
}

fn analyze_machine(machine: parse::Machine) -> Result<MachineModel> {
    Ok(MachineModel {
        ident: machine.ident,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let ast: parse::UmlState = syn::parse_quote! {
            machine Foo {
            }
        };

        let model = analyze(ast).unwrap();
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].ident, "Foo");
    }
}
