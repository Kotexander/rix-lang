use crate::{strings::StrId, tir};
use inkwell::{types::BasicType, values::BasicValue};
use std::collections::HashMap;

enum BasicVoidType<'ctx> {
    Basic(inkwell::types::BasicTypeEnum<'ctx>),
    Void(inkwell::types::VoidType<'ctx>),
}
impl<'ctx> BasicVoidType<'ctx> {
    fn fn_type(
        &self,
        param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
        varargs: bool,
    ) -> inkwell::types::FunctionType<'ctx> {
        match self {
            BasicVoidType::Basic(basic_type) => basic_type.fn_type(param_types, varargs),
            BasicVoidType::Void(void_type) => void_type.fn_type(param_types, varargs),
        }
    }
    fn as_basic(&self) -> Option<inkwell::types::BasicTypeEnum<'ctx>> {
        match self {
            BasicVoidType::Basic(basic_type) => Some(*basic_type),
            BasicVoidType::Void(_) => None,
        }
    }
}
impl<'ctx> From<inkwell::types::BasicTypeEnum<'ctx>> for BasicVoidType<'ctx> {
    fn from(value: inkwell::types::BasicTypeEnum<'ctx>) -> Self {
        Self::Basic(value)
    }
}
impl<'ctx> From<inkwell::types::VoidType<'ctx>> for BasicVoidType<'ctx> {
    fn from(value: inkwell::types::VoidType<'ctx>) -> Self {
        Self::Void(value)
    }
}
fn lower_basic_typ<'a>(
    ctx: &'a inkwell::context::Context,
    target_data: &inkwell::targets::TargetData,
    typ: &tir::typ::Type,
) -> Option<BasicVoidType<'a>> {
    match typ {
        tir::typ::Type::Atom(atom_type) => Some(match atom_type {
            tir::typ::AtomType::Void => ctx.void_type().into(),
            tir::typ::AtomType::Bool => ctx.bool_type().as_basic_type_enum().into(),
            tir::typ::AtomType::U8 => ctx.i8_type().as_basic_type_enum().into(),
            tir::typ::AtomType::U16 => ctx.i16_type().as_basic_type_enum().into(),
            tir::typ::AtomType::U32 => ctx.i32_type().as_basic_type_enum().into(),
            tir::typ::AtomType::U64 => ctx.i64_type().as_basic_type_enum().into(),
            tir::typ::AtomType::UPtr => ctx
                .ptr_sized_int_type(target_data, None)
                .as_basic_type_enum()
                .into(),
            tir::typ::AtomType::I8 => ctx.i8_type().as_basic_type_enum().into(),
            tir::typ::AtomType::I16 => ctx.i16_type().as_basic_type_enum().into(),
            tir::typ::AtomType::I32 => ctx.i32_type().as_basic_type_enum().into(),
            tir::typ::AtomType::I64 => ctx.i64_type().as_basic_type_enum().into(),
            tir::typ::AtomType::IPtr => ctx
                .ptr_sized_int_type(target_data, None)
                .as_basic_type_enum()
                .into(),
        }),
        tir::typ::Type::Ptr(_) => {
            Some(ctx.ptr_type(Default::default()).as_basic_type_enum().into())
        }
        _ => None,
    }
}
fn lower_fun_typ<'a>(
    ctx: &'a inkwell::context::Context,
    target_data: &inkwell::targets::TargetData,
    fun_type: &tir::typ::FunType,
    typs: &tir::typ::Typs,
) -> inkwell::types::FunctionType<'a> {
    let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = fun_type
        .params
        .iter()
        .map(|param| lower_basic_typ(ctx, target_data, &typs[*param]).unwrap())
        .filter_map(|param| param.as_basic()) // skip voids
        .map(|basic_void_type| basic_void_type.into())
        .collect();

    let ret_type = lower_basic_typ(ctx, target_data, &typs[fun_type.ret_type]).unwrap();
    ret_type.fn_type(&param_types, fun_type.varargs)
}

pub fn lower(view: tir::TirView) {
    let ctx = inkwell::context::Context::create();

    inkwell::targets::Target::initialize_x86(&Default::default());
    let triple = inkwell::targets::TargetMachine::get_default_triple();
    let target = inkwell::targets::Target::from_triple(&triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &triple,
            "x86-64",
            "",
            inkwell::OptimizationLevel::Default,
            inkwell::targets::RelocMode::PIC,
            inkwell::targets::CodeModel::Default,
        )
        .unwrap();

    // inkwell::targets::Target::initialize_riscv(&Default::default());
    // let triple = inkwell::targets::TargetTriple::create("riscv64-unknown-linux-gnu");
    // let target = inkwell::targets::Target::from_triple(&triple).unwrap();
    // let target_machine = target
    //     .create_target_machine(
    //         &triple,
    //         "generic-rv64",
    //         "",
    //         inkwell::OptimizationLevel::Default,
    //         inkwell::targets::RelocMode::PIC,
    //         inkwell::targets::CodeModel::Default,
    //     )
    //     .unwrap();

    let module = ctx.create_module("");
    module.set_triple(&triple);
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());

    let fun_map: HashMap<tir::def::DefId, inkwell::values::FunctionValue> = view
        .tir
        .funs
        .iter()
        .map(|fun| {
            let def = &view.tir.defs[fun.def];
            let name = view.interner.resolve(def.str);
            let typ = view.tir.typs[def.typ].as_fun().unwrap();
            let llvm_fun_type =
                lower_fun_typ(&ctx, &target_machine.get_target_data(), typ, &view.tir.typs);
            let llvm_fun = module.add_function(name, llvm_fun_type, None);
            (fun.def, llvm_fun)
        })
        .collect();

    for fun in &view.tir.funs {
        lower_fun(
            &ctx,
            &target_machine.get_target_data(),
            &module,
            view,
            &fun_map,
            fun,
        );
    }

    module.print_to_file("test.ll").unwrap();
    if let Err(str) = module.verify() {
        eprintln!(
            "LLVM module verification failed:\n{}",
            str.to_string_lossy()
        );
        return;
    }

    // target_machine
    //     .write_to_file(
    //         &module,
    //         inkwell::targets::FileType::Object,
    //         std::path::Path::new("test.o"),
    //     )
    //     .unwrap();

    // optimize
    let opts = inkwell::passes::PassBuilderOptions::create();
    opts.set_verify_each(true);

    module
        .run_passes("default<O3>", &target_machine, opts)
        .unwrap();
    module.print_to_file("test-opt.ll").unwrap();
    target_machine
        .write_to_file(
            &module,
            inkwell::targets::FileType::Object,
            std::path::Path::new("test.o"),
        )
        .unwrap();
}

#[derive(Debug)]
struct Maps<'a, 'ctx> {
    view: tir::TirView<'a>,
    module: &'a inkwell::module::Module<'ctx>,
    dmap: HashMap<tir::def::DefId, inkwell::values::PointerValue<'ctx>>,
    vmap: HashMap<tir::Temp, inkwell::values::BasicValueEnum<'ctx>>,
    strings: HashMap<StrId, inkwell::values::GlobalValue<'ctx>>,
}
impl<'a, 'ctx> Maps<'a, 'ctx> {
    pub fn new(view: tir::TirView<'a>, module: &'a inkwell::module::Module<'ctx>) -> Self {
        Self {
            view,
            module,
            dmap: HashMap::new(),
            vmap: HashMap::new(),
            strings: HashMap::new(),
        }
    }
    pub fn get_str(&mut self, str_id: StrId) -> &inkwell::values::GlobalValue<'ctx> {
        self.strings.entry(str_id).or_insert_with(|| {
            let str = self.view.interner.resolve(str_id);
            let str_bytes = str.as_bytes();
            let array_type = self
                .module
                .get_context()
                .i8_type()
                .array_type(str_bytes.len() as u32 + 1);
            let global = self.module.add_global(array_type, None, "str");
            global.set_initializer(&self.module.get_context().const_string(str_bytes, true));
            global.set_constant(true);
            global.set_linkage(inkwell::module::Linkage::Private);
            global.set_unnamed_addr(true);
            global
        })
    }
    pub fn get_def(&mut self, def_id: tir::def::DefId) -> inkwell::values::PointerValue<'ctx> {
        self.dmap[&def_id]
    }
    pub fn alloc_def(&mut self, def_id: tir::def::DefId, ptr: inkwell::values::PointerValue<'ctx>) {
        self.dmap.insert(def_id, ptr);
    }
    pub fn get_temp(&mut self, temp: tir::Temp) -> inkwell::values::BasicValueEnum<'ctx> {
        self.vmap[&temp]
    }
    pub fn set_temp(&mut self, temp: tir::Temp, val: inkwell::values::BasicValueEnum<'ctx>) {
        self.vmap.insert(temp, val);
    }
    pub fn get_value(&mut self, value: &tir::Value) -> inkwell::values::BasicValueEnum<'ctx> {
        match value {
            tir::Value::Tmp(temp) => self.get_temp(*temp),
            tir::Value::ConstInt(v) => self
                .module
                .get_context()
                .i32_type()
                .const_int(*v as u64, false)
                .as_basic_value_enum(),
            tir::Value::Str(str_id) => self
                .get_str(*str_id)
                .as_pointer_value()
                .as_basic_value_enum(),
            tir::Value::Def(def) => self.get_def(*def).as_basic_value_enum(),
        }
    }
}

fn lower_fun<'ctx>(
    ctx: &'ctx inkwell::context::Context,
    target_data: &inkwell::targets::TargetData,
    module: &inkwell::module::Module<'ctx>,
    view: tir::TirView,
    fun_map: &HashMap<tir::def::DefId, inkwell::values::FunctionValue<'ctx>>,
    fun: &tir::Fun,
) -> inkwell::values::FunctionValue<'ctx> {
    let llvm_fun = fun_map[&fun.def];

    let Some(body) = fun.body.as_ref() else {
        return llvm_fun;
    };

    let llvm_blocks: Vec<_> = body
        .blocks
        .iter()
        .map(|block| ctx.append_basic_block(llvm_fun, &block.name))
        .collect();

    let entry_builder = ctx.create_builder();
    entry_builder.position_at_end(llvm_blocks[0]);

    let mut maps = Maps::new(view, module);

    // allocate space for arguments
    for (i, param) in llvm_fun.get_param_iter().enumerate() {
        let def = &view.tir.defs[fun.args[i]];
        let str = view.interner.resolve(def.str);
        param.set_name(str);

        let llvm_typ = lower_basic_typ(ctx, target_data, &view.tir.typs[def.typ]).unwrap();
        let ptr = entry_builder
            .build_alloca(llvm_typ.as_basic().unwrap(), &format!("{}.alloca", str))
            .unwrap();
        maps.alloc_def(fun.args[i], ptr);
    }

    // allocate space for locals
    for local in &body.locals {
        let def = &view.tir.defs[*local];
        let str = view.interner.resolve(def.str);
        let llvm_typ = lower_basic_typ(ctx, target_data, &view.tir.typs[def.typ]).unwrap();
        let ptr = entry_builder
            .build_alloca(llvm_typ.as_basic().unwrap(), str)
            .unwrap();
        maps.alloc_def(*local, ptr);
    }

    // store arguments to their allocated space
    for (i, arg) in fun.args.iter().enumerate() {
        let ptr = maps.get_def(*arg);
        entry_builder
            .build_store(ptr, llvm_fun.get_nth_param(i as u32).unwrap())
            .unwrap();
    }

    for (llvm_block, block) in llvm_blocks.iter().zip(&body.blocks) {
        let builder = ctx.create_builder();
        builder.position_at_end(*llvm_block);

        for instr in &block.instrs {
            match instr {
                tir::Instr::Load { dst, src } => {
                    let src_ptr = maps.get_def(*src);
                    let def = &view.tir.defs[*src];
                    let str = view.interner.resolve(def.str);
                    let typ = lower_basic_typ(ctx, target_data, &view.tir.typs[def.typ])
                        .unwrap()
                        .as_basic()
                        .unwrap();
                    let val = builder.build_load(typ, src_ptr, str).unwrap();
                    maps.set_temp(*dst, val);
                }
                tir::Instr::Store { dst, src } => {
                    let dst_ptr = maps.get_def(*dst);
                    let src_val = maps.get_value(src);
                    builder.build_store(dst_ptr, src_val).unwrap();
                }
                tir::Instr::Return { val } => match val {
                    Some(val) => {
                        let ret_val = maps.get_value(val);
                        builder.build_return(Some(&ret_val)).unwrap();
                    }
                    None => {
                        builder.build_return(None).unwrap();
                    }
                },
                tir::Instr::Jump { blk } => {
                    builder
                        .build_unconditional_branch(llvm_blocks[*blk as usize])
                        .unwrap();
                }
                tir::Instr::JumpCond {
                    cond,
                    then_blk,
                    else_blk,
                } => {
                    let cond_val = maps.get_value(cond);
                    builder
                        .build_conditional_branch(
                            cond_val.into_int_value(),
                            llvm_blocks[*then_blk as usize],
                            llvm_blocks[*else_blk as usize],
                        )
                        .unwrap();
                }
                tir::Instr::Cmp { dst, lhs, rhs, op } => {
                    let lhs_val = maps.get_value(lhs);
                    let rhs_val = maps.get_value(rhs);
                    let cmp = match op {
                        tir::CmpOp::Eq => inkwell::IntPredicate::EQ,
                        tir::CmpOp::Ne => inkwell::IntPredicate::NE,
                        tir::CmpOp::Lt => inkwell::IntPredicate::SLT,
                        tir::CmpOp::Gt => inkwell::IntPredicate::SGT,
                        tir::CmpOp::Le => inkwell::IntPredicate::SLE,
                        tir::CmpOp::Ge => inkwell::IntPredicate::SGE,
                    };
                    let val = builder
                        .build_int_compare(
                            cmp,
                            lhs_val.into_int_value(),
                            rhs_val.into_int_value(),
                            "cmp",
                        )
                        .unwrap();
                    maps.set_temp(*dst, val.as_basic_value_enum());
                }
                tir::Instr::BinOp { dst, lhs, rhs, op } => {
                    let lhs_val = maps.get_value(lhs).into_int_value();
                    let rhs_val = maps.get_value(rhs).into_int_value();
                    let val = match op {
                        tir::BinOp::Add => builder.build_int_add(lhs_val, rhs_val, "add").unwrap(),
                        tir::BinOp::Sub => builder.build_int_sub(lhs_val, rhs_val, "sub").unwrap(),
                        tir::BinOp::Mul => builder.build_int_mul(lhs_val, rhs_val, "mul").unwrap(),
                        tir::BinOp::Div => builder
                            .build_int_signed_div(lhs_val, rhs_val, "div")
                            .unwrap(),
                    };
                    maps.set_temp(*dst, val.as_basic_value_enum());
                }
                tir::Instr::Call { dst, callee, args } => {
                    let tir::Value::Def(callee_def) = callee else {
                        panic!("");
                    };
                    let function = fun_map[callee_def];
                    let args: Vec<inkwell::values::BasicMetadataValueEnum> =
                        args.iter().map(|arg| maps.get_value(arg).into()).collect();
                    let val = builder.build_call(function, &args, "").unwrap();
                    if let Some(basic) = val.try_as_basic_value().basic() {
                        maps.set_temp(*dst, basic);
                    }
                }
                tir::Instr::Index {
                    dst,
                    base,
                    index,
                    typ,
                } => {
                    // TODO: FIX THIS
                    let typ = lower_basic_typ(ctx, target_data, &view.tir.typs[*typ])
                        .unwrap()
                        .as_basic()
                        .unwrap();
                    let val = unsafe {
                        builder.build_gep(
                            typ,
                            maps.get_value(base).into_pointer_value(),
                            &[maps.get_value(index).into_int_value()],
                            "",
                        )
                    }
                    .unwrap();
                    let val = builder.build_load(typ, val, "").unwrap();
                    maps.set_temp(*dst, val.as_basic_value_enum());
                }
            }
        }
    }

    llvm_fun
}
