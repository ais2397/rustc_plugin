extern crate rustc_type_ir;
use std::any::Any;
use std::collections::HashMap;
use std::ops::{ControlFlow, Deref};
//use crate::regex_syntax::hir::print;
use rustc_hir::{HirId, Ty, TyKind};
use rustc_middle::mir::coverage::Op;
use rustc_middle::ty::{self, TypeckResults};
use rustc_middle::query::{IntoQueryParam, Key};
use rustc_middle::ty::TypeVisitor;
use rustc_middle::hir::nested_filter;
//use rustc_transmute::layout::rustc::Def;
use rustc_type_ir::visit::{TypeSuperVisitable, TypeVisitable};
use rustc_span::symbol::Symbol;
use rustc_span::Span;
//use crate::rustix::path::Arg;
use crate::rustc_middle::ty as rustc_ty;


use crate::rustc_middle::ty::TyCtxt;
//use rustc_span::symbol::sym;
use crate::rustc_hir::intravisit::{self, Visitor};
use crate::rustc_hir::ForeignItem;
use crate::rustc_span;
use crate::rustc_hir::def_id::DefId;
use crate::rustc_data_structures::fx::FxHashMap;
use crate::rustc_span::symbol::sym;

pub type ForeignFnMap = FxHashMap<Symbol, (HirId, Option<Symbol>)>; 

pub type ResolvedFnTypeMap<'tcx> = FxHashMap<HirId, (Vec<String>, String)>;

pub(crate) struct CForeignFnCollector<'tcx> {
    tcx: TyCtxt<'tcx>,
    pub foreign_fn_map: ForeignFnMap,
}

impl CForeignFnCollector<'_> {
    pub(crate) fn new(tcx: TyCtxt<'_>) -> CForeignFnCollector<'_> {
        CForeignFnCollector {
            tcx,
            foreign_fn_map: ForeignFnMap::default(),
        }
    }

    pub fn collect_foreign_fns(&mut self) {
        self.tcx.hir().visit_all_item_likes_in_crate(self);
    }

    /*pub(crate) fn get_c_funcs(&self) -> &HashMap<String, String> {
        //create a new hashmap of the foreign functions, initialize it with the foreign function map

        &self.foreign_fn_map.iter().map(|(k, v)| (k.to_string(), v.1.unwrap().to_string())).collect()
    }
*/
    fn extract_fn_name(&self, foreign_item: &ForeignItem<'_>) -> Option<Symbol> {
        Some(foreign_item.ident.name) 
    }

    fn extract_link_name(&self, hir_id: HirId) -> Option<Symbol> {
        // Get the attributes associated with the item
        let attrs = self.tcx.hir().attrs(hir_id);

        // Look for the `link_name` attribute
        for attr in attrs {
            if attr.has_name(sym::link_name) {
                if let Some(value) = attr.value_str() {
                    return Some(value); // Return the value of `#[link_name = "..."]`
                }
            }
        }
        None
    }
}

impl<'tcx> Visitor<'tcx> for CForeignFnCollector<'tcx> {

    //visit foreign items
    fn visit_foreign_item(&mut self, foreign_item: &'tcx ForeignItem<'tcx>) {
        //check if foreign item is a function
        if let rustc_hir::ForeignItemKind::Fn( _, _, _) = foreign_item.kind {
            
        if let Some(fn_name) = self.extract_fn_name(foreign_item) {
            if let Some(link_name) = self.extract_link_name(foreign_item.hir_id()) {
                self.foreign_fn_map.insert(fn_name, (foreign_item.hir_id(), Some(link_name)));
                println!("Function Name: {}", fn_name);
            } 
        }
        intravisit::walk_foreign_item(self, foreign_item);
        }
        
    }
 
}

pub struct FnDeclTypeResolver<'tcx> {
    tcx: TyCtxt<'tcx>,
    foreign_fns: ForeignFnMap, // Foreign functions to process, including names, link_name info, and HirId
    resolved_fn_types: ResolvedFnTypeMap<'tcx>, // Resolved input and output types for each foreign fn
}

impl<'tcx> FnDeclTypeResolver<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, foreign_fns: ForeignFnMap) -> Self {
        FnDeclTypeResolver {
            tcx,
            foreign_fns,
            resolved_fn_types: FxHashMap::default(),
        }
    }

    /// Resolve input and output types for each foreign function.
    pub fn resolve_types(&mut self) {
        for (&fn_name, &(hir_id, _)) in &self.foreign_fns {
            let foreign_item = self.tcx.hir().expect_foreign_item(hir_id.owner);
            //print function name
            println!("--------------------");
            println!("Function Name: {:?}", fn_name);
            if let rustc_hir::ForeignItemKind::Fn(decl, _, _) = &foreign_item.kind {
                // Resolve the input types and output type
                let input_types = self.resolve_input_types(decl);
                let output_type = self.resolve_output_type(decl);

                // Store the resolved types in the map
                self.resolved_fn_types.insert(hir_id, (input_types, output_type));
            }
        }
    }

    /// Resolve the input parameter types for the function.
    fn resolve_input_types(&self, fn_decl: &'tcx rustc_hir::FnDecl<'tcx>) -> Vec<String> {
    // Get the input types
    println!("Inputs: {:?}", fn_decl.inputs);
    fn_decl.inputs.iter().map(|arg| {
        println!("Arg: {:?}", arg);
        // Check if the argument has a type definition
        if arg.hir_id.ty_def_id().is_some() {
            let input_def_id =  arg.hir_id.type_id()
            .expect("Expected a type definition ID");
            // Get the type definition
            let type_def = self.tcx.type_of(input_def_id);
            //print the type definition to the console
            //println!("Type definition: {:?}", type_def);
            /*
            // Convert the type we have to rustc_middle::ty::Ty
            let ty = self.tcx.type_of().skip_binder();
            //print the resolved type to the console
            //println!("Type: {:?}", ty);
            // Resolve and return the type as a string
            match ty.kind() {
                rustc_ty::TyKind::Float(_) => {
                    println!("Float type");
                    "Float".to_string()
                }
                rustc_ty::TyKind::Uint(_) => {
                    println!("Uint type");
                    "Uint".to_string()
                }
                rustc_ty::TyKind::Coroutine(..) => {
                    println!("Coroutine type");
                    "Coroutine".to_string()
                }
                rustc_ty::TyKind::CoroutineWitness(..)=> {
                    println!("Coroutine Witness type");
                    "Coroutine Witness".to_string()
                }
                rustc_ty::TyKind::Alias(..) => {
                    println!("Alias type");
                    "Alias".to_string()
                }
                rustc_ty::TyKind::Bound(..) => {
                    println!("Bound type");
                    "Bound".to_string()
                }
                rustc_ty::TyKind::Placeholder(..) => {
                    println!("Placeholder type");
                    "Placeholder".to_string()
                }

                rustc_ty::TyKind::Array(ty, _) => {
                    println!("Array type: {:?}", ty);
                    "Array".to_string()
                }
                rustc_ty::TyKind::Bool => {
                    println!("Bool type");
                    "Bool".to_string()
                }
                rustc_ty::TyKind::Char => {
                    println!("Char type");
                    "Char".to_string()
                }
                rustc_ty::TyKind::Closure(def_id, substs) => {
                    println!("Closure type: {:?}", def_id);
                    "Closure".to_string()
                }
                rustc_ty::TyKind::Error(..) => {
                    println!("Error type");
                    "Error".to_string()
                }
                rustc_ty::TyKind::FnDef(def_id, substs) => {
                    //println!("Function definition type: {:?}", def_id);
                    "Function".to_string()
                }
                rustc_ty::TyKind::FnPtr(_) => {
                    println!("Function pointer type");
                    "Function Pointer".to_string()
                }

                rustc_ty::TyKind::Infer(_) => {
                    println!("Infer type");
                    "Infer".to_string()
                }
                rustc_ty::TyKind::Int(_) => {
                    println!("Int type");
                    "Int".to_string()
                }
                rustc_ty::TyKind::Never => {
                    println!("Never type");
                    "Never".to_string()
                }
                rustc_ty::TyKind::Param(_) => {
                    println!("Param type");
                    "Param".to_string()
                }

                rustc_ty::TyKind::Ref(_, ty, _) => {
                    println!("Reference type: {:?}", ty);
                    "Reference".to_string()
                }
                rustc_ty::TyKind::Str => {
                    println!("String type");
                    "String".to_string()
                }
                rustc_ty::TyKind::Tuple(_) => {
                    println!("Tuple type");
                    "Tuple".to_string()
                }
                rustc_ty::TyKind::Dynamic(..) => {
                    println!("Dynamic type");
                    "Dynamic".to_string()
                }
                rustc_ty::TyKind::Slice(ty) => {
                    println!("Slice type: {:?}", ty);
                    "Slice".to_string()
                }
                rustc_ty::TyKind::RawPtr(_) => {
                    println!("Raw pointer type");
                    "Raw Pointer".to_string()
                }
                rustc_ty::TyKind::Adt(def_id, substs) => {
                    println!("Adt type: {:?}", def_id);
                    "Adt".to_string()
                }
                rustc_ty::TyKind::Foreign(def_id) => {
                    println!("Foreign type: {:?}", def_id);
                    "Foreign".to_string()
                }
        } 
        } else {
            // If the argument doesn't have a type definition, return a placeholder
            "Unknown".to_string()
    }*/
    "Unknown".to_string()
    }).collect()
}


    /// Resolve the output (return) type for the function.
    fn resolve_output_type(&self, fn_decl: &'tcx rustc_hir::FnDecl<'tcx>) -> String {
        // Check if the output type is explicitly specified or if it's the default `()` (unit type)
        match &fn_decl.output {
            rustc_hir::FnRetTy::Return(ty) => {
                // get the resolved type
                let ty = self.tcx.type_of(ty.hir_id.owner.def_id).skip_binder();
                //print the resolved type to the console
                match ty.kind() {
                    rustc_ty::TyKind::Array(ty, _) => {
                        println!("Array type: {:?}", ty);
                        "Array".to_string()
                    }
                    rustc_ty::TyKind::Float(_) => {
                        println!("Float type");
                        "Float".to_string()
                    }
                    rustc_ty::TyKind::Uint(_) => {
                        println!("Uint type");
                        "Uint".to_string()
                    }
                    rustc_ty::TyKind::Coroutine(..) => {
                        println!("Coroutine type");
                        "Coroutine".to_string()
                    }
                    rustc_ty::TyKind::CoroutineWitness(..)=> {
                        println!("Coroutine Witness type");
                        "Coroutine Witness".to_string()
                    }
                    rustc_ty::TyKind::Alias(..) => {
                        println!("Alias type");
                        "Alias".to_string()
                    }
                    rustc_ty::TyKind::Bound(..) => {
                        println!("Bound type");
                        "Bound".to_string()
                    }
                    rustc_ty::TyKind::Placeholder(..) => {
                        println!("Placeholder type");
                        "Placeholder".to_string()
                    }
                    rustc_ty::TyKind::Dynamic(..) => {
                        println!("Dynamic type");
                        "Dynamic".to_string()
                    }
                    rustc_ty::TyKind::Slice(ty) => {
                        println!("Slice type: {:?}", ty);
                        "Slice".to_string()
                    }
                    rustc_ty::TyKind::RawPtr(_) => {
                        println!("Raw pointer type");
                        "Raw Pointer".to_string()
                    }
                    rustc_ty::TyKind::Adt(def_id, substs) => {
                        println!("Adt type: {:?}", def_id);
                        "Adt".to_string()
                    }
                    rustc_ty::TyKind::Foreign(def_id) => {
                        println!("Foreign type: {:?}", def_id);
                        "Foreign".to_string()
                    }

                    
                    rustc_ty::TyKind::Bool => {
                        println!("Bool type");
                        "Bool".to_string()
                    }
                    rustc_ty::TyKind::Char => {
                        println!("Char type");
                        "Char".to_string()
                    }
                    rustc_ty::TyKind::Closure(def_id, substs) => {
                        println!("Closure type: {:?}", def_id);
                        "Closure".to_string()
                    }
                    rustc_ty::TyKind::Error(..) => {
                        println!("Error type");
                        "Error".to_string()
                    }
                    rustc_ty::TyKind::FnDef(def_id, substs) => {
                        println!("Function definition type: {:?}", def_id);
                        "Function".to_string()
                    }
                    rustc_ty::TyKind::FnPtr(_) => {
                        println!("Function pointer type");
                        "Function Pointer".to_string()
                    }
                    rustc_ty::TyKind::Infer(_) => {
                        //print type and identifier
                        println!("Infer type: {:?}", ty);
                        "Infer".to_string()
                    }
                    rustc_ty::TyKind::Int(_) => {
                        println!("Int type");
                        "Int".to_string()
                    }
                    rustc_ty::TyKind::Never => {
                        println!("Never type");
                        "Never".to_string()
                    }
                    rustc_ty::TyKind::Param(_) => {
                        println!("Param type");
                        "Param".to_string()
                    }
                    rustc_ty::TyKind::Ref(_, ty, _) => {
                        println!("Reference type: {:?}", ty);
                        "Reference".to_string()
                    }
                    rustc_ty::TyKind::Str => {
                        println!("String type");
                        "String".to_string()
                    }
                    rustc_ty::TyKind::Tuple(_) => {
                        println!("Tuple type");
                        "Tuple".to_string()
                    }


                 }
            }
            rustc_hir::FnRetTy::DefaultReturn(_) => {
                // The default return type is `()`
                //println!("Default return type: ()");
                "()".to_string()
                
            }
        }
    }
}