//! Rendering of generated enums and value structs through Askama templates.
//!
//! Public value types are intentionally emitted one file per type. The module
//! index is rendered separately and re-exports the generated items flat from
//! the crate root.

use std::collections::BTreeSet;

use crate::r#rust::formatter::{
    enum_variant_name, field_name, render_docs, type_def_name, type_module_name,
};
use crate::r#rust::gen::method::render_method;
use crate::r#rust::lifetimes::{struct_generics, struct_lifetime_phantom};
use crate::r#rust::type_map::{safe_struct_field_type, safe_value_type, struct_needs_abi_mirror};
use askama::Template;
use diplomat_core::hir::{
    self, DocsUrlGenerator, MaybeOwn, Mutability, Slice, StructPathLike, Type, TypeContext, TypeDef,
};

#[derive(Template)]
#[template(path = "rust/types_mod.rs.jinja", escape = "none")]
struct TypesModTemplate<'a> {
    modules: &'a [String],
    has_modules: bool,
}

#[derive(Template)]
#[template(path = "rust/enum.rs.jinja", escape = "none")]
struct EnumTemplate {
    docs: String,
    imports: Vec<String>,
    has_ffi: bool,
    name: String,
    variants: Vec<EnumVariantView>,
    methods: Vec<String>,
}

#[derive(Template)]
#[template(path = "rust/struct.rs.jinja", escape = "none")]
struct StructTemplate {
    docs: String,
    imports: Vec<String>,
    has_ffi: bool,
    has_phantom: bool,
    name: String,
    generics: String,
    args: String,
    needs_abi_mirror: bool,
    derives: String,
    fields: Vec<FieldView>,
    phantom: String,
    methods: Vec<String>,
}

struct EnumVariantView {
    docs: String,
    name: String,
    discriminant: String,
}

struct FieldView {
    docs: String,
    name: String,
    ty: String,
}

/// Generate `types/mod.rs` and one source file for every enabled enum/struct.
pub(crate) fn generate_type_files(
    tcx: &TypeContext,
    docs_url_gen: &DocsUrlGenerator,
) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut modules = Vec::new();

    for enm in tcx.enums().iter().filter(|ty| !ty.attrs.disable) {
        let module = type_module_name(TypeDef::Enum(enm));
        modules.push(module.clone());
        files.push((
            format!("src/types/{module}.rs"),
            render_enum(enm, tcx, docs_url_gen),
        ));
    }
    for strct in tcx.structs().iter().filter(|ty| !ty.attrs.disable) {
        let module = type_module_name(TypeDef::Struct(strct));
        modules.push(module.clone());
        files.push((
            format!("src/types/{module}.rs"),
            render_struct(strct, tcx, docs_url_gen),
        ));
    }

    let index = TypesModTemplate {
        modules: &modules,
        has_modules: !modules.is_empty(),
    }
    .render()
    .expect("Rust types module template rendering cannot fail");
    files.insert(0, ("src/types/mod.rs".into(), index));
    files
}

fn render_enum(enm: &hir::EnumDef, tcx: &TypeContext, docs_url_gen: &DocsUrlGenerator) -> String {
    let name = type_def_name(TypeDef::Enum(enm));
    let methods = enm
        .methods
        .iter()
        .filter(|method| !method.attrs.disable)
        .map(|method| render_method(0, method, tcx, docs_url_gen))
        .collect::<Vec<_>>();
    let imports = referenced_names_for_methods(&enm.methods, tcx, Some(&name));
    let variants = enm
        .variants
        .iter()
        .map(|variant| EnumVariantView {
            docs: render_docs(&variant.docs, docs_url_gen, "    "),
            name: enum_variant_name(variant),
            discriminant: variant.discriminant.to_string(),
        })
        .collect();

    EnumTemplate {
        docs: render_docs(&enm.docs, docs_url_gen, ""),
        imports,
        has_ffi: !methods.is_empty(),
        name,
        variants,
        methods,
    }
    .render()
    .expect("Rust enum template rendering cannot fail")
}

fn render_struct(
    strct: &hir::StructDef,
    tcx: &TypeContext,
    docs_url_gen: &DocsUrlGenerator,
) -> String {
    let name = type_def_name(TypeDef::Struct(strct));
    let methods = strct
        .methods
        .iter()
        .filter(|method| !method.attrs.disable)
        .map(|method| render_method(strct.lifetimes.num_lifetimes(), method, tcx, docs_url_gen))
        .collect::<Vec<_>>();
    let fields = strct
        .fields
        .iter()
        .map(|field| FieldView {
            docs: render_docs(&field.docs, docs_url_gen, "    "),
            name: field_name(field),
            ty: if struct_needs_abi_mirror(strct, tcx) {
                safe_struct_field_type(&field.ty, strct, tcx)
            } else {
                safe_value_type(&field.ty, tcx)
            },
        })
        .collect();
    let phantom = struct_lifetime_phantom(strct);
    let (generics, args) = struct_generics(strct);
    let imports = referenced_names_for_struct(strct, tcx, &name);

    StructTemplate {
        docs: render_docs(&strct.docs, docs_url_gen, ""),
        imports,
        has_ffi: !methods.is_empty(),
        has_phantom: !phantom.is_empty(),
        name,
        generics,
        args,
        needs_abi_mirror: struct_needs_abi_mirror(strct, tcx),
        derives: derives(strct, tcx),
        fields,
        phantom,
        methods,
    }
    .render()
    .expect("Rust struct template rendering cannot fail")
}

/// Which of the derived traits a generated value struct can carry.
///
/// The template derives from the field types, and the field rules accept shapes for
/// which some of them do not hold: a borrowed *mutable* slice (`DiplomatSliceMut` in the
/// bridge, `&mut [T]` in the public API) is `PartialEq` and `Eq` but neither `Clone` nor
/// `Copy`, and a float — on its own, inside `Option`, or as a slice element — is not
/// `Eq`. Deriving them anyway produced a struct rustc rejects:
///
/// ```text
/// error[E0204]: the trait `Copy` cannot be implemented for this type
/// error[E0277]: the trait bound `f64: Eq` is not satisfied
/// ```
#[derive(Clone, Copy)]
struct FieldTraits {
    copy: bool,
    clone: bool,
    eq: bool,
}

impl FieldTraits {
    const ALL: Self = Self {
        copy: true,
        clone: true,
        eq: true,
    };

    /// Every field has to carry the trait, so the struct's set is the intersection.
    fn and(self, other: Self) -> Self {
        Self {
            copy: self.copy && other.copy,
            clone: self.clone && other.clone,
            eq: self.eq && other.eq,
        }
    }
}

/// The derive list for a value struct, in the template's order.
fn derives(strct: &hir::StructDef, tcx: &TypeContext) -> String {
    let traits = struct_traits(strct, tcx);
    let mut list = Vec::new();
    if traits.clone {
        list.push("Clone");
    }
    if traits.copy {
        list.push("Copy");
    }
    // `Debug` and `PartialEq` hold for every field shape the validator admits.
    list.push("Debug");
    list.push("PartialEq");
    if traits.eq {
        list.push("Eq");
    }
    list.join(", ")
}

fn struct_traits(strct: &hir::StructDef, tcx: &TypeContext) -> FieldTraits {
    let mut stack = vec![type_def_name(TypeDef::Struct(strct))];
    strct.fields.iter().fold(FieldTraits::ALL, |traits, field| {
        traits.and(field_traits(&field.ty, tcx, &mut stack))
    })
}

fn field_traits<P: hir::TyPosition>(
    ty: &Type<P>,
    tcx: &TypeContext,
    stack: &mut Vec<String>,
) -> FieldTraits {
    match ty {
        // A float is not `Eq`. Everything else a primitive can be.
        Type::Primitive(hir::PrimitiveType::Float(_)) => FieldTraits {
            eq: false,
            ..FieldTraits::ALL
        },
        Type::Primitive(_) | Type::Enum(_) => FieldTraits::ALL,
        Type::DiplomatOption(inner) => field_traits(inner.as_ref(), tcx, stack),
        Type::Struct(path) => match tcx.resolve_type(path.id()) {
            TypeDef::Struct(inner) => {
                let name = type_def_name(TypeDef::Struct(inner));
                // A struct that contains itself cannot be built, so a repeated name is
                // a shape the validator should have refused; treat it as unconstrained
                // rather than recursing forever.
                if stack.iter().any(|seen| seen == &name) {
                    return FieldTraits::ALL;
                }
                stack.push(name);
                let traits = struct_traits(inner, tcx);
                stack.pop();
                traits
            }
            _ => FieldTraits::ALL,
        },
        Type::Slice(slice) => {
            // A borrow is `Clone`/`Copy` unless it is exclusive; an element type only
            // matters for `Eq`, and only floats are excluded there.
            let shared = !matches!(slice, hir::Slice::Primitive(MaybeOwn::Borrow(borrow), _)
                if borrow.mutability == Mutability::Mutable);
            FieldTraits {
                copy: shared,
                clone: shared,
                ..slice_element_traits(slice, tcx, stack)
            }
        }
        // Validation admits no other field shape; do not claim traits for one.
        _ => FieldTraits {
            copy: false,
            clone: false,
            eq: false,
        },
    }
}

/// A slice is `Eq` when its element type is.
fn slice_element_traits<P: hir::TyPosition>(
    slice: &hir::Slice<P>,
    tcx: &TypeContext,
    stack: &mut Vec<String>,
) -> FieldTraits {
    match slice {
        hir::Slice::Primitive(_, primitive) => match primitive {
            hir::PrimitiveType::Float(_) => FieldTraits {
                eq: false,
                ..FieldTraits::ALL
            },
            _ => FieldTraits::ALL,
        },
        // `&str`/`&[u16]`-style string slices carry code units, which are `Eq`.
        hir::Slice::Str(_, _) | hir::Slice::Strs(_) => FieldTraits::ALL,
        hir::Slice::Struct(_, path) => field_traits(&Type::<P>::Struct(path.clone()), tcx, stack),
        // Validation admits no opaque slice as a field; do not claim traits for one.
        hir::Slice::Opaque(_, _) => FieldTraits {
            copy: false,
            clone: false,
            eq: false,
        },
        // `Slice` is `#[non_exhaustive]`: a variant this backend has never seen gets no
        // claim at all, rather than a derive rustc would have to reject.
        _ => FieldTraits {
            copy: false,
            clone: false,
            eq: false,
        },
    }
}

fn referenced_names_for_struct(
    strct: &hir::StructDef,
    tcx: &TypeContext,
    own_name: &str,
) -> Vec<String> {
    let mut names = BTreeSet::new();
    for field in &strct.fields {
        collect_value_type_names(&field.ty, tcx, &mut names);
    }
    collect_method_names(&strct.methods, tcx, &mut names);
    names.remove(own_name);
    names.into_iter().collect()
}

fn referenced_names_for_methods(
    methods: &[hir::Method],
    tcx: &TypeContext,
    own_name: Option<&str>,
) -> Vec<String> {
    let mut names = BTreeSet::new();
    collect_method_names(methods, tcx, &mut names);
    if let Some(own_name) = own_name {
        names.remove(own_name);
    }
    names.into_iter().collect()
}

fn collect_method_names(methods: &[hir::Method], tcx: &TypeContext, names: &mut BTreeSet<String>) {
    for method in methods.iter().filter(|method| !method.attrs.disable) {
        for param in &method.params {
            collect_value_type_names(&param.ty, tcx, names);
        }
        method.output.with_contained_types(|ty| {
            collect_value_type_names(ty, tcx, names);
        });
    }
}

fn collect_value_type_names<P: hir::TyPosition>(
    ty: &Type<P>,
    tcx: &TypeContext,
    names: &mut BTreeSet<String>,
) {
    match ty {
        Type::Enum(path) => {
            names.insert(type_def_name(TypeDef::Enum(path.resolve(tcx))));
        }
        Type::Struct(path) => {
            names.insert(type_def_name(tcx.resolve_type(path.id())));
        }
        Type::DiplomatOption(inner) => collect_value_type_names(inner, tcx, names),
        Type::Slice(Slice::Struct(_, path)) => {
            names.insert(type_def_name(tcx.resolve_type(path.id())));
        }
        _ => {}
    }
}
