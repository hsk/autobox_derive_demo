use proc_macro::TokenStream as TS;
use proc_macro2::TokenStream as TS2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(AutoBox)]
pub fn autobox_derive(input: TS) -> TS {
    let input = &parse_macro_input!(input as DeriveInput);
    match gen(input) {
        Ok(generated) => generated,
        Err(err) => err.to_compile_error().into(),
    }
}

fn checkbox(s:&str)->Option<String> {
    let re = regex::Regex::new(r"^Box < (.+) >$").unwrap();
    match re.captures(s) {
        Some(cap) => Some(cap[1].to_string()),
        None => None,
    }
}
fn cnv_input(s:String)->String {
    if s == "String" {
        "&str".to_string()
    } else if let Some(ty) = checkbox(s.as_str()) {
        cnv_input(ty)
    } else {
        s
    }
}
fn cnv_input2(s:String)->String {
    if let Some(ty) = checkbox(s.as_str()) {
        cnv_input2(ty)
    } else {
        s
    }
}
fn cnv_output(v:String,s:String)->String {
    if s == "String" {
        format!("{}.to_string()",v)
    } else if let Some(ty) = checkbox(s.as_str()) {
        format!("Box::new({})",cnv_output(v,ty))
    } else {
        v
    }
}

fn gen(derive_input: &DeriveInput) -> Result<TS, syn::Error> {
    match &derive_input.data {
        syn::Data::Struct(v) => gen_struct(derive_input,v),
        syn::Data::Enum(v) => gen_enum(derive_input,v),
        e => Err(syn::Error::new_spanned(
                &derive_input.ident,
                format!("Must be struct type {:?}",e),
            )),
    }
}

fn gen_fun_name(text:String)->String {
    if text.len() == 1 {
        text.to_lowercase()
    } else {
        let i = text.char_indices().nth(1).unwrap().0;
        let ch1 = (&text[..i]).to_lowercase();
        let ch2 = &text[i..];
        format!("{}{}",ch1,ch2)   
    }
}

fn gen_struct(derive_input: &DeriveInput, struct_data:&syn::DataStruct)-> Result<TS, syn::Error> {
    let mut args = Vec::new();
    let mut ps = Vec::new();
    let mut un = false;
    for (i,field) in struct_data.fields.iter().enumerate() {
        let ty = field.ty.to_token_stream().to_string();
        let ty1:TS2 = cnv_input(ty.clone()).parse().unwrap();
        let id = match &field.ident.as_ref() {
            Some(v) => v.to_string(),
            None => {un=true;format!("v{}",i)},
        };
        let v:TS2 = cnv_output(id.clone(),ty).parse().unwrap();
        let id:TS2 = id.parse().unwrap();
        args.push(quote!{#id : #ty1});
        ps.push(if un {quote!{#v}} else {quote!{#id: #v}})
    }
    let name = &derive_input.ident;
    let fun_name:TS2 = gen_fun_name(name.to_string()).parse().unwrap();
    let mod_name:TS2 = name.to_string().to_lowercase().parse().unwrap();
    let gen = if ps.len() == 0 {
        quote! {
            mod #mod_name {
                use super:: #name;
                pub const #fun_name : #name = #name{};
            }
        }
    } else {
        let ps = if un {quote!{(#(#ps,)*)}} else {quote!{{#(#ps,)*}}};
        quote! {
            mod #mod_name {
                use super:: #name;
                pub fn #fun_name (#(#args,)*) -> #name {
                    #name #ps
                }
            }
        }
    };
    Ok(gen.into())
}
fn doc(attrs:&Vec<syn::Attribute>) -> String {
    if let Some(a) = attrs.iter().find(|attr|attr.path.is_ident("doc")) {
        let lit = &a.tokens.clone().into_iter().collect::<Vec<_>>()[1];
        match litrs::StringLit::try_from(lit) {
            Err(_e) => "error".to_string(),
            Ok(lit) => lit.value().trim().to_string(),
        }

    } else {
        "".to_string()
    }
}

fn gen_enum(derive_input: &DeriveInput, data:&syn::DataEnum)-> Result<TS, syn::Error> {
    let variants = data.variants.iter();
    let name = &derive_input.ident;
    let mut bnfs = vec![];
    let funs = variants.map(|v:&syn::Variant| {
        let vid = &v.ident;
        let fun_name:TS2 = gen_fun_name(vid.to_string()).parse().unwrap();
        let mut args = Vec::new();
        let mut params = Vec::new();
        let mut params2 = Vec::new();
        let mut un = false;
        for (n,field) in v.fields.iter().enumerate() {
            let id:TS2 = match &field.ident.as_ref() {
                Some(v) => v.to_string(),
                None => {un=true;format!("v{}",n)},
            }.parse().unwrap();
            let ty = field.ty.to_token_stream().to_string();
            let v:TS2 = cnv_output(id.to_string(),ty.clone()).parse().unwrap();
            let ty2:TS2 = cnv_input2(ty.clone()).parse().unwrap();
            let ty:TS2 = cnv_input(ty).parse().unwrap();
            args.push(quote!{#id : #ty});
            params.push(if un {quote!{#v}} else {quote!{#id:#v}});
            params2.push(if un {quote!{#ty2}} else {quote!{#id:#ty2}});
        }
        if params.len() == 0 {
            let d:TS2 = format!("{}",doc(&v.attrs)).parse().unwrap();
            bnfs.push((format!("{}",quote!{#fun_name}),format!("{}",quote!{#d})));
            quote! {
                pub const #fun_name: #name = #name::#vid;
            }
        } else {
            let ps2 = if un {quote!{(#(#params2),*)}} else {quote!{{#(#params2),*}}};
            let d:TS2 = format!("{}",doc(&v.attrs)).parse().unwrap();
            bnfs.push((format!("{}",quote!{#fun_name #ps2}),format!("{}",quote!{#d})));
            let ps = if un {quote!{(#(#params),*)}} else {quote!{{#(#params),*}}};
            quote!(
                pub fn #fun_name (#(#args),*) -> #name {
                    #name::#vid #ps
                }
            )
        }
    });
    let mod_name:TS2 = name.to_string().to_lowercase().parse().unwrap();
    let gen = quote! {
        pub mod #mod_name {
            use super:: #name;
            #(#funs)*
        }
    };
    //println!("gen {}",gen);
    println!("{:<33}{}",format!("{} ::=",name),doc(&derive_input.attrs));
    for (bnf,com) in bnfs.iter() {
        println!("  {:<30} {}",bnf,com);
    }

    Ok(gen.into())
}
