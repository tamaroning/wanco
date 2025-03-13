use anyhow::{bail, Result};
use inkwell::{
    attributes::Attribute,
    values::{IntValue, PointerValue},
    AddressSpace,
};
use wasmparser::{
    Chunk, Element, ElementItems, ElementKind, ElementSectionReader, ExportSectionReader,
    FunctionSectionReader, ImportSectionReader, MemoryType, Operator, Parser, Payload,
    SectionLimited, TableSectionReader, TypeRef,
};

use crate::{
    compile::{
        compile_function::compile_function,
        compile_global::{compile_data_section, compile_global_section},
        compile_memory::compile_memory_section,
        compile_type::compile_type_section,
        cr,
    },
    context::{Context, Function},
};

use super::synthesize::{finalize, initialize};

pub fn compile_module(mut data: &[u8], ctx: &mut Context) -> Result<()> {
    log::info!("Compiling module");
    // Synthesize the entry function
    initialize(ctx)?;

    // Parse Wasm binary and generate LLVM IR
    let mut code_section_data: Option<&[u8]> = None;
    let mut elements_section: Option<SectionLimited<'_, Element<'_>>> = None;

    let mut parser = Parser::new(0);
    loop {
        let payload = match parser.parse(data, true)? {
            Chunk::Parsed { consumed, payload } => {
                if let Payload::CodeSectionStart { size, .. } = &payload {
                    code_section_data = Some(&data[..(*size as usize + consumed)]);
                }
                data = &data[consumed..];
                payload
            }
            // this state is unreachable with `eof = true`
            Chunk::NeedMoreData(_) => unreachable!(),
        };

        //log::debug!("### {:?}", payload.as_ref().expect("fail get payload"));
        match payload {
            Payload::TypeSection(types) => {
                log::debug!("TypeSection");
                compile_type_section(ctx, types)?;
            }
            Payload::ImportSection(imports) => {
                log::debug!("ImportSection");
                compile_import_section(ctx, imports)?;
            }
            Payload::FunctionSection(functions) => {
                log::debug!("FunctionSection");
                compile_function_section(ctx, functions)?;
            }
            Payload::MemorySection(memories) => {
                log::debug!("MemorySection");
                compile_memory_section(ctx, memories)?;
            }
            Payload::TableSection(tables) => {
                log::debug!("TableSection");
                compile_table_section(tables)?;
            }
            Payload::GlobalSection(globals) => {
                log::debug!("GlobalSection]");
                compile_global_section(ctx, globals)?;
            }
            Payload::ExportSection(exports) => {
                log::debug!("ExportSection");
                compile_export_section(ctx, exports)?;
            }
            Payload::ElementSection(elements) => {
                // parse later
                elements_section = Some(elements);
            }
            Payload::DataSection(data_segs) => {
                log::debug!("DataSection");
                compile_data_section(ctx, data_segs)?;
            }
            Payload::CodeSectionEntry(_) => {
                // parse later
            }
            Payload::CustomSection(_c) => {
                log::debug!("CustomSection");
                log::debug!("TODO:")
            }
            Payload::Version { num, encoding, .. } => {
                log::debug!("version:{}, encoding: {:?}", num, encoding);
            }
            Payload::CodeSectionStart { count, range, size } => {
                log::debug!(
                    "CodeSectionStart: count:{}, range:{:?}, size:{}",
                    count,
                    range,
                    size
                );
                parser.skip_section();
                data = &data[size as usize..];
            }
            Payload::End(..) => {
                log::debug!("Parse End");
                break;
            }
            _ => {
                log::warn!(
                    "Unimplemented Section. Run with `RUST_LOG=debug` ctx variable for more info."
                );
            }
        }
    }

    log::info!("Compiling functions");
    declare_functions(ctx)?;
    if let Some(elems) = elements_section {
        compile_element_section(ctx, elems)?;
    }

    // pass
    let mut function_bodies = vec![];
    match code_section_data {
        Some(mut code_section_data) => {
            while let Chunk::Parsed { consumed, payload } =
                parser.parse(code_section_data, false).expect("Error parse")
            {
                code_section_data = &code_section_data[consumed..];
                match payload {
                    Payload::CodeSectionStart { .. } => (),
                    Payload::CodeSectionEntry(f) => {
                        function_bodies.push(f);
                    }
                    _ => unreachable!("Unexpected payload in CodeSection"),
                }
            }
        }
        None => {
            log::error!("CodeSection empty");
        }
    }

    ctx.current_function_idx = Some(ctx.num_imports);
    for body in function_bodies {
        compile_function(ctx, body)?;
        ctx.current_function_idx = Some(ctx.current_function_idx.unwrap() + 1);
    }

    ctx.current_fn = None;
    ctx.current_function_idx = None;

    finalize(ctx)?;

    if ctx.config.enable_cr || ctx.config.legacy_cr {
        log::info!("Inserted {} migration points", ctx.num_migration_points);
    }

    Ok(())
}

fn compile_table_section(tables: TableSectionReader) -> Result<()> {
    for (i, table) in tables.into_iter().enumerate() {
        let table = table?;
        log::debug!("- table[{}] size={:?}", i, table.ty.initial);
    }
    Ok(())
}

fn compile_import_section(ctx: &mut Context<'_, '_>, imports: ImportSectionReader) -> Result<()> {
    assert!(ctx.functions.is_empty());
    ctx.num_imports = imports.count();
    for import in imports {
        let import = import?;
        match import.ty {
            TypeRef::Func(ty) => {
                let mut name = import.name.to_string();
                if import.module == "wasi_snapshot_preview1" || import.module == "wasi_unstable" {
                    name = format!("{}_{}", import.module, name);
                }
                ctx.functions.push(Function {
                    name,
                    type_idx: ty,
                    orig_name: Some((import.module.to_string(), import.name.to_string())),
                });
            }
            TypeRef::Memory(MemoryType {
                memory64: false, ..
            }) => {}
            _ => bail!("Unimplemented import type: {:?}", import.ty),
        }
    }
    log::info!("- declare {} functions", ctx.num_imports);
    Ok(())
}

fn compile_export_section(ctx: &mut Context<'_, '_>, exports: ExportSectionReader) -> Result<()> {
    for export in exports {
        log::debug!("ExportSection {:?}", export);
        let export = export?;
        match export.kind {
            wasmparser::ExternalKind::Func => {
                log::debug!("Export func[{}] = {}", export.name, export.index);
                //ctx.functions[export.index as usize].0 = export.name.to_string();
                if export.name == "_start" {
                    //ctx.functions[export.index as usize].0 = "wanco_start".to_string();
                    ctx.start_function_idx = Some(export.index);
                }
            }
            _ => {
                log::debug!("ExportSection: Exports other than function are not supported");
            }
        }
    }
    Ok(())
}

fn compile_element_section(
    ctx: &mut Context<'_, '_>,
    elements: ElementSectionReader,
) -> Result<()> {
    for element in elements {
        let element = element?;
        match element.kind {
            ElementKind::Active {
                table_index,
                offset_expr,
            } => {
                log::debug!("table[{:?}]", table_index);
                // TODO: support multiple tables
                // FIXME: not sure what to do if table_index is None
                let table_index = table_index.unwrap_or(0);
                assert_eq!(table_index, 0);

                let offset_op = offset_expr
                    .get_binary_reader()
                    .read_operator()
                    .expect("failed to get data section offset");
                let offset = match offset_op {
                    Operator::I32Const { value } => value,
                    _other => unreachable!("unsupported offset type"),
                };
                match element.items {
                    ElementItems::Functions(elems) => {
                        // Declare function pointer array
                        {
                            let mut fpointers: Vec<PointerValue> = Vec::new();
                            for f in ctx.function_values.iter() {
                                fpointers.push(f.as_global_value().as_pointer_value());
                            }
                            let fptr_array = ctx
                                .inkwell_types
                                .i8_ptr_type
                                .array_type(fpointers.len() as u32);
                            let global_fptr_array = ctx.module.add_global(
                                fptr_array,
                                Some(AddressSpace::default()),
                                "GLOBAL_FPTR_ARRAY",
                            );
                            global_fptr_array.set_constant(true);
                            let initializer = ctx.inkwell_types.i8_ptr_type.const_array(&fpointers);
                            global_fptr_array.set_initializer(&initializer);
                            ctx.global_fptr_array = Some(global_fptr_array);
                        }
                        // Declare function table
                        {
                            let table_size = elems.count() + offset as u32;
                            let idx_array_type = ctx.inkwell_types.i32_type.array_type(table_size);
                            let global_table = ctx.module.add_global(
                                idx_array_type,
                                Some(AddressSpace::default()),
                                "global_table",
                            );
                            ctx.global_table = Some(global_table);
                            ctx.global_table_size = Some(table_size as usize);

                            // Initialize function index array
                            let mut fn_indices: Vec<IntValue> = Vec::new();
                            for _ in 0..offset {
                                fn_indices.push(ctx.inkwell_types.i32_type.const_int(0, false));
                            }
                            for (i, elem) in elems.into_iter().enumerate() {
                                let elem = elem?;
                                fn_indices
                                    .push(ctx.inkwell_types.i32_type.const_int(elem as u64, false));
                                log::debug!("- elem[{}] = Function[{}]", i + offset as usize, elem);
                            }
                            let initializer = ctx.inkwell_types.i32_type.const_array(&fn_indices);
                            global_table.set_initializer(&initializer);
                        }
                    }
                    ElementItems::Expressions { .. } => {
                        bail!("ElementSection: Expressions item Unsupported");
                    }
                }
            }
            ElementKind::Declared => {
                bail!("ElementSection: Declared kind Unsupported");
            }
            ElementKind::Passive => {
                bail!("ElementSection: Passive kind Unsupported");
            }
        }
    }
    Ok(())
}

fn compile_function_section(
    ctx: &mut Context<'_, '_>,
    functions: FunctionSectionReader,
) -> Result<()> {
    // Hold function signature
    // These functions will be registerd in ExportSection
    for function in functions {
        let sig = function?;
        let fname = format!("func_{}", ctx.functions.len());
        ctx.functions.push(Function {
            name: fname,
            type_idx: sig,
            orig_name: None,
        });
    }
    ctx.num_functions = ctx.functions.len() as u32;
    log::debug!("- declare {} functions", ctx.num_functions);
    Ok(())
}

fn declare_functions(ctx: &mut Context<'_, '_>) -> Result<()> {
    for function in &ctx.functions {
        // check if fname is already defined
        let f = ctx.module.get_function(&function.name);
        let fn_value = f.unwrap_or({
            let sig = ctx.signatures[function.type_idx as usize];
            let f = ctx.module.add_function(&function.name, sig, None);
            f.get_first_param()
                .expect("should have &exec_env as the first param")
                .set_name("exec_env_ptr");

            // Add noredzone attribute to the function
            let attr_noredzone = ctx
                .ictx
                .create_enum_attribute(Attribute::get_named_enum_kind_id("noredzone"), 0);
            f.add_attribute(inkwell::attributes::AttributeLoc::Function, attr_noredzone);

            // Add noinline attribute to the function since we need correct call stack when making a checkpoint
            if ctx.config.enable_cr || ctx.config.legacy_cr {
                let attr_noinline = ctx
                    .ictx
                    .create_enum_attribute(Attribute::get_named_enum_kind_id("noinline"), 0);
                f.add_attribute(inkwell::attributes::AttributeLoc::Function, attr_noinline);
            }
            f
        });
        ctx.function_values.push(fn_value);
    }
    Ok(())
}
