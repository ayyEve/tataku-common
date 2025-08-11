use quote::*;
use syn::*;
use syn::punctuated::Punctuated;

pub const REFLECT_ATTRIBUTE: &str = "reflect";
pub const SKIP_ATTRIBUTE: &str = "skip";
pub const RENAME_ATTRIBUTE: &str = "rename";
pub const ALIAS_ATTRIBUTE: &str = "alias";
pub const FLATTEN_ATTRIBUTE: &str = "flatten";
pub const DONT_CLONE_ATTRIBUTE: &str = "dont_clone";

pub const FROM_STRING_ATTRIBUTE: &str = "from_string";
pub const FROM_STRING_NONE: &str = "none";
pub const FROM_STRING_FROM_STR: &str = "from_str";
pub const FROM_STRING_AUTO: &str = "auto";

pub const DISPLAY_ATTRIBUTE: &str = "display";
pub const DISPLAY_NONE: &str = "none";
pub const DISPLAY_DISPLAY: &str = "display";
pub const DISPLAY_DEBUG: &str = "debug";

macro_rules! try_error {
    ($($t:tt)+) => {
        match $($t)+ {
            Ok(ok) => ok,
            Err(e) => return e.into_compile_error(),
        }
    };
}

#[derive(Default)]
struct ReflectAttributes {
    skip: bool,
    dont_clone: bool,
    display: DisplayType,
    from_string_type: FromStringType,
    rename: Option<LitStr>,
    aliases: Vec<LitStr>,
}

#[derive(Copy, Clone, Default)]
enum FromStringType {
    #[default]
    None,
    FromStr,
    AutoFromStr,
}
#[derive(Copy, Clone, Default)]
enum DisplayType {
    #[default]
    None,
    Display,
    Debug,
    // Custom(Ident)
}


impl ReflectAttributes {
    fn parse_from_attrs(attrs: &[Attribute], global: bool) -> Result<Self> {
        let mut reflect = Self::default();

        for attr in attrs {
            if !attr.path().is_ident(REFLECT_ATTRIBUTE) { continue; }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(SKIP_ATTRIBUTE) {
                    reflect.skip = true;
                }
                else if meta.path.is_ident(DONT_CLONE_ATTRIBUTE) {
                    reflect.dont_clone = true;
                }
                else if meta.path.is_ident(DISPLAY_ATTRIBUTE) {
                    let _ = meta.value()?;

                    let name: LitStr = meta.input.parse()?;

                    reflect.display = match &*name.value() {
                        DISPLAY_NONE => DisplayType::None,
                        DISPLAY_DISPLAY => DisplayType::Display,
                        DISPLAY_DEBUG => DisplayType::Debug,
                        _ => return Err(meta.error("Invalid display type")),
                    }
                }
                else if meta.path.is_ident(FROM_STRING_ATTRIBUTE) {
                    let _ = meta.value()?;

                    let name: LitStr = meta.input.parse()?;

                    reflect.from_string_type = match &*name.value() {
                        FROM_STRING_NONE => FromStringType::None,
                        FROM_STRING_AUTO => FromStringType::AutoFromStr,
                        FROM_STRING_FROM_STR => FromStringType::FromStr,
                        _ => return Err(meta.error("Invalid rename type")),
                    }
                }
                else if meta.path.is_ident(RENAME_ATTRIBUTE) {
                    if global { return Err(meta.error("rename is not valid on container type")); }

                    let _ = meta.value()?;

                    let name: LitStr = meta.input.parse()?;

                    reflect.rename = Some(name);
                }
                else if meta.path.is_ident(ALIAS_ATTRIBUTE) {
                    if global { return Err(meta.error("alias is not valid on container type")); }

                    let aliases;
                    parenthesized!(aliases in meta.input);

                    let aliases: Punctuated<LitStr, Token![,]> = aliases.call(Punctuated::parse_separated_nonempty)?;

                    reflect.aliases.extend(aliases);
                }
                else if meta.path.is_ident(FLATTEN_ATTRIBUTE) {
                    // meta.
                }
                else {
                    return Err(meta.error("Invalid attribute"))
                }

                Ok(())
            })?;
        }

        Ok(reflect)
    }
}

pub fn derive(derive: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let type_name = &derive.ident;

    let global_attributes = try_error!(ReflectAttributes::parse_from_attrs(derive.attrs.as_slice(), true));

    let (impl_generics, ty_generics, where_clause) = derive.generics.split_for_impl();

    let mut get_impl = proc_macro2::TokenStream::default();
    let mut get_mut_impl = proc_macro2::TokenStream::default();
    let mut insert_impl = proc_macro2::TokenStream::default();
    let mut iter_impl = proc_macro2::TokenStream::default();
    let mut iter_fields = proc_macro2::TokenStream::default();
    let mut iter_mut_impl = proc_macro2::TokenStream::default();
    let mut iter_mut_fields = proc_macro2::TokenStream::default();

    let mut variant_name_helper = proc_macro2::TokenStream::default();

    let duplicate = if global_attributes.dont_clone {
        quote!(None)
    } else {
        quote!(Some(Box::new(self.clone())))
    };

    let mut unit_fields = Vec::new();

    if !global_attributes.skip {
        match &derive.data {
            Data::Struct(DataStruct { fields, .. }) => {
                match fields {
                    Fields::Named(fields) => {
                        for field in fields.named.iter() {
                            let field_attributes = try_error!(ReflectAttributes::parse_from_attrs(field.attrs.as_slice(), false));

                            if field_attributes.skip { continue; }

                            let field_name = &field.ident.as_ref().unwrap();

                            let name = match field_attributes.rename {
                                Some(n) => n.to_token_stream(),
                                None => quote!{ stringify!(#field_name) },
                            };

                            let paths: Vec<_> = Some(name).into_iter()
                                .chain(field_attributes.aliases.iter()
                                    .map(|alias| alias.to_token_stream())
                                ).collect();

                            get_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#field_name.impl_get(path),
                                )*
                            }));

                            get_mut_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#field_name.impl_get_mut(path),
                                )*
                            }));

                            insert_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#field_name.impl_insert(path, value),
                                )*
                            }));

                            iter_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#field_name.impl_iter(path),
                                )*
                            }));

                            iter_fields.extend(Some(quote! {
                                self.#field_name.as_dyn(),
                            }));

                            iter_mut_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#field_name.impl_iter_mut(path),
                                )*
                            }));

                            iter_mut_fields.extend(Some(quote! {
                                self.#field_name.as_dyn_mut(),
                            }));
                        }
                    },
                    Fields::Unnamed(fields) => {
                        for (i, field) in fields.unnamed.iter().enumerate() {
                            let i = Index::from(i);

                            let field_attributes = try_error!(ReflectAttributes::parse_from_attrs(field.attrs.as_slice(), false));

                            if field_attributes.skip { continue; }

                            let name = match field_attributes.rename {
                                Some(n) => n.to_token_stream(),
                                None => quote!{ stringify!(#i) },
                            };

                            let paths: Vec<_> = Some(name).into_iter()
                                .chain(field_attributes.aliases.iter()
                                    .map(|alias| alias.to_token_stream())
                                ).collect();

                            get_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#i.impl_get(path),
                                )*
                            }));

                            get_mut_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#i.impl_get_mut(path),
                                )*
                            }));

                            insert_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#i.impl_insert(path, value),
                                )*
                            }));

                            iter_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#i.impl_iter(path),
                                )*
                            }));

                            iter_fields.extend(Some(quote! {
                                self.#i.as_dyn(),
                            }));

                            iter_mut_impl.extend(Some(quote! {
                                #(
                                    Some(#paths) => self.#i.impl_iter_mut(path),
                                )*
                            }));

                            iter_mut_fields.extend(Some(quote! {
                                self.#i.as_dyn_mut(),
                            }));
                        }
                    },
                    Fields::Unit => {},
                }

                iter_fields = quote!{ vec![#iter_fields] };
                iter_mut_fields = quote!{ vec![#iter_mut_fields] };
            },

            Data::Enum(e) => {
                for variant in e.variants.iter() {
                    let variant_name = &variant.ident;
                    let fields = &variant.fields;

                    let variant_attributes = try_error!(ReflectAttributes::parse_from_attrs(variant.attrs.as_slice(), false));

                    if variant_attributes.skip { continue; }

                    let mut pattern: proc_macro2::TokenStream = Default::default();
                    let mut pattern_omitted: proc_macro2::TokenStream = Default::default();

                    let mut get_impl2: proc_macro2::TokenStream = Default::default();
                    let mut get_mut_impl2: proc_macro2::TokenStream = Default::default();
                    let mut insert_impl2: proc_macro2::TokenStream = Default::default();
                    let mut iter_impl2: proc_macro2::TokenStream = Default::default();
                    let mut iter_fields2: proc_macro2::TokenStream = Default::default();
                    let mut iter_mut_impl2: proc_macro2::TokenStream = Default::default();
                    let mut iter_mut_fields2: proc_macro2::TokenStream = Default::default();

                    match fields {
                        Fields::Named(fields) => {
                            let p = fields.named.iter()
                                .map(|field| &field.ident);

                            pattern = quote! {
                                { #(#p),* , .. }
                            };

                            pattern_omitted = quote! { { ..} };

                            for field in fields.named.iter() {
                                let field_attributes = try_error!(ReflectAttributes::parse_from_attrs(field.attrs.as_slice(), false));

                                if field_attributes.skip { continue; }

                                let field_name = &field.ident;

                                let name = match field_attributes.rename {
                                    Some(n) => n.to_token_stream(),
                                    None => quote!{ stringify!(#field_name) },
                                };

                                let paths: Vec<_> = Some(name).into_iter()
                                    .chain(field_attributes.aliases.iter()
                                        .map(|alias| alias.to_token_stream())
                                    ).collect();

                                get_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => #field_name.impl_get(path),
                                    )*
                                }));

                                get_mut_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => if let Self::#variant_name { #field_name, .. } = s { #field_name.impl_get_mut(path) } else { unreachable!(); },
                                    )*
                                }));

                                insert_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => #field_name.impl_insert(path, value),
                                    )*
                                }));

                                iter_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => #field_name.impl_iter(path),
                                    )*
                                }));

                                iter_fields2.extend(Some(quote! {
                                    #field_name.as_dyn(),
                                }));

                                iter_mut_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => #field_name.impl_iter_mut(path),
                                    )*
                                }));

                                iter_mut_fields2.extend(Some(quote! {
                                    #field_name.as_dyn_mut(),
                                }));
                            }
                        },
                        Fields::Unnamed(fields) => {
                            let p = (0..fields.unnamed.len())
                                .map(|i| format_ident!("f{}", i));

                            pattern = quote! {
                                ( #(#p),* , .. )
                            };

                            pattern_omitted = quote! { ( .. ) };

                            for (i, field) in fields.unnamed.iter().enumerate() {
                                let i = Index::from(i);

                                let ident = format_ident!("f{}", i);

                                let field_attributes = try_error!(ReflectAttributes::parse_from_attrs(field.attrs.as_slice(), false));

                                if field_attributes.skip { continue; }

                                let name = match field_attributes.rename {
                                    Some(n) => n.to_token_stream(),
                                    None => quote!{ stringify!(#i) },
                                };

                                let paths: Vec<_> = Some(name).into_iter()
                                    .chain(field_attributes.aliases.iter()
                                        .map(|alias| alias.to_token_stream())
                                    ).collect();

                                get_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => #ident.impl_get(path),
                                    )*
                                }));

                                get_mut_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => if let Self::#variant_name(#ident, ..) = s { #ident.impl_get_mut(path) } else { unreachable!(); },
                                    )*
                                }));

                                insert_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => #ident.impl_insert(path, value),
                                    )*
                                }));

                                iter_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => #ident.impl_iter(path),
                                    )*
                                }));

                                iter_fields2.extend(Some(quote! {
                                    #ident.as_dyn(),
                                }));

                                iter_mut_impl2.extend(Some(quote! {
                                    #(
                                        Some(#paths) => #ident.impl_iter_mut(path),
                                    )*
                                }));

                                iter_mut_fields2.extend(Some(quote! {
                                    #ident.as_dyn_mut(),
                                }));
                            }
                        },
                        Fields::Unit => {
                            unit_fields.push(variant_name)
                        },
                    }

                    let name = match variant_attributes.rename {
                        Some(n) => n.to_token_stream(),
                        None => quote!{ stringify!(#variant_name) },
                    };

                    let paths: Vec<_> = Some(name).into_iter()
                        .chain(variant_attributes.aliases.iter()
                            .map(|alias| alias.to_token_stream())
                        ).collect();

                    variant_name_helper.extend(Some(quote! {
                        Self::#variant_name #pattern_omitted => stringify!(#variant_name),
                    }));

                    get_impl.extend(Some(quote! {
                        #(
                            Some(#paths) => match self {
                                Self::#variant_name #pattern => match path.next() {
                                    None => Ok(self.as_dyn().into()),
                                    #get_impl2
                                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                                },
                                other => Err(ReflectError::wrong_variant(other.__reflect_variant_name(), stringify!(#variant_name))),
                            }
                        )*
                    }));

                    get_mut_impl.extend(Some(quote! {
                        #(
                            Some(#paths) => match self {
                                s @ Self::#variant_name #pattern_omitted => match path.next() {
                                    None => Ok(s.as_dyn_mut()),
                                    #get_mut_impl2
                                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                                },
                                other => Err(ReflectError::wrong_variant(other.__reflect_variant_name(), stringify!(#variant_name))),
                            }
                        )*
                    }));

                    insert_impl.extend(Some(quote! {
                        #(
                            Some(#paths) => match self {
                                Self::#variant_name #pattern => match path.next() {
                                    None => Err(ReflectError::entry_not_exist(stringify!(#variant_name))),
                                    #insert_impl2
                                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                                },
                                other => Err(ReflectError::wrong_variant(other.__reflect_variant_name(), stringify!(#variant_name))),
                            }
                        )*
                    }));


                    let iter_self = if iter_fields2.is_empty() {
                        quote! {
                            ReflectIter::default()
                        }
                    } else {
                        quote! {
                            ReflectIter::new(
                                vec![#iter_fields2]
                                .into_iter()
                                .enumerate()
                                .map(|(k, v)| ReflectIterEntry {
                                    item: v,
                                    index: Some(ReflectItemIndex::Number(k))
                                }
                            ))
                        }
                    };

                    iter_impl.extend(Some(quote! {
                        #(
                            Some(#paths) => match self {
                                Self::#variant_name #pattern => match path.next() {
                                    // None => Ok(vec![#iter_fields2].into()),
                                    None => Ok(#iter_self),

                                    #iter_impl2
                                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                                },
                                other => Err(ReflectError::wrong_variant(other.__reflect_variant_name(), stringify!(#variant_name))),
                            }
                        )*
                    }));

                    iter_fields.extend(Some(quote! {
                        Self::#variant_name #pattern => vec![#iter_fields2],
                    }));

                    let iter_mut_self = if iter_mut_fields2.is_empty() {
                        quote! {
                            ReflectIterMut::default()
                        }
                    } else {
                        quote! {
                            ReflectIterMut::new(vec![#iter_mut_fields2]
                                .into_iter()
                                .enumerate()
                                .map(|(k, v)| ReflectIterMutEntry {
                                    item: v,
                                    index: Some(ReflectItemIndex::Number(k))
                                }
                            ))
                        }
                    };

                    iter_mut_impl.extend(Some(quote! {
                        #(
                            Some(#paths) => match self {
                                Self::#variant_name #pattern => match path.next() {
                                    // None => Ok(vec![#iter_mut_fields2].into()),
                                    None => Ok(#iter_mut_self),

                                    #iter_mut_impl2
                                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                                },
                                other => Err(ReflectError::wrong_variant(other.__reflect_variant_name(), stringify!(#variant_name))),
                            }
                        )*
                    }));

                    iter_mut_fields.extend(Some(quote! {
                        Self::#variant_name #pattern => vec![#iter_mut_fields2],
                    }));
                }

                iter_fields = quote!{
                    match self {
                        #iter_fields
                        // Skipped variants
                        _ => vec![],
                    }
                };
                iter_mut_fields = quote!{
                    match self {
                        #iter_mut_fields
                        // Skipped variants
                        _ => vec![],
                    }
                };

                variant_name_helper = quote! {
                    impl #impl_generics #type_name #ty_generics where #where_clause {
                        #[doc(hidden)]
                        fn __reflect_variant_name(&self) -> &'static str {
                            match self {
                                #variant_name_helper
                                // Skipped variants
                                _ => unreachable!("skipped variant"),
                            }
                        }
                    }
                }
            },
            Data::Union(_) => unimplemented!("unions bad"),
        }
    } else {
        iter_fields = quote! { vec![self.as_dyn()] };
        iter_mut_fields = quote! { vec![self.as_dyn_mut()] };
    }


    let mut from_str_impl = proc_macro2::TokenStream::default();
    let from_string = match global_attributes.from_string_type {
        FromStringType::None => quote!(Err(ReflectError::NoFromString)),
        FromStringType::FromStr
        | FromStringType::AutoFromStr => quote!(
            use std::str::FromStr; 
            Ok(Box::new(Self::from_str(_s)?))
        ),
    };
    let display_impl = match global_attributes.display {
        DisplayType::None => quote! { return Ok("No Reflect Display".to_string()); },
        DisplayType::Display => quote! { return Ok(format!("{self}")); },
        DisplayType::Debug => quote! { return Ok(format!("{self:?}")); },
    };


    if let FromStringType::AutoFromStr = global_attributes.from_string_type {
        from_str_impl = quote!(
            impl #impl_generics std::str::FromStr for #type_name #ty_generics where #where_clause {
                type Err = ReflectError<'static>;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        #(stringify!(#unit_fields) => Ok(Self::#unit_fields),)*
                        other => Err(ReflectError::entry_not_exist(other).to_owned())
                    }
                }
            }
            impl #impl_generics Stringable for #type_name #ty_generics where #where_clause {
                type Err = ReflectError<'static>;

                fn parse_str(s: &str) -> Result<Self, Self::Err> {
                    s.parse()
                }
            }
        );
    }


    quote! {
        #variant_name_helper

        impl #impl_generics Reflect for #type_name #ty_generics where #where_clause {
            fn impl_get<'a, 's>(&'s self, mut path: ReflectPath<'a>) -> ReflectResult<'a, MaybeOwnedReflect<'s>> {
                match path.next() {
                    None => Ok(self.as_dyn().into()),
                    #get_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }

            fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, &mut dyn Reflect> {
                match path.next() {
                    None => Ok(self.as_dyn_mut()),
                    #get_mut_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }

            fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> ReflectResult<'a, ()> {
                // println!("{}", std::any::type_name_of_val(&value));

                match path.next() {
                    None => {
                        value.downcast::<Self>().map(|v| *self = *v)
                            .or_else(|v| v.downcast::<Box<dyn Reflect>>().and_then(|v| v.downcast::<Self>().map(|v| *self = *v)))
                            .map_err(|v| ReflectError::wrong_type(std::any::type_name::<Self>(), v.type_name()))
                    },
                    #insert_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }

            fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIter<'_>> {
                match path.next() {
                    // None => Ok(#iter_fields.into()),
                    None => Ok(ReflectIter::new(
                        #iter_fields.into_iter()
                        .enumerate()
                        .map(|(k, v)| ReflectIterEntry {
                            item: v,
                            index: Some(ReflectItemIndex::Number(k))
                        })
                    )),
                    #iter_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }
            fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> ReflectResult<'a, ReflectIterMut<'_>> {
                match path.next() {
                    // None => Ok(#iter_mut_fields.into()),
                    None => Ok(ReflectIterMut::new(#iter_mut_fields
                        .into_iter()
                        .enumerate()
                        .map(|(k, v)| ReflectIterMutEntry {
                            item: v,
                            index: Some(ReflectItemIndex::Number(k))
                        })
                    )),
                    #iter_mut_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }

            fn duplicate(&self) -> Option<Box<dyn Reflect>> {
                #duplicate
            }

            fn from_string(_s: &str) -> ReflectResult<Box<dyn Reflect>> {
                #from_string
            }

            fn impl_display<'a>(&self, path: ReflectPath<'a>, precision: Option<usize>) -> ReflectResult<'a, String> {
                if !path.has_next() { #display_impl }
                match self.impl_get(path)? {
                    MaybeOwnedReflect::Borrowed(reflect) => reflect.reflect_display(ReflectPath::new(""), precision),
                    MaybeOwnedReflect::Owned(reflect) => reflect.reflect_display(ReflectPath::new(""), precision),
                }
            }
        }

        #from_str_impl
    }
}
