use anyhow::{bail, Result};
use inkwell::{attributes::Attribute, values::PointerValue, AddressSpace};
use wasmparser::{
    Chunk, Element, ElementItems, ElementKind, ElementSectionReader, ExportSectionReader,
    FunctionSectionReader, ImportSectionReader, Operator, Parser, Payload, SectionLimited,
    TableSectionReader, TypeRef,
};

use crate::{
    compile::{
        compile_function::compile_function,
        compile_global::{compile_data_section, compile_global_section},
        compile_memory::compile_memory_section,
        compile_type::compile_type_section,
    },
    context::Context,
};

use super::synthesize::{finalize, initialize};

pub fn compile_module(mut data: &[u8], ctx: &mut Context) -> Result<()> {
    // Synthesize the entry function
    initialize(ctx)?;

    // Parse Wasm binary and generate LLVM IR
    let mut code_section_data: Option<&[u8]> = None;
    let mut elements_section: Option<SectionLimited<'_, Element<'_>>> = None;

    let mut parser = Parser::new(0);
    log::debug!("Parse Start");
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

    declare_functions(ctx)?;

    if let Some(elems) = elements_section {
        compile_element_section(ctx, elems)?;
    }

    ctx.current_function_idx = ctx.num_imports;
    match code_section_data {
        Some(mut code_section_data) => {
            while let Chunk::Parsed { consumed, payload } =
                parser.parse(code_section_data, false).expect("Error parse")
            {
                code_section_data = &code_section_data[consumed..];
                match payload {
                    Payload::CodeSectionStart { .. } => (),
                    Payload::CodeSectionEntry(f) => {
                        compile_function(ctx, f)?;
                        ctx.current_function_idx += 1;
                    }
                    _ => unreachable!("Unexpected payload in CodeSection"),
                }
            }
        }
        None => {
            log::error!("CodeSection empty");
        }
    }

    finalize(ctx)?;

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
                // We discard module names and just use function names
                ctx.functions.push((import.name.to_string(), ty));
            }
            _ => bail!("Unimplemented import type: {:?}", import.ty),
        }
    }
    log::debug!("- declare {} functions", ctx.num_imports);
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
                        // Declare function pointer array as global
                        let count = elems.count();
                        let array_fpointer = ctx
                            .inkwell_types
                            .i8_ptr_type
                            .array_type(count + offset as u32);
                        let global_table = ctx.module.add_global(
                            array_fpointer,
                            Some(AddressSpace::default()),
                            "global_table",
                        );
                        ctx.global_table = Some(global_table);

                        // Initialize function pointer array
                        let mut fpointers: Vec<PointerValue> = Vec::new();
                        for _ in 0..offset {
                            fpointers.push(ctx.inkwell_types.i8_ptr_type.const_null());
                        }
                        for (i, elem) in elems.into_iter().enumerate() {
                            let elem = elem?;
                            let func = ctx.function_values[elem as usize];
                            fpointers.push(func.as_global_value().as_pointer_value());
                            log::debug!("- elem[{}] = Function[{}]", i + offset as usize, elem);
                        }
                        let initializer = ctx.inkwell_types.i8_ptr_type.const_array(&fpointers);
                        global_table.set_initializer(&initializer);
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
        ctx.functions.push((fname, sig));
    }
    ctx.num_functions = ctx.functions.len() as u32;
    log::debug!("- declare {} functions", ctx.num_functions);
    Ok(())
}

fn declare_functions(ctx: &mut Context<'_, '_>) -> Result<()> {
    for (name, sig) in &ctx.functions {
        // check if fname is already defined
        let f = ctx.module.get_function(name);
        let fn_value = f.unwrap_or({
            let sig = ctx.signatures[*sig as usize];
            let f = ctx.module.add_function(name, sig, None);
            // add attribute
            // create noredzone attribute
            let attr_noredzone = ctx
                .ictx
                .create_enum_attribute(Attribute::get_named_enum_kind_id("noredzone"), 0);

            f.add_attribute(inkwell::attributes::AttributeLoc::Function, attr_noredzone);
            f
        });
        ctx.function_values.push(fn_value);
    }
    Ok(())
}
