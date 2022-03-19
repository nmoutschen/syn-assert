use crate::utils::{self, Check};
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

    fn has_name(&self, name: &str) -> bool;
    fn has_vis(&self, vis: &Visibility) -> bool;
    fn has_attrs(&self, attrs: &[String]) -> bool;
    fn has_block(&self, block: &TokenStream) -> bool;
}

impl HasFn for ItemFn {
    fn has_name(&self, name: &str) -> bool {
        self.sig.ident == name
    }

    fn has_vis(&self, vis: &Visibility) -> bool {
        &self.vis == vis
    }

    fn has_attrs(&self, attrs: &[String]) -> bool {
        let self_attrs = self
            .attrs
            .iter()
            .map(|a| {
                a.path
                    .segments
                    .iter()
                    .fold(String::from(""), |acc, s| acc + "::" + &s.ident.to_string())
            })
            .collect::<HashSet<_>>();
        attrs.iter().all(|attr| self_attrs.contains(attr))
    }

    fn has_block(&self, block: &TokenStream) -> bool {
        self.block.to_token_stream().to_string() == block.to_string()
    }
}

macro_rules! hasfn_item {
    ($v:ident, $t: ty) => {
        paste::paste! {
            fn [<has_ $v>](&self, $v: $t) -> bool {
                match self {
                    Item::Fn(func) => func.[<has_ $v>]($v),
                    _ => false,
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
            fn [<has_ $v>](&self, $v: $t) -> bool {
                self.iter().any(|f| f.[<has_ $v>](&$v))
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
    fn check(self) -> bool {
        utils::option_check!(self, name);
        utils::option_check!(self, vis);
        utils::vec_check!(self, attrs);
        utils::option_check!(self, block);

        true
    }
}
