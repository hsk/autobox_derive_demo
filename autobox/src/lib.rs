use proc_macro::TokenStream as TS;
use proc_macro2::TokenStream as TS2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input,parse_quote,parse_str,Error,DeriveInput,Type};

#[proc_macro_derive(AutoBox,attributes(autobox))]
pub fn autobox_derive(input: TS) -> TS {
    let input:&DeriveInput = &parse_macro_input!(input);
    match gen(input) {
        Ok(generated) => generated,
        Err(err) => err.to_compile_error().into(),
    }
}

fn get_option(item:& DeriveInput) -> bool {
    use darling::{FromDeriveInput};
    #[derive(Debug, FromDeriveInput)]
    #[darling(attributes(autobox))]
    struct AutoBoxOption {
        #[darling(default)]
        uppercase: bool,
    }
    match AutoBoxOption::from_derive_input(item) {
        Ok(option) => option.uppercase,
        Err(_) => false,
    }
}

fn checkbox0(ty:Type)->Result<String,()> {
    let syn::Type::Path(syn::TypePath{path:syn::Path{segments,..},..})=ty else { return Err(()) };
    let syn::PathSegment{ident,
            arguments: syn::PathArguments::AngleBracketed(
                syn::AngleBracketedGenericArguments {args,..})}
                = &segments[0] else { return Err(()) };
    if ident.to_string().as_str() != "Box" { return Err(()) }
    let str = args[0].to_token_stream().to_string();
    Ok(str)
}

fn checkbox1(ty:Type)->Result<Type,()> {
    let s = checkbox0(ty)?;
    Ok(parse_str(s.as_str()).unwrap())
}

fn checkbox(s:&str)->Result<String,()> {
    let Ok(s) = parse_str(s) else { return Err(()) };
    checkbox0(s)
}

fn cnv_input(sup: String, ty: Type)->String {
    use syn::visit_mut::{VisitMut,visit_type_mut};
    struct CnvInput { sup:String }
    impl VisitMut for CnvInput {
        fn visit_type_mut(&mut self, node: &mut Type) {
            visit_type_mut(self,node);
            let s = node.to_token_stream().to_string();
            if s == "String" {
                *node = parse_quote!{&str};
            } else
            if s == self.sup {
                *node = parse_quote!{super:: #node};
            } else
            if let Ok(ty) = checkbox1(node.clone()) {
                *node = ty;
            }
        }
    }
    let mut ty = ty;
    CnvInput{sup}.visit_type_mut(&mut ty);
    ty.to_token_stream().to_string()
}

fn cnv_input2(s:String)->String {
    let Ok(ty) = checkbox(s.as_str()) else { return s };
    ty
}

/*
fn cnv_output1(sup: String, ty: Type)->String {
    use syn::visit_mut::{VisitMut,visit_type_mut};
    struct CnvOutput {
        sup:String
    }
    impl VisitMut for CnvOutput {
        fn visit_type_mut(&mut self, node: &mut Type) {
            visit_type_mut(self,node);
            let s = node.to_token_stream().to_string();
            if s == "String" {
                *node = parse_quote!{&str};
            } else
            if s == self.sup {
                *node = parse_quote!{super:: #node};
            } else
            if let Ok(ty) = checkbox1(node.clone()) {
                *node = ty;
            }
        }
    }
    let mut ty = ty;
    CnvOutput{sup}.visit_type_mut(&mut ty);
    ty.to_token_stream().to_string()
}
*/
fn cnv_output(v:String,ty:String)->String {
    if ty == "String" { return format!("{}.to_string()",v) } 
    if let Ok(ty) = checkbox(ty.as_str()) {
        return format!("Box::new({})",cnv_output(v,ty))
    }
    v
}

fn gen(derive_input: &DeriveInput) -> Result<TS, Error> {
    match &derive_input.data {
        syn::Data::Struct(v) => gen_struct(derive_input,v),
        syn::Data::Enum(v) => gen_enum(derive_input,v),
        e => Err(Error::new_spanned(
                &derive_input.ident,
                format!("Must be struct type {:?}",e),
            )),
    }
}

fn gen_fun_name(derive_input: &DeriveInput, text:String)->String {
    if get_option(derive_input) {
        return text
    }
    if text.len() == 1 {
        text.to_lowercase()
    } else {
        let i = text.char_indices().nth(1).unwrap().0;
        let ch1 = (&text[..i]).to_lowercase();
        let ch2 = &text[i..];
        format!("{}{}",ch1,ch2)   
    }
}

fn gen_struct(derive_input: &DeriveInput, struct_data:&syn::DataStruct)-> Result<TS, Error> {
    let mut args = Vec::new();
    let mut ps = Vec::new();
    let mut un = false;
    let name = &derive_input.ident;
    for (i,field) in struct_data.fields.iter().enumerate() {
        let ty = field.ty.to_token_stream().to_string();
        let ty1:TS2 = cnv_input(name.to_string(),field.ty.clone()).parse().unwrap();
        let id = match &field.ident {
            Some(v) => v.to_string(),
            None => {un=true;format!("v{}",i)},
        };
        let v:TS2 = cnv_output(id.clone(),ty.clone()).parse().unwrap();
        let id:TS2 = id.parse().unwrap();
        args.push(quote!{#id : #ty1});
        ps.push(if un {quote!{#v}} else {quote!{#id: #v}})
    }
    let fun_name:TS2 = gen_fun_name(derive_input,name.to_string()).parse().unwrap();
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
                pub fn #fun_name (#(#args,)*) -> super::#name {
                    super::#name #ps
                }
            }
        }
    };
    //println!("{}",gen);
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

fn gen_enum(derive_input: &DeriveInput, data:&syn::DataEnum)-> Result<TS, Error> {
    let name = &derive_input.ident;
    let mod_name:TS2 = name.to_string().to_lowercase().parse().unwrap();
    let mut bnfs = vec![];
    let funs = data.variants.iter().map(|v:&syn::Variant| {
        let vid = &v.ident;
        let fun_name:TS2 = gen_fun_name(derive_input, vid.to_string()).parse().unwrap();
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
            let ty2:TS2 = cnv_input2(ty).parse().unwrap();
            let ty:TS2 = cnv_input(name.to_string(),field.ty.clone()).parse().unwrap();
            args.push(quote!{#id : #ty});
            params.push(if un {quote!{#v}} else {quote!{#id:#v}});
            params2.push(if un {quote!{#ty2}} else {quote!{#id:#ty2}});
        }
        if params.len() == 0 {
            let d:TS2 = format!("{}",doc(&v.attrs)).parse().unwrap();
            bnfs.push((format!("{}",quote!{#fun_name}),format!("{}",quote!{#d})));
            quote!{ pub const #fun_name: #name = #name::#vid; }
        } else {
            let ps2 = if un {quote!{(#(#params2),*)}} else {quote!{{#(#params2),*}}};
            let d:TS2 = format!("{}",doc(&v.attrs)).parse().unwrap();
            bnfs.push((format!("{}",quote!{#fun_name #ps2}),format!("{}",quote!{#d})));
            let ps = if un {quote!{(#(#params),*)}} else {quote!{{#(#params),*}}};
            quote!{ pub fn #fun_name (#(#args),*) -> #name { #name::#vid #ps } }
        }
    });
    let funs = quote!{#(#funs)*};
    let bnf:TS2 = {
        let mut bnf = String::new();
        use std::fmt::Write;
        writeln!(bnf,"{:<33}{}",format!("{} ::=",name.to_string()),doc(&derive_input.attrs)).unwrap();
        for (x,com) in bnfs.iter() {
            writeln!(bnf,"  {:<30} {}",x,com).unwrap();
        }
        format!("{:?}",bnf).parse().unwrap()
    };
    let gen = quote!{
        pub mod #mod_name {
            use super:: #name;
            pub const bnflike: &str = #bnf;
            #funs
        }
    };
    //println!("gen {}",gen);
    Ok(gen.into())
}
