extern crate rustc_type_ir;
use std::any::Any;
use std::collections::HashMap;
use std::ops::{ControlFlow, Deref};
use rustc_hir::{HirId, Ty, TyKind};
use rustc_middle::ty::{self, TypeckResults};
use rustc_middle::query::{IntoQueryParam, Key};
use rustc_middle::ty::TypeVisitor;
use rustc_middle::hir::nested_filter;
//use rustc_transmute::layout::rustc::Def;
use rustc_type_ir::visit::{TypeSuperVisitable, TypeVisitable};



use crate::rustc_middle::ty::TyCtxt;
//use rustc_span::symbol::sym;
use crate::rustc_hir::intravisit::{self, Visitor};
use crate::rustc_hir::ForeignItem;
use crate::rustc_span;
use crate::rustc_hir::def_id::DefId;

pub(crate) struct CFuncVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    c_funcs: HashMap<String, String>,
    args: HashMap<String, String>,
    ret_type: HashMap<String, String>,
}

impl CFuncVisitor<'_> {
    pub(crate) fn new(tcx: TyCtxt<'_>) -> CFuncVisitor<'_> {
        CFuncVisitor {
            tcx,
            c_funcs: HashMap::new(),
            args: HashMap::new(),
            ret_type: HashMap::new(),
        }
    }

    pub(crate) fn get_c_funcs(&self) -> &HashMap<String, String> {
        &self.c_funcs
    }
}

impl<'tcx> Visitor<'tcx> for CFuncVisitor<'tcx> {
    //nested filter for all items
    type NestedFilter = nested_filter::All;
    /*fn visit_item(&mut self, i: &'tcx rustc_hir::Item<'tcx>) {
        println!("Item: {:?}", i);
        println!("Item ID: {:?}", i.hir_id());
        intravisit::walk_item(self, i);
    }*/
    fn nested_visit_map(&mut self) -> Self::Map {
                self.tcx.hir()
    }

    type Map =
        <Self::NestedFilter as rustc_hir::intravisit::nested_filter::NestedFilter<'tcx>>::Map;
        
    //visit foreign items
    fn visit_foreign_item(&mut self, item: &'tcx ForeignItem<'tcx>) {
        //check if foreign item is a function
        if let rustc_hir::ForeignItemKind::Fn( _, _, _) = item.kind {
            
        
        let fn_def_id = item.owner_id.to_def_id();
        let fn_name = self.tcx.def_path_str(fn_def_id);
        println!("Function Name: {}", fn_name);
        /*let fn_sig = self.tcx.fn_sig(fn_def_id);
        println!("Function Signature: {:?}", fn_sig);
            // Normalize the signature (to replace possible `impl Trait` with concrete types)
        let fn_sig = fn_sig.skip_binder();

        // Get the input types by decoding binder values
        let inputs: Vec<String> = fn_sig.inputs().iter().map(|arg| format!("{:?}", arg)).collect();
        println!("Input Types: {:?}", inputs);

    

        // Get the output type
        let output = format!("{:?}", fn_sig.output());
        println!("Output Type: {:?}", output);
*/
        // Check if the function is public
        for attr in self.tcx.hir().attrs(item.hir_id()) {
            if attr.has_name(rustc_span::symbol::sym::link_name) {
                if let Some(link_name_value) = attr.value_str() {
                    println!("Found link_name: {}", link_name_value);
                    self.c_funcs.insert(item.ident.to_string(), link_name_value.to_string());
                }
            }
        }
        //get types for input values/parameters of this foreign item
        if let Some(parameters) = self.tcx.hir().fn_sig_by_hir_id(item.hir_id()) {
            let mut args = String::new();
            let mut ret = String::new();
            for arg in parameters.decl.inputs.iter() {
                //use typevisitor to get the type of the argument
                let mut type_analyser = TypeAnalyser::new(self.tcx, arg.hir_id);
                type_analyser.visit_ty(arg);
                args.push_str(&format!("{:?}", type_analyser.resolved_type.unwrap()));
                              
            }
            self.args.insert(item.ident.to_string(), args);
            //self.ret_type.insert(item.ident.to_string(), format!("{:?}", parameters.output));
        }
        //println!("Foreign Item: {:?}", item);*/

        println!("Input Types: {:?}", self.args.get(&item.ident.to_string()));
    }
        intravisit::walk_foreign_item(self, item);

    }

    /*fn visit_fn(&mut self, fk: intravisit::FnKind<'tcx>, fd: &'tcx rustc_hir::FnDecl<'tcx>, b: rustc_hir::BodyId, _: rustc_span::Span, id: rustc_hir::def_id::LocalDefId) {
        println!("Function: {:?}", fk);
        println!("Function Decl: {:?}", fd);
        println!("Function Body: {:?}", b);
        println!("Function ID: {:?}", id);
        intravisit::walk_fn(self, fk, fd, b, id);
    }*/
 
}


pub(crate) struct TypeAnalyser<'tcx> {
    tcx: TyCtxt<'tcx>,
    resolved: bool,
    resolved_type: Option<&'tcx ty::TypeckResults<'tcx>>,
    hir_id: HirId,
}

impl<'tcx> TypeAnalyser<'tcx> {
    pub(crate) fn new(tcx: TyCtxt<'tcx>, hir_id: HirId) -> TypeAnalyser<'tcx> {
        TypeAnalyser {
            tcx,
            resolved: false,
            resolved_type: None,
            hir_id,
        }
    }
}

impl<'tcx> Visitor<'tcx> for TypeAnalyser<'tcx> {
    fn visit_ty(&mut self, t: &'tcx Ty<'tcx>) {
        println!("Type: {:?}", t);
        // Get the `TyKind` of the self.hir_id
        //get local def id of the hir_id
        /*let local_def_id = self.hir_id.owner.to_def_id();
        let hir_ty = self.tcx.typeck(local_def_id);
        println!("HIR Type: {:?}", hir_ty);
        //get the resolved type of the hir_ty
        let ty = 

        // Store the reference to the `Ty` instance
        self.resolved_type = Some(hir_ty);
        self.resolved = true;*/
    }
}
