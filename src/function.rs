use crate::utils::{check_option, Check, CheckResult};
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Item, ItemFn, Visibility};

pub trait HasFn {
    fn has_fn(&self) -> AssertFn<'_, Self>
    where
        Self: Sized,
    {
        AssertFn::new(self)
    }

    fn has_name(&self, name: &str) -> CheckResult;
    fn has_vis(&self, vis: &Visibility) -> CheckResult;
    fn has_attrs(&self, attrs: &[String]) -> CheckResult;
    fn has_block(&self, block: &TokenStream) -> CheckResult;
}

impl HasFn for ItemFn {
    fn has_name(&self, name: &str) -> CheckResult {
        CheckResult::compare(name, &self.sig.ident)
    }

    fn has_vis(&self, vis: &Visibility) -> CheckResult {
        CheckResult::compare(vis, &self.vis)
    }

    fn has_attrs(&self, attrs: &[String]) -> CheckResult {
        let self_attrs = self
            .attrs
            .iter()
            .map(|a| {
                a.path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            })
            .collect::<HashSet<_>>();
        CheckResult::contains(self_attrs, attrs)
    }

    fn has_block(&self, block: &TokenStream) -> CheckResult {
        CheckResult::compare(block.to_string(), self.block.to_token_stream().to_string())
    }
}

macro_rules! hasfn_item {
    ($v:ident, $t: ty) => {
        paste::paste! {
            fn [<has_ $v>](&self, $v: $t) -> CheckResult {
                match self {
                    Item::Fn(func) => func.[<has_ $v>]($v),
                    _ => CheckResult::missing(stringify!($v)),
                }
            }
        }
    };
}

impl HasFn for Item {
    hasfn_item!(name, &str);
    hasfn_item!(vis, &Visibility);
    hasfn_item!(attrs, &[String]);
    hasfn_item!(block, &TokenStream);
}

macro_rules! hasfn_vec {
    ($v: ident, $t: ty) => {
        paste::paste! {
            fn [<has_ $v>](&self, $v: $t) -> CheckResult {
                CheckResult::any(self.iter().map(|f| f.[<has_ $v>](&$v)))
            }
        }
    };
}

impl<T> HasFn for Vec<T>
where
    T: HasFn,
{
    hasfn_vec!(name, &str);
    hasfn_vec!(vis, &Visibility);
    hasfn_vec!(attrs, &[String]);
    hasfn_vec!(block, &TokenStream);
}

pub struct AssertFn<'s, T> {
    t: &'s T,
    name: Option<&'s str>,
    vis: Option<Visibility>,
    attrs: Vec<String>,
    block: Option<TokenStream>,
}

impl<'s, T> AssertFn<'s, T> {
    pub fn new(t: &'s T) -> Self {
        Self {
            t,
            name: Default::default(),
            vis: Default::default(),
            attrs: Default::default(),
            block: Default::default(),
        }
    }

    pub fn with_name(self, name: &'s str) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    pub fn with_vis(self, vis: Visibility) -> Self {
        Self {
            vis: Some(vis),
            ..self
        }
    }

    pub fn with_attrs(self, attrs: Vec<String>) -> Self {
        Self { attrs, ..self }
    }

    pub fn with_block(self, block: TokenStream) -> Self {
        Self {
            block: Some(block),
            ..self
        }
    }
}

impl<'s, T> Check for AssertFn<'s, T>
where
    T: HasFn,
{
    fn check(self) -> CheckResult {
        check_option!(self, name)
            + check_option!(self, vis)
            + check_option!(self, block)
            + self.t.has_attrs(&self.attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error;

    type TestError = Box<dyn error::Error>;

    #[test]
    fn test_itemfn() -> Result<(), TestError> {
        let func: syn::ItemFn = syn::parse_str(
            r#"
            fn main() { println!("Hello, world!"); }
        "#,
        )?;
        let block = quote::quote! { {
            println!("Hello, world!");
        } };

        let results = func.has_fn().with_name("main").with_block(block).check();
        dbg!(&results);
        assert!(results.as_bool());

        Ok(())
    }

    #[test]
    fn test_item() -> Result<(), TestError> {
        let func: syn::Item = syn::parse_str(
            r#"
            fn main() { println!("Hello, world!"); }
        "#,
        )?;
        let block = quote::quote! { {
            println!("Hello, world!");
        } };

        let results = func.has_fn().with_name("main").with_block(block).check();
        dbg!(&results);
        assert!(results.as_bool());

        Ok(())
    }

    #[test]
    fn test_file() -> Result<(), TestError> {
        let func: syn::File = syn::parse_str(
            r#"
            fn main() { println!("Hello, world!"); }
        "#,
        )?;
        let block = quote::quote! { {
            println!("Hello, world!");
        } };

        let results = func
            .items
            .has_fn()
            .with_name("main")
            .with_block(block)
            .check();
        dbg!(&results);
        assert!(results.as_bool());

        Ok(())
    }

    #[test]
    fn test_name() -> Result<(), TestError> {
        let func: syn::ItemFn = syn::parse_str(
            r#"
            fn main() { println!("Hello, world!"); }
        "#,
        )?;

        let results = func.has_fn().with_name("main").check();
        dbg!(&results);
        assert!(results.as_bool());

        Ok(())
    }

    #[test]
    fn test_name_fail() -> Result<(), TestError> {
        let func: syn::ItemFn = syn::parse_str(
            r#"
            fn main() { println!("Hello, world!"); }
        "#,
        )?;

        let results = func.has_fn().with_name("not_main").check();
        dbg!(&results);
        assert!(!results.as_bool());

        Ok(())
    }

    #[test]
    fn test_attrs_1() -> Result<(), TestError> {
        let func: syn::ItemFn = syn::parse_str(
            r#"
            #[my_attr]
            fn main() { println!("Hello, world!"); }
        "#,
        )?;

        let results = func
            .has_fn()
            .with_attrs(vec!["my_attr".to_string()])
            .check();
        dbg!(&results);
        assert!(results.as_bool());

        Ok(())
    }

    #[test]
    fn test_attrs_2() -> Result<(), TestError> {
        let func: syn::ItemFn = syn::parse_str(
            r#"
            #[my_attr]
            #[a_crate::my_other_attr]
            fn main() { println!("Hello, world!"); }
            "#,
        )?;

        let results = func
            .has_fn()
            .with_attrs(vec![
                "my_attr".to_string(),
                "a_crate::my_other_attr".to_string(),
            ])
            .check();
        dbg!(&results);
        assert!(results.as_bool());

        Ok(())
    }

    #[test]
    fn test_attrs_2_fail() -> Result<(), TestError> {
        let func: syn::ItemFn = syn::parse_str(
            r#"
            #[my_attr]
            fn main() { println!("Hello, world!"); }
            "#,
        )?;

        let results = func
            .has_fn()
            .with_attrs(vec![
                "my_attr".to_string(),
                "a_crate::my_other_attr".to_string(),
            ])
            .check();
        dbg!(&results);
        assert!(!results.as_bool());

        Ok(())
    }
}
