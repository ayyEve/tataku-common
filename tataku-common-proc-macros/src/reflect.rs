use quote::*;
use syn::*;

pub const REFLECT_ATTRIBUTE: &str = "reflect";
pub const SKIP_ATTRIBUTE: &str = "skip";

fn skip(field: &Field) -> bool {
    field.attrs.iter().any(|attr|
        attr.path().is_ident(REFLECT_ATTRIBUTE) && {
            attr.parse_args::<Path>().ok()
                .map(|i| i.is_ident(SKIP_ATTRIBUTE)).unwrap_or(false)
        }
    )
}

pub fn derive(derive: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let type_name = &derive.ident;

    let mut get_impl: proc_macro2::TokenStream = Default::default();
    let mut get_mut_impl: proc_macro2::TokenStream = Default::default();
    let mut insert_impl: proc_macro2::TokenStream = Default::default();
    let mut iter_impl: proc_macro2::TokenStream = Default::default();
    let mut iter_fields: proc_macro2::TokenStream = Default::default();
    let mut iter_mut_impl: proc_macro2::TokenStream = Default::default();
    let mut iter_mut_fields: proc_macro2::TokenStream = Default::default();

    match &derive.data {
        Data::Struct(DataStruct { fields, .. }) => {
            match fields {
                Fields::Named(fields) => {
                    for field in fields.named.iter() {
                        if skip(field) { continue; }

                        let field_name = &field.ident;

                        get_impl.extend(Some(quote! {
                            Some(stringify!(#field_name)) => self.#field_name.impl_get(path),
                        }));

                        get_mut_impl.extend(Some(quote! {
                            Some(stringify!(#field_name)) => self.#field_name.impl_get_mut(path),
                        }));

                        insert_impl.extend(Some(quote! {
                            Some(stringify!(#field_name)) => self.#field_name.impl_insert(path, value),
                        }));

                        iter_impl.extend(Some(quote! {
                            Some(stringify!(#field_name)) => self.#field_name.impl_iter(path),
                        }));

                        iter_fields.extend(Some(quote! {
                            &self.#field_name as &dyn Reflect,
                        }));

                        iter_mut_impl.extend(Some(quote! {
                            Some(stringify!(#field_name)) => self.#field_name.impl_iter_mut(path),
                        }));

                        iter_mut_fields.extend(Some(quote! {
                            &mut self.#field_name as &mut dyn Reflect,
                        }));
                    }
                },
                Fields::Unnamed(fields) => {
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        let i = Index::from(i);

                        if skip(field) { continue; }

                        get_impl.extend(Some(quote! {
                            Some(stringify!(#i)) => self.#i.impl_get(path),
                        }));

                        get_mut_impl.extend(Some(quote! {
                            Some(stringify!(#i)) => self.#i.impl_get_mut(path),
                        }));

                        insert_impl.extend(Some(quote! {
                            Some(stringify!(#i)) => self.#i.impl_insert(path, value),
                        }));

                        iter_impl.extend(Some(quote! {
                            Some(stringify!(#i)) => self.#i.impl_iter(path),
                        }));

                        iter_fields.extend(Some(quote! {
                            &self.#i as &dyn Reflect,
                        }));

                        iter_mut_impl.extend(Some(quote! {
                            Some(stringify!(#i)) => self.#i.impl_iter_mut(path),
                        }));

                        iter_mut_fields.extend(Some(quote! {
                            &mut self.#i as &mut dyn Reflect,
                        }));
                    }
                },
                Fields::Unit => {},
            }

            iter_fields = quote!{ vec![#iter_fields] };
            iter_mut_fields = quote!{ vec![#iter_mut_fields] };
        },

        Data::Enum(e) => {
            for (i, variant) in e.variants.iter().enumerate() {
                let variant_name = &variant.ident;
                let fields = &variant.fields;

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
                            if skip(field) { continue; }

                            let field_name = &field.ident;

                            get_impl2.extend(Some(quote! {
                                Some(stringify!(#field_name)) => #field_name.impl_get(path),
                            }));

                            get_mut_impl2.extend(Some(quote! {
                                Some(stringify!(#field_name)) => if let Self::#variant_name { #field_name, .. } = s { #field_name.impl_get_mut(path) } else { unreachable!(); },
                            }));

                            insert_impl2.extend(Some(quote! {
                                Some(stringify!(#field_name)) => #field_name.impl_insert(path, value),
                            }));

                            iter_impl2.extend(Some(quote! {
                                Some(stringify!(#field_name)) => #field_name.impl_iter(path),
                            }));

                            iter_fields2.extend(Some(quote! {
                                #field_name as &dyn Reflect,
                            }));

                            iter_mut_impl2.extend(Some(quote! {
                                Some(stringify!(#field_name)) => #field_name.impl_iter_mut(path),
                            }));

                            iter_mut_fields2.extend(Some(quote! {
                                #field_name as &mut dyn Reflect,
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

                            if skip(field) { continue; }

                            get_impl2.extend(Some(quote! {
                                Some(stringify!(#i)) => #ident.impl_get(path),
                            }));

                            get_mut_impl2.extend(Some(quote! {
                                Some(stringify!(#i)) => if let Self::#variant_name(#ident, ..) = s { #ident.impl_get_mut(path) } else { unreachable!(); },
                            }));

                            insert_impl2.extend(Some(quote! {
                                Some(stringify!(#i)) => #ident.impl_insert(path, value),
                            }));

                            iter_impl2.extend(Some(quote! {
                                Some(stringify!(#i)) => #ident.impl_iter(path),
                            }));

                            iter_fields2.extend(Some(quote! {
                                #ident as &dyn Reflect,
                            }));

                            iter_mut_impl2.extend(Some(quote! {
                                Some(stringify!(#i)) => #ident.impl_iter_mut(path),
                            }));

                            iter_mut_fields2.extend(Some(quote! {
                                #ident as &mut dyn Reflect,
                            }));
                        }
                    },
                    Fields::Unit => {},
                }

                let wrong_variants: Vec<proc_macro2::TokenStream> = e.variants.iter()
                    .enumerate()
                    .filter_map(|(i2, v)| (i != i2).then_some(v))
                    .map(|v| (&v.ident, match &v.fields {
                        Fields::Named(_) => quote! { { .. } },
                        Fields::Unnamed(_) => quote! { (..) },
                        Fields::Unit => quote!{ },
                    }))
                    .map(|(v, pat)| quote! {
                        Self::#v #pat => Err(ReflectError::wrong_variant(stringify!(#variant_name), stringify!(#v)))
                    })
                    .collect();

                get_impl.extend(Some(quote! {
                    Some(stringify!(#variant_name)) => match self {
                        Self::#variant_name #pattern => match path.next() {
                            None => Ok(self as &dyn Reflect),
                            #get_impl2
                            Some(p) => Err(ReflectError::entry_not_exist(p)),
                        },
                        #(#wrong_variants),*
                    }
                }));

                get_mut_impl.extend(Some(quote! {
                    Some(stringify!(#variant_name)) => match self {
                        s @ Self::#variant_name #pattern_omitted => match path.next() {
                            None => Ok(s as &mut dyn Reflect),
                            #get_mut_impl2
                            Some(p) => Err(ReflectError::entry_not_exist(p)),
                        },
                        #(#wrong_variants),*
                    }
                }));

                insert_impl.extend(Some(quote! {
                    Some(stringify!(#variant_name)) => match self {
                        Self::#variant_name #pattern => match path.next() {
                            None => Err(ReflectError::entry_not_exist(stringify!(#variant_name))),
                            #insert_impl2
                            Some(p) => Err(ReflectError::entry_not_exist(p)),
                        },
                        #(#wrong_variants),*
                    }
                }));

                iter_impl.extend(Some(quote! {
                    Some(stringify!(#variant_name)) => match self {
                        Self::#variant_name #pattern => match path.next() {
                            None => Ok(vec![#iter_fields2].into()),
                            #iter_impl2
                            Some(p) => Err(ReflectError::entry_not_exist(p)),
                        },
                        #(#wrong_variants),*
                    }
                }));

                iter_fields.extend(Some(quote! {
                    Self::#variant_name #pattern => vec![#iter_fields2],
                }));

                iter_mut_impl.extend(Some(quote! {
                    Some(stringify!(#variant_name)) => match self {
                        Self::#variant_name #pattern => match path.next() {
                            None => Ok(vec![#iter_mut_fields2].into()),
                            #iter_mut_impl2
                            Some(p) => Err(ReflectError::entry_not_exist(p)),
                        },
                        #(#wrong_variants),*
                    }
                }));

                iter_mut_fields.extend(Some(quote! {
                    Self::#variant_name #pattern => vec![#iter_mut_fields2],
                }));
            }

            iter_fields = quote!{ match self { #iter_fields }};
            iter_mut_fields = quote!{ match self { #iter_mut_fields }};
        },
        Data::Union(_) => unimplemented!("unions bad"),
    }

    quote! {
        impl Reflect for #type_name {
            fn impl_get<'a>(&self, mut path: ReflectPath<'a>) -> Result<&dyn Reflect, ReflectError<'a>> {
                match path.next() {
                    None => Ok(self as &dyn Reflect),
                    #get_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }

            fn impl_get_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<&mut dyn Reflect, ReflectError<'a>> {
                match path.next() {
                    None => Ok(self as &mut dyn Reflect),
                    #get_mut_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }

            fn impl_insert<'a>(&mut self, mut path: ReflectPath<'a>, value: Box<dyn Reflect>) -> Result<(), ReflectError<'a>> {
                match path.next() {
                    None => value.downcast::<Self>().map(|v| *self = *v)
                        .map_err(|_| ReflectError::wrong_type(std::any::type_name::<Self>(), "TODO: cry")),
                    #insert_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }

            fn impl_iter<'a>(&self, mut path: ReflectPath<'a>) -> Result<IterThing<'_>, ReflectError<'a>> {
                match path.next() {
                    None => Ok(#iter_fields.into()),
                    #iter_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }
            fn impl_iter_mut<'a>(&mut self, mut path: ReflectPath<'a>) -> Result<IterThingMut<'_>, ReflectError<'a>> {
                match path.next() {
                    None => Ok(#iter_mut_fields.into()),
                    #iter_mut_impl
                    Some(p) => Err(ReflectError::entry_not_exist(p)),
                }
            }
        }
    }
}