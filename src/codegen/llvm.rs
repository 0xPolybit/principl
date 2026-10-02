use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ast::{
    AssignmentOperator, BinaryOperator, Block, ClassMember, Declaration, Expression,
    ExpressionKind, FunctionDeclaration, InitializerDeclaration, Literal, Statement, StatementKind,
    TypeDeclaration, UnaryOperator,
};
use crate::compiler::{BuildOptions, Target};
use crate::diagnostics::Diagnostic;
use crate::semantic::{GlobalSymbol, TypedProgram};
use crate::source::{SourceFile, SourceSpan};
use crate::types::{FunctionType, Type};

const WINDOWS_X86_64_TRIPLE: &str = "x86_64-w64-windows-gnu";
const WINDOWS_X86_64_DATA_LAYOUT: &str =
    "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128";

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

/// Lower the semantically checked procedural and class subsets to LLVM IR.
pub fn generate_llvm_ir(typed: &TypedProgram, source: &SourceFile) -> Result<String, Diagnostic> {
    IrGenerator::new(typed, source).generate()
}

/// Verify LLVM IR, emit a Windows x86-64 COFF object, and link it to an exe.
///
/// LLVM and the MinGW linker are invoked with structured arguments. All
/// intermediate files live in a uniquely named temporary directory.
pub fn compile_native(ir: &str, options: &BuildOptions) -> Result<(), Diagnostic> {
    if options.compiler.target != Target::WindowsX86_64 {
        return Err(Diagnostic::new(
            "unsupported native target; expected Windows x86-64",
        ));
    }

    let build_dir = TemporaryBuildDirectory::new()?;
    let ir_path = build_dir.path.join("program.ll");
    let object_path = build_dir.path.join("program.obj");
    fs::write(&ir_path, ir).map_err(|error| {
        Diagnostic::new(format!(
            "could not write temporary LLVM IR '{}': {error}",
            ir_path.display()
        ))
    })?;

    let clang = std::env::var_os("PRINCI_CLANG").unwrap_or_else(|| OsString::from("clang"));
    let emitted = Command::new(&clang)
        .arg("-x")
        .arg("ir")
        .arg("-target")
        .arg(WINDOWS_X86_64_TRIPLE)
        .arg("-c")
        .arg(native_tool_path(&ir_path))
        .arg("-o")
        .arg(native_tool_path(&object_path))
        .output()
        .map_err(|error| missing_tool("LLVM IR compiler (clang)", &clang, error))?;
    require_success(
        "LLVM IR verification and Windows x86-64 object generation",
        &clang,
        &emitted,
    )?;

    let linker = std::env::var_os("PRINCI_CC").unwrap_or_else(|| OsString::from("gcc"));
    let target = Command::new(&linker)
        .arg("-dumpmachine")
        .output()
        .map_err(|error| {
            Diagnostic::new(format!(
                "could not start Windows linker '{}': {error}; install x86-64 MinGW GCC or set PRINCI_CC",
                linker.to_string_lossy()
            ))
        })?;
    require_success("Windows linker target detection", &linker, &target)?;
    let target_name = String::from_utf8_lossy(&target.stdout);
    if !target_name.trim().starts_with("x86_64-w64-mingw32") {
        return Err(Diagnostic::new(format!(
            "native linker '{}' targets '{}'; v0.1 requires x86_64-w64-mingw32",
            linker.to_string_lossy(),
            target_name.trim()
        )));
    }

    let mut output_file = TemporaryOutput::beside(&options.output_path)?;
    let linked = Command::new(&linker)
        .arg("-m64")
        .arg(native_tool_path(&object_path))
        .arg("-o")
        .arg(native_tool_path(&output_file.path))
        .output()
        .map_err(|error| {
            Diagnostic::new(format!(
                "could not start Windows linker '{}': {error}",
                linker.to_string_lossy()
            ))
        })?;
    require_success("Windows executable linking", &linker, &linked)?;

    if options.output_path.exists() {
        fs::remove_file(&options.output_path).map_err(|error| {
            Diagnostic::new(format!(
                "could not replace output '{}': {error}",
                options.output_path.display()
            ))
        })?;
    }
    fs::rename(&output_file.path, &options.output_path).map_err(|error| {
        Diagnostic::new(format!(
            "could not move executable to '{}': {error}",
            options.output_path.display()
        ))
    })?;
    output_file.keep = true;
    drop(build_dir);
    Ok(())
}

fn missing_tool(stage: &str, program: &OsStr, error: std::io::Error) -> Diagnostic {
    let hint = if stage.starts_with("LLVM") {
        "install LLVM/Clang with LLVM IR support and the X86 target, or set PRINCI_CLANG"
    } else {
        "install x86-64 MinGW GCC or set PRINCI_CC"
    };
    Diagnostic::new(format!(
        "could not start {stage} '{}': {error}; {hint}",
        program.to_string_lossy()
    ))
}

fn require_success(stage: &str, program: &OsStr, output: &Output) -> Result<(), Diagnostic> {
    if output.status.success() {
        Ok(())
    } else {
        Err(Diagnostic::new(format!(
            "{stage} failed with '{}':\n{}",
            program.to_string_lossy(),
            process_output(output)
        )))
    }
}

fn process_output(output: &Output) -> String {
    let mut text = String::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.trim().is_empty() {
        text.push_str(stdout.trim());
    }
    if !stderr.trim().is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(stderr.trim());
    }
    if text.is_empty() {
        "the tool produced no diagnostic output".to_owned()
    } else {
        text
    }
}

fn native_tool_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let path = path.to_string_lossy();
        if let Some(unc_path) = path.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{unc_path}"));
        }
        if let Some(path) = path.strip_prefix(r"\\?\") {
            return PathBuf::from(path);
        }
        PathBuf::from(path.as_ref())
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

struct TemporaryBuildDirectory {
    path: PathBuf,
    keep: bool,
}

impl TemporaryBuildDirectory {
    fn new() -> Result<Self, Diagnostic> {
        for _ in 0..32 {
            let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("princi-build-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => {
                    let keep = std::env::var_os("PRINCI_KEEP_INTERMEDIATES")
                        .is_some_and(|value| value == "1" || value == "true");
                    return Ok(Self { path, keep });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(Diagnostic::new(format!(
                        "could not create temporary build directory '{}': {error}",
                        path.display()
                    )))
                }
            }
        }
        Err(Diagnostic::new(
            "could not allocate a unique temporary build directory",
        ))
    }
}

impl Drop for TemporaryBuildDirectory {
    fn drop(&mut self) {
        if self.keep {
            eprintln!(
                "princi: kept intermediate files in '{}'",
                self.path.display()
            );
        } else {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

struct TemporaryOutput {
    path: PathBuf,
    keep: bool,
}

impl TemporaryOutput {
    fn beside(destination: &Path) -> Result<Self, Diagnostic> {
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let name = destination.file_name().ok_or_else(|| {
            Diagnostic::new(format!("invalid output path '{}'", destination.display()))
        })?;
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = name.to_os_string();
        temporary_name.push(format!(".princi-{}-{id}.tmp.exe", std::process::id()));
        Ok(Self {
            path: parent.join(temporary_name),
            keep: false,
        })
    }
}

impl Drop for TemporaryOutput {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_file(&self.path);
        }
    }
}

struct IrGenerator<'a> {
    typed: &'a TypedProgram,
    source: &'a SourceFile,
    strings: StringPool,
}

impl<'a> IrGenerator<'a> {
    fn new(typed: &'a TypedProgram, source: &'a SourceFile) -> Self {
        Self {
            typed,
            source,
            strings: StringPool::default(),
        }
    }

    fn generate(mut self) -> Result<String, Diagnostic> {
        let (functions, classes) = self.supported_declarations()?;
        let main = functions
            .iter()
            .find(|function| function.name.name == "main")
            .copied()
            .ok_or_else(|| {
                self.error(
                    "program must declare fn main() as its entry point",
                    self.typed.program.span,
                )
            })?;
        if !main.parameters.is_empty() {
            return Err(self.error(
                "main entry point cannot have parameters",
                main.parameters[0].span,
            ));
        }
        let main_return = self.function_return_type(main)?;
        if !matches!(main_return, Type::Void | Type::Int) {
            return Err(self.error(
                format!("main must return Void or Int, found {main_return}"),
                main.return_type
                    .as_ref()
                    .map_or(main.name.span, |ty| ty.span),
            ));
        }

        let mut definitions = Vec::with_capacity(functions.len());
        for function in &functions {
            definitions.push(
                FunctionEmitter::new(self.typed, self.source, &mut self.strings).emit(function)?,
            );
        }
        for class in &classes {
            definitions.extend(self.emit_class_routines(class)?);
        }

        let mut ir = format!(
            "; Princi v0.1 Windows x86-64 module\ntarget datalayout = \"{WINDOWS_X86_64_DATA_LAYOUT}\"\ntarget triple = \"{WINDOWS_X86_64_TRIPLE}\"\n\n{}",
            crate::runtime::windows_x86_64::LLVM_RUNTIME
        );
        ir.push_str("%princi.typeinfo = type { ptr }\n");
        for class in &classes {
            ir.push_str(&self.class_layout(class)?);
        }
        ir.push('\n');
        for class in &classes {
            ir.push_str(&self.class_typeinfo(class));
        }
        for (index, value) in self.strings.values.iter().enumerate() {
            ir.push_str(&format!(
                "@.princi.str.{index} = private unnamed_addr constant [{} x i8] c\"{}\", align 1\n",
                value.len() + 1,
                llvm_bytes(value.as_bytes())
            ));
        }
        if !self.strings.values.is_empty() {
            ir.push('\n');
        }
        ir.push('\n');
        for definition in definitions {
            ir.push_str(&definition);
            ir.push('\n');
        }
        ir.push_str(&crate::runtime::windows_x86_64::entry_point(
            &function_name("main"),
            main_return == Type::Int,
        ));
        Ok(ir)
    }

    fn supported_declarations(
        &self,
    ) -> Result<(Vec<&'a FunctionDeclaration>, Vec<&'a TypeDeclaration>), Diagnostic> {
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        for declaration in &self.typed.program.declarations {
            match declaration {
                Declaration::Function(function) => functions.push(function),
                Declaration::Class(item) => classes.push(item),
                Declaration::Struct(item) => {
                    return Err(self.error(
                        "structs are parsed and type-checked but are not natively supported in v0.1",
                        item.name.span,
                    ));
                }
                Declaration::Import(item) => {
                    let span = item.path.first().map_or(item.span, |name| name.span);
                    return Err(
                        self.error("imports are not supported by the v0.1 native backend", span)
                    );
                }
            }
        }
        Ok((functions, classes))
    }

    fn class_layout(&self, class: &TypeDeclaration) -> Result<String, Diagnostic> {
        let info = self
            .typed
            .symbols
            .type_symbols(&class.name.name)
            .ok_or_else(|| self.error("missing semantic class layout", class.name.span))?;
        let mut fields = vec!["ptr".to_owned()];
        for field_name in &info.field_order {
            let field_type = info.fields.get(field_name).ok_or_else(|| {
                self.error(
                    format!("missing type information for field '{field_name}'"),
                    class.name.span,
                )
            })?;
            let field = class.members.iter().find_map(|member| match member {
                ClassMember::Field(field) if field.name.name == *field_name => Some(field),
                _ => None,
            });
            let span = field.map_or(class.name.span, |field| field.type_reference.span);
            fields.push(llvm_type(field_type, span, self.source)?.to_owned());
        }
        Ok(format!(
            "{} = type {{ {} }}\n",
            class_type_name(&class.name.name),
            fields.join(", ")
        ))
    }

    fn class_typeinfo(&self, class: &TypeDeclaration) -> String {
        let encoded = encode_identifier(&class.name.name);
        format!(
            "@.princi.typeinfo.name.{encoded} = private unnamed_addr constant [{} x i8] c\"{}\", align 1\n@.princi.typeinfo.{encoded} = private constant %princi.typeinfo {{ ptr @.princi.typeinfo.name.{encoded} }}\n",
            class.name.name.len() + 1,
            llvm_bytes(class.name.name.as_bytes())
        )
    }

    fn emit_class_routines(&mut self, class: &TypeDeclaration) -> Result<Vec<String>, Diagnostic> {
        let Some(info) = self.typed.symbols.type_symbols(&class.name.name).cloned() else {
            return Err(self.error("missing semantic class symbols", class.name.span));
        };
        let mut output = Vec::new();
        for member in &class.members {
            match member {
                ClassMember::Method(method) => {
                    let Some(signature) = info.methods.get(&method.name.name) else {
                        continue;
                    };
                    output.push(
                        FunctionEmitter::new(self.typed, self.source, &mut self.strings)
                            .emit_method(class, method, signature)?,
                    );
                }
                ClassMember::Initializer(initializer) => {
                    if let Some(signature) = info.initializer.as_ref() {
                        output.push(
                            FunctionEmitter::new(self.typed, self.source, &mut self.strings)
                                .emit_initializer(class, initializer, signature)?,
                        );
                    }
                }
                _ => {}
            }
        }
        output.push(self.emit_constructor(class)?);
        Ok(output)
    }

    fn emit_constructor(&self, class: &TypeDeclaration) -> Result<String, Diagnostic> {
        let info = self
            .typed
            .symbols
            .type_symbols(&class.name.name)
            .ok_or_else(|| self.error("missing semantic class symbols", class.name.span))?;
        let explicit_initializer = info.initializer.is_some();
        let field_types = info
            .field_order
            .iter()
            .map(|name| {
                info.fields.get(name).cloned().ok_or_else(|| {
                    self.error(
                        format!("missing type information for field '{name}'"),
                        class.name.span,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let signature = info
            .initializer
            .clone()
            .unwrap_or_else(|| FunctionType::new(field_types.clone(), Type::Void));
        let mut parameters = Vec::new();
        let mut call_arguments = Vec::new();
        for (index, ty) in signature.parameters.iter().enumerate() {
            let ty_name = llvm_type(ty, class.name.span, self.source)?;
            parameters.push(format!("{ty_name} %arg{index}"));
            call_arguments.push(format!("{ty_name} %arg{index}"));
        }

        let encoded = encode_identifier(&class.name.name);
        let layout = class_type_name(&class.name.name);
        let mut body = format!(
            "define ptr @{}({}) {{\nentry:\n  %size.end = getelementptr {layout}, ptr null, i32 1\n  %size = ptrtoint ptr %size.end to i64\n  %object = call ptr @{}(i64 %size)\n  %metadata.slot = getelementptr inbounds {layout}, ptr %object, i32 0, i32 0\n  store ptr @.princi.typeinfo.{encoded}, ptr %metadata.slot\n",
            constructor_name(&class.name.name),
            parameters.join(", "),
            crate::runtime::windows_x86_64::object_allocator_function()
        );
        if explicit_initializer {
            let mut arguments = vec!["ptr %object".to_owned()];
            arguments.extend(call_arguments);
            body.push_str(&format!(
                "  call void @{}({})\n",
                initializer_name(&class.name.name),
                arguments.join(", ")
            ));
        } else {
            for (index, ty) in field_types.iter().enumerate() {
                let ty_name = llvm_type(ty, class.name.span, self.source)?;
                let field_index = index + 1;
                body.push_str(&format!(
                    "  %field.{index} = getelementptr inbounds {layout}, ptr %object, i32 0, i32 {field_index}\n  store {ty_name} %arg{index}, ptr %field.{index}\n"
                ));
            }
        }
        body.push_str("  ret ptr %object\n}\n");
        Ok(body)
    }

    fn function_return_type(&self, function: &FunctionDeclaration) -> Result<Type, Diagnostic> {
        self.function_signature(function)
            .map(|signature| *signature.return_type)
    }

    fn function_signature(
        &self,
        function: &FunctionDeclaration,
    ) -> Result<FunctionType, Diagnostic> {
        match self.typed.symbols.global(&function.name.name) {
            Some(GlobalSymbol::Function(signature)) => Ok(signature.clone()),
            _ => Err(self.error(
                format!(
                    "missing semantic signature for function '{}'",
                    function.name.name
                ),
                function.name.span,
            )),
        }
    }

    fn error(&self, message: impl Into<String>, span: SourceSpan) -> Diagnostic {
        Diagnostic::at(message, self.source.location(span))
    }
}

fn llvm_type(ty: &Type, span: SourceSpan, source: &SourceFile) -> Result<&'static str, Diagnostic> {
    match ty {
        Type::Int => Ok("i64"),
        Type::Float => Ok("double"),
        Type::Bool => Ok("i1"),
        Type::String => Ok("ptr"),
        Type::Class(_) => Ok("ptr"),
        Type::Void => Ok("void"),
        other => Err(Diagnostic::at(
            format!("LLVM code generation does not support type {other}"),
            source.location(span),
        )),
    }
}

fn function_name(name: &str) -> String {
    format!("princi_fn_{}", encode_identifier(name))
}

fn class_type_name(name: &str) -> String {
    format!("%princi.class.{}", encode_identifier(name))
}

fn method_name(class_name: &str, name: &str) -> String {
    format!(
        "princi_method_{}_{}",
        encode_identifier(class_name),
        encode_identifier(name)
    )
}

fn initializer_name(class_name: &str) -> String {
    format!("princi_init_{}", encode_identifier(class_name))
}

fn constructor_name(class_name: &str) -> String {
    format!("princi_new_{}", encode_identifier(class_name))
}

fn encode_identifier(name: &str) -> String {
    name.as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Default)]
struct StringPool {
    values: Vec<String>,
    indices: HashMap<String, usize>,
}

impl StringPool {
    fn intern(&mut self, value: &str) -> usize {
        if let Some(index) = self.indices.get(value) {
            return *index;
        }
        let index = self.values.len();
        self.values.push(value.to_owned());
        self.indices.insert(value.to_owned(), index);
        index
    }
}

fn llvm_bytes(bytes: &[u8]) -> String {
    let mut output = String::new();
    for byte in bytes {
        match byte {
            b' '..=b'~' if *byte != b'"' && *byte != b'\\' => output.push(*byte as char),
            _ => output.push_str(&format!("\\{byte:02X}")),
        }
    }
    output.push_str("\\00");
    output
}

#[derive(Clone)]
struct LocalBinding {
    address: String,
    ty: Type,
}

struct IrValue {
    ty: Type,
    operand: String,
}

struct FunctionEmitter<'a, 'pool> {
    typed: &'a TypedProgram,
    source: &'a SourceFile,
    strings: &'pool mut StringPool,
    output: String,
    allocas: Vec<String>,
    scopes: Vec<HashMap<String, LocalBinding>>,
    next_value: usize,
    next_label: usize,
    current_label: String,
    terminated: bool,
}

impl<'a, 'pool> FunctionEmitter<'a, 'pool> {
    fn new(
        typed: &'a TypedProgram,
        source: &'a SourceFile,
        strings: &'pool mut StringPool,
    ) -> Self {
        Self {
            typed,
            source,
            strings,
            output: String::new(),
            allocas: Vec::new(),
            scopes: Vec::new(),
            next_value: 0,
            next_label: 0,
            current_label: "entry".to_owned(),
            terminated: false,
        }
    }

    fn emit(self, function: &FunctionDeclaration) -> Result<String, Diagnostic> {
        let signature = self.function_signature(function)?;
        self.emit_routine(
            &function_name(&function.name.name),
            &function.name.name,
            function.name.span,
            &function.parameters,
            &function.body,
            &signature,
            None,
        )
    }

    fn emit_method(
        self,
        class: &TypeDeclaration,
        method: &FunctionDeclaration,
        signature: &FunctionType,
    ) -> Result<String, Diagnostic> {
        self.emit_routine(
            &method_name(&class.name.name, &method.name.name),
            &format!("{}.{}", class.name.name, method.name.name),
            method.name.span,
            &method.parameters,
            &method.body,
            signature,
            Some(&class.name.name),
        )
    }

    fn emit_initializer(
        self,
        class: &TypeDeclaration,
        initializer: &InitializerDeclaration,
        signature: &FunctionType,
    ) -> Result<String, Diagnostic> {
        self.emit_routine(
            &initializer_name(&class.name.name),
            &format!("{}.init", class.name.name),
            initializer.span,
            &initializer.parameters,
            &initializer.body,
            signature,
            Some(&class.name.name),
        )
    }

    fn emit_routine(
        mut self,
        symbol: &str,
        display_name: &str,
        name_span: SourceSpan,
        routine_parameters: &[crate::ast::Parameter],
        body: &Block,
        signature: &FunctionType,
        owner: Option<&str>,
    ) -> Result<String, Diagnostic> {
        if signature.return_type.as_ref() != &Type::Void && !block_returns(body) {
            return Err(self.error(
                format!(
                    "function '{display_name}' may finish without returning {}",
                    signature.return_type
                ),
                body.span,
            ));
        }

        let return_ty = llvm_type(&signature.return_type, name_span, self.source)?;
        let mut llvm_parameters = Vec::new();
        if owner.is_some() {
            llvm_parameters.push("ptr %self".to_owned());
        }
        for (index, (parameter, ty)) in routine_parameters
            .iter()
            .zip(&signature.parameters)
            .enumerate()
        {
            let ty_name = llvm_type(ty, parameter.type_reference.span, self.source)?;
            llvm_parameters.push(format!("{ty_name} %arg{index}"));
        }

        self.push_scope();
        if let Some(owner) = owner {
            let address = self.allocate("ptr");
            self.instruction(&format!("store ptr %self, ptr {address}"));
            self.bind(
                "self",
                LocalBinding {
                    address,
                    ty: Type::Class(owner.to_owned()),
                },
            );
        }
        for (index, parameter) in routine_parameters.iter().enumerate() {
            let ty = self
                .typed
                .parameter_type(parameter)
                .cloned()
                .unwrap_or(Type::Error);
            let ty_name = llvm_type(&ty, parameter.type_reference.span, self.source)?;
            let address = self.allocate(&ty_name);
            self.instruction(&format!("store {ty_name} %arg{index}, ptr {address}"));
            self.bind(&parameter.name.name, LocalBinding { address, ty });
        }
        self.emit_block_contents(body)?;
        if !self.terminated {
            if signature.return_type.as_ref() == &Type::Void {
                self.terminate("ret void");
            } else {
                // This is only reachable when flow-sensitive semantic checking
                // is incomplete; it still keeps the module structurally valid.
                let ty_name = llvm_type(&signature.return_type, body.span, self.source)?;
                let fallback = if ty_name == "double" {
                    "0.0"
                } else if ty_name == "ptr" {
                    "null"
                } else {
                    "0"
                };
                self.terminate(&format!("ret {ty_name} {fallback}"));
            }
        }
        self.pop_scope();
        let mut output = format!(
            "define {return_ty} @{}({}) {{\nentry:\n",
            symbol,
            llvm_parameters.join(", ")
        );
        for allocation in self.allocas {
            output.push_str("  ");
            output.push_str(&allocation);
            output.push('\n');
        }
        output.push_str(&self.output);
        output.push_str("}\n");
        Ok(output)
    }

    fn emit_block_contents(&mut self, block: &Block) -> Result<(), Diagnostic> {
        for statement in &block.statements {
            if self.terminated {
                let dead = self.fresh_label("dead");
                self.label(&dead);
            }
            self.emit_statement(statement)?;
        }
        Ok(())
    }

    fn emit_statement(&mut self, statement: &Statement) -> Result<(), Diagnostic> {
        match &statement.kind {
            StatementKind::Variable(variable) => {
                let ty = self
                    .typed
                    .variable_type(variable)
                    .cloned()
                    .unwrap_or(Type::Error);
                let ty_name = llvm_type(&ty, variable.name.span, self.source)?;
                let initializer = variable
                    .initializer
                    .as_ref()
                    .map(|value| self.emit_expression(value))
                    .transpose()?;
                let address = self.allocate(ty_name);
                if let Some(initializer) = initializer {
                    self.instruction(&format!(
                        "store {ty_name} {}, ptr {address}",
                        initializer.operand
                    ));
                }
                self.bind(&variable.name.name, LocalBinding { address, ty });
            }
            StatementKind::Assignment(assignment) => {
                let (address, ty) = match ungroup_kind(&assignment.target.kind) {
                    ExpressionKind::Identifier(identifier) => {
                        let binding = self.lookup(&identifier.name).ok_or_else(|| {
                            self.error(
                                format!(
                                    "code generation could not resolve assignment target '{}'",
                                    identifier.name
                                ),
                                identifier.span,
                            )
                        })?;
                        (binding.address, binding.ty)
                    }
                    ExpressionKind::Member { object, member } => {
                        self.emit_member_address(object, member)?
                    }
                    _ => {
                        return Err(self.error(
                            "the LLVM backend supports assignment to local variables and class fields",
                            assignment.target.span,
                        ));
                    }
                };
                let value = self.emit_expression(&assignment.value)?;
                let ty_name = llvm_type(&ty, assignment.target.span, self.source)?;
                let stored = if assignment.operator == AssignmentOperator::Assign {
                    value.operand
                } else {
                    let previous = self.fresh_value();
                    self.instruction(&format!("{previous} = load {ty_name}, ptr {address}"));
                    match (assignment.operator, &ty) {
                        (AssignmentOperator::AddAssign, Type::String) => {
                            let combined = self.fresh_value();
                            self.instruction(&format!(
                                "{combined} = call ptr @{}(ptr {previous}, ptr {})",
                                crate::runtime::windows_x86_64::string_concat_function(),
                                value.operand
                            ));
                            combined
                        }
                        (operator, ty) => {
                            let result = self.fresh_value();
                            let instruction =
                                assignment_instruction(operator, ty).ok_or_else(|| {
                                    self.error(
                                        format!("invalid assignment operator for type {ty}"),
                                        assignment.target.span,
                                    )
                                })?;
                            self.instruction(&format!(
                                "{result} = {instruction} {ty_name} {previous}, {}",
                                value.operand
                            ));
                            result
                        }
                    }
                };
                self.instruction(&format!("store {ty_name} {stored}, ptr {address}"));
            }
            StatementKind::If(if_statement) => {
                let condition = self.emit_expression(&if_statement.condition)?;
                let then_label = self.fresh_label("if.then");
                let else_label = self.fresh_label("if.else");
                let merge_label = self.fresh_label("if.end");
                self.terminate(&format!(
                    "br i1 {}, label %{then_label}, label %{else_label}",
                    condition.operand
                ));

                self.label(&then_label);
                self.push_scope();
                self.emit_block_contents(&if_statement.then_branch)?;
                self.pop_scope();
                if !self.terminated {
                    self.terminate(&format!("br label %{merge_label}"));
                }

                self.label(&else_label);
                if let Some(else_branch) = &if_statement.else_branch {
                    self.push_scope();
                    self.emit_statement(else_branch)?;
                    self.pop_scope();
                }
                if !self.terminated {
                    self.terminate(&format!("br label %{merge_label}"));
                }
                self.label(&merge_label);
            }
            StatementKind::While(while_statement) => {
                let condition_label = self.fresh_label("while.cond");
                let body_label = self.fresh_label("while.body");
                let end_label = self.fresh_label("while.end");
                self.terminate(&format!("br label %{condition_label}"));
                self.label(&condition_label);
                let condition = self.emit_expression(&while_statement.condition)?;
                self.terminate(&format!(
                    "br i1 {}, label %{body_label}, label %{end_label}",
                    condition.operand
                ));
                self.label(&body_label);
                self.push_scope();
                self.emit_block_contents(&while_statement.body)?;
                self.pop_scope();
                if !self.terminated {
                    self.terminate(&format!("br label %{condition_label}"));
                }
                self.label(&end_label);
            }
            StatementKind::For(for_statement) => {
                let ExpressionKind::Range { start, end } = ungroup_kind(&for_statement.range.kind)
                else {
                    return Err(self.error(
                        "for loop requires an integer range",
                        for_statement.range.span,
                    ));
                };
                let start_value = self.emit_expression(start)?;
                let end_value = self.emit_expression(end)?;
                let start_slot = self.allocate("i64");
                let end_slot = self.allocate("i64");
                self.instruction(&format!(
                    "store i64 {}, ptr {start_slot}",
                    start_value.operand
                ));
                self.instruction(&format!("store i64 {}, ptr {end_slot}", end_value.operand));
                let index_slot = self.allocate("i64");
                self.instruction(&format!(
                    "store i64 {}, ptr {index_slot}",
                    start_value.operand
                ));

                let condition_label = self.fresh_label("for.cond");
                let body_label = self.fresh_label("for.body");
                let step_label = self.fresh_label("for.step");
                let end_label = self.fresh_label("for.end");
                self.terminate(&format!("br label %{condition_label}"));
                self.label(&condition_label);
                let index = self.fresh_value();
                let bound = self.fresh_value();
                let compare = self.fresh_value();
                self.instruction(&format!("{index} = load i64, ptr {index_slot}"));
                self.instruction(&format!("{bound} = load i64, ptr {end_slot}"));
                self.instruction(&format!("{compare} = icmp slt i64 {index}, {bound}"));
                self.terminate(&format!(
                    "br i1 {compare}, label %{body_label}, label %{end_label}"
                ));

                self.label(&body_label);
                self.push_scope();
                self.bind(
                    &for_statement.variable.name,
                    LocalBinding {
                        address: index_slot.clone(),
                        ty: Type::Int,
                    },
                );
                self.push_scope();
                self.emit_block_contents(&for_statement.body)?;
                self.pop_scope();
                self.pop_scope();
                if !self.terminated {
                    self.terminate(&format!("br label %{step_label}"));
                }
                self.label(&step_label);
                let current = self.fresh_value();
                let next = self.fresh_value();
                self.instruction(&format!("{current} = load i64, ptr {index_slot}"));
                self.instruction(&format!("{next} = add i64 {current}, 1"));
                self.instruction(&format!("store i64 {next}, ptr {index_slot}"));
                self.terminate(&format!("br label %{condition_label}"));
                self.label(&end_label);
            }
            StatementKind::Return(value) => {
                if let Some(value) = value {
                    let value = self.emit_expression(value)?;
                    let ty_name = llvm_type(&value.ty, statement.span, self.source)?;
                    self.terminate(&format!("ret {ty_name} {}", value.operand));
                } else {
                    self.terminate("ret void");
                }
            }
            StatementKind::Expression(expression) => {
                self.emit_expression(expression)?;
            }
            StatementKind::Block(block) => {
                self.push_scope();
                self.emit_block_contents(block)?;
                self.pop_scope();
            }
        }
        Ok(())
    }

    fn emit_expression(&mut self, expression: &Expression) -> Result<IrValue, Diagnostic> {
        match &expression.kind {
            ExpressionKind::Identifier(identifier) => {
                let binding = self.lookup(&identifier.name).ok_or_else(|| {
                    self.error(
                        format!(
                            "code generation could not resolve local '{}'",
                            identifier.name
                        ),
                        identifier.span,
                    )
                })?;
                let ty_name = llvm_type(&binding.ty, identifier.span, self.source)?;
                let result = self.fresh_value();
                self.instruction(&format!(
                    "{result} = load {ty_name}, ptr {}",
                    binding.address
                ));
                Ok(IrValue {
                    ty: binding.ty,
                    operand: result,
                })
            }
            ExpressionKind::SelfValue => {
                let binding = self.lookup("self").ok_or_else(|| {
                    self.error("self is unavailable in this routine", expression.span)
                })?;
                let result = self.fresh_value();
                self.instruction(&format!("{result} = load ptr, ptr {}", binding.address));
                Ok(IrValue {
                    ty: binding.ty,
                    operand: result,
                })
            }
            ExpressionKind::Literal(literal) => self.emit_literal(literal, expression.span),
            ExpressionKind::Call { callee, arguments } => {
                self.emit_call(callee, arguments, expression.span)
            }
            ExpressionKind::Member { object, member } => {
                let (address, ty) = self.emit_member_address(object, member)?;
                let ty_name = llvm_type(&ty, member.span, self.source)?;
                let result = self.fresh_value();
                self.instruction(&format!("{result} = load {ty_name}, ptr {address}"));
                Ok(IrValue {
                    ty,
                    operand: result,
                })
            }
            ExpressionKind::Unary { operator, operand } => {
                if *operator == UnaryOperator::Negative && self.is_minimum_int_literal(operand) {
                    return Ok(IrValue {
                        ty: Type::Int,
                        operand: i64::MIN.to_string(),
                    });
                }
                let operand = self.emit_expression(operand)?;
                let result = self.fresh_value();
                let ty_name = llvm_type(&operand.ty, expression.span, self.source)?;
                match operator {
                    UnaryOperator::Positive => Ok(operand),
                    UnaryOperator::Negative if operand.ty == Type::Int => {
                        self.instruction(&format!("{result} = sub i64 0, {}", operand.operand));
                        Ok(IrValue {
                            ty: Type::Int,
                            operand: result,
                        })
                    }
                    UnaryOperator::Negative if operand.ty == Type::Float => {
                        self.instruction(&format!("{result} = fneg double {}", operand.operand));
                        Ok(IrValue {
                            ty: Type::Float,
                            operand: result,
                        })
                    }
                    UnaryOperator::Not => {
                        self.instruction(&format!("{result} = xor i1 {}, true", operand.operand));
                        Ok(IrValue {
                            ty: Type::Bool,
                            operand: result,
                        })
                    }
                    _ => Err(self.error(
                        format!("invalid unary operation for type {ty_name}"),
                        expression.span,
                    )),
                }
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => self.emit_binary(left, *operator, right, expression.span),
            ExpressionKind::Construction { type_name, fields } => {
                self.emit_named_construction(type_name, fields, expression.span)
            }
            ExpressionKind::Group(inner) => self.emit_expression(inner),
            ExpressionKind::List(_)
            | ExpressionKind::Index { .. }
            | ExpressionKind::Range { .. } => Err(self.error(
                "this expression is outside the supported v0.1 LLVM backend",
                expression.span,
            )),
        }
    }

    fn emit_member_address(
        &mut self,
        object: &Expression,
        member: &crate::ast::Identifier,
    ) -> Result<(String, Type), Diagnostic> {
        let receiver = self.emit_expression(object)?;
        let Type::Class(class_name) = &receiver.ty else {
            return Err(self.error(
                format!("cannot lower member '{}' on {}", member.name, receiver.ty),
                member.span,
            ));
        };
        let info = self
            .typed
            .symbols
            .type_symbols(class_name)
            .ok_or_else(|| self.error("missing semantic class layout", member.span))?;
        let Some(field_offset) = info
            .field_order
            .iter()
            .position(|name| name == &member.name)
        else {
            return Err(self.error(
                format!("'{}' is not a field of class '{}'", member.name, class_name),
                member.span,
            ));
        };
        let ty = info
            .fields
            .get(&member.name)
            .cloned()
            .ok_or_else(|| self.error("missing semantic field type", member.span))?;
        let address = self.fresh_value();
        self.instruction(&format!(
            "{address} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}",
            class_type_name(class_name),
            receiver.operand,
            field_offset + 1
        ));
        Ok((address, ty))
    }

    fn emit_named_construction(
        &mut self,
        type_name: &crate::ast::Identifier,
        fields: &[crate::ast::FieldInitializer],
        span: SourceSpan,
    ) -> Result<IrValue, Diagnostic> {
        let class_name = match self.typed.symbols.global(&type_name.name) {
            Some(GlobalSymbol::Type(Type::Class(name))) => name.clone(),
            _ => {
                return Err(self.error(
                    format!(
                        "named construction for '{}' is not supported by the native backend",
                        type_name.name
                    ),
                    span,
                ));
            }
        };
        let object = self.emit_allocate_object(&class_name)?;
        let layout = class_type_name(&class_name);
        let info = self
            .typed
            .symbols
            .type_symbols(&class_name)
            .cloned()
            .ok_or_else(|| self.error("missing semantic class layout", type_name.span))?;
        for field in fields {
            let Some(field_offset) = info
                .field_order
                .iter()
                .position(|name| name == &field.name.name)
            else {
                return Err(self.error("missing semantic field layout", field.name.span));
            };
            let field_type = info
                .fields
                .get(&field.name.name)
                .cloned()
                .ok_or_else(|| self.error("missing semantic field type", field.name.span))?;
            let value = self.emit_expression(&field.value)?;
            if !field_type.accepts(&value.ty) {
                return Err(self.error(
                    "field initializer type changed after semantic analysis",
                    field.value.span,
                ));
            }
            let ty_name = llvm_type(&field_type, field.span, self.source)?;
            let address = self.fresh_value();
            self.instruction(&format!(
                "{address} = getelementptr inbounds {layout}, ptr {object}, i32 0, i32 {}",
                field_offset + 1
            ));
            self.instruction(&format!("store {ty_name} {}, ptr {address}", value.operand));
        }
        Ok(IrValue {
            ty: Type::Class(class_name),
            operand: object,
        })
    }

    fn emit_allocate_object(&mut self, class_name: &str) -> Result<String, Diagnostic> {
        let layout = class_type_name(class_name);
        let encoded = encode_identifier(class_name);
        let size_end = self.fresh_value();
        let size = self.fresh_value();
        let object = self.fresh_value();
        let metadata = self.fresh_value();
        self.instruction(&format!(
            "{size_end} = getelementptr {layout}, ptr null, i32 1"
        ));
        self.instruction(&format!("{size} = ptrtoint ptr {size_end} to i64"));
        self.instruction(&format!(
            "{object} = call ptr @{}(i64 {size})",
            crate::runtime::windows_x86_64::object_allocator_function()
        ));
        self.instruction(&format!(
            "{metadata} = getelementptr inbounds {layout}, ptr {object}, i32 0, i32 0"
        ));
        self.instruction(&format!(
            "store ptr @.princi.typeinfo.{encoded}, ptr {metadata}"
        ));
        Ok(object)
    }

    fn emit_literal(&mut self, literal: &Literal, span: SourceSpan) -> Result<IrValue, Diagnostic> {
        match literal {
            Literal::Integer(value) => {
                let integer = value.parse::<u128>().map_err(|_| {
                    self.error("integer literal is outside the supported Int range", span)
                })?;
                if integer > i64::MAX as u128 {
                    return Err(
                        self.error("integer literal is outside the supported Int range", span)
                    );
                }
                Ok(IrValue {
                    ty: Type::Int,
                    operand: integer.to_string(),
                })
            }
            Literal::FloatingPoint(value) => {
                if !matches!(value.parse::<f64>(), Ok(number) if number.is_finite()) {
                    return Err(self.error(
                        "floating-point literal is outside the supported Float range",
                        span,
                    ));
                }
                Ok(IrValue {
                    ty: Type::Float,
                    operand: value.clone(),
                })
            }
            Literal::String(value) => {
                let index = self.strings.intern(value);
                let length = value.len() + 1;
                let result = self.fresh_value();
                self.instruction(&format!(
                    "{result} = getelementptr inbounds [{length} x i8], ptr @.princi.str.{index}, i64 0, i64 0"
                ));
                Ok(IrValue {
                    ty: Type::String,
                    operand: result,
                })
            }
            Literal::Boolean(value) => Ok(IrValue {
                ty: Type::Bool,
                operand: value.to_string(),
            }),
        }
    }

    fn emit_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        span: SourceSpan,
    ) -> Result<IrValue, Diagnostic> {
        if let ExpressionKind::Member { object, member } = ungroup_kind(&callee.kind) {
            return self.emit_method_call(callee, object, member, arguments, span);
        }
        let ExpressionKind::Identifier(identifier) = ungroup_kind(&callee.kind) else {
            return Err(self.error(
                "only direct function and instance method calls are supported by the v0.1 LLVM backend",
                callee.span,
            ));
        };
        if self.lookup(&identifier.name).is_some() {
            return Err(self.error(
                "indirect calls through local values are outside the v0.1 LLVM backend",
                callee.span,
            ));
        }

        let values = arguments
            .iter()
            .map(|argument| self.emit_expression(argument))
            .collect::<Result<Vec<_>, _>>()?;
        if identifier.name == "print" || identifier.name == "println" {
            if values.len() != 1 {
                return Err(self.error(
                    format!("{} expects exactly one argument", identifier.name),
                    span,
                ));
            }
            let runtime = crate::runtime::windows_x86_64::print_function(
                &values[0].ty,
                identifier.name == "println",
            )
            .ok_or_else(|| {
                self.error(
                    format!(
                        "{} cannot lower values of type {}",
                        identifier.name, values[0].ty
                    ),
                    span,
                )
            })?;
            let expected = llvm_type(&values[0].ty, span, self.source)?;
            self.instruction(&format!(
                "call void @{runtime}({expected} {})",
                values[0].operand
            ));
            return Ok(IrValue {
                ty: Type::Void,
                operand: String::new(),
            });
        }

        let global = self.typed.symbols.global(&identifier.name).cloned();
        let signature = match global {
            Some(GlobalSymbol::Type(Type::Class(class_name))) => {
                let signature = self.constructor_signature(&class_name, span)?;
                self.emit_constructor_call(&class_name, &signature, &values, span)
            }
            Some(GlobalSymbol::Type(Type::Struct(_))) => Err(self.error(
                "struct construction is not supported by the v0.1 native backend",
                identifier.span,
            )),
            Some(GlobalSymbol::Function(signature)) => {
                self.emit_function_call(&identifier.name, &signature, &values, span)
            }
            _ => Err(self.error(
                format!(
                    "missing semantic signature for function '{}'",
                    identifier.name
                ),
                identifier.span,
            )),
        };
        signature
    }

    fn emit_method_call(
        &mut self,
        callee: &Expression,
        object: &Expression,
        member: &crate::ast::Identifier,
        arguments: &[Expression],
        span: SourceSpan,
    ) -> Result<IrValue, Diagnostic> {
        let receiver = self.emit_expression(object)?;
        let Type::Class(class_name) = &receiver.ty else {
            return Err(self.error(
                format!(
                    "cannot dispatch method '{}' on {}",
                    member.name, receiver.ty
                ),
                member.span,
            ));
        };
        let signature = match self.typed.expression_type(callee) {
            Some(Type::Function(signature)) => signature.clone(),
            _ => {
                return Err(self.error(
                    format!(
                        "missing semantic signature for method '{}.{}'",
                        class_name, member.name
                    ),
                    member.span,
                ));
            }
        };
        let values = arguments
            .iter()
            .map(|argument| self.emit_expression(argument))
            .collect::<Result<Vec<_>, _>>()?;
        if values.len() != signature.parameters.len() {
            return Err(self.error(
                "method argument count changed after semantic analysis",
                span,
            ));
        }
        let return_ty = llvm_type(&signature.return_type, span, self.source)?;
        let mut call_arguments = vec![format!("ptr {}", receiver.operand)];
        for (value, ty) in values.iter().zip(&signature.parameters) {
            call_arguments.push(format!(
                "{} {}",
                llvm_type(ty, span, self.source)?,
                value.operand
            ));
        }
        let call = format!(
            "call {return_ty} @{}({})",
            method_name(class_name, &member.name),
            call_arguments.join(", ")
        );
        self.emit_call_result(call, signature.return_type.as_ref().clone())
    }

    fn emit_function_call(
        &mut self,
        name: &str,
        signature: &FunctionType,
        values: &[IrValue],
        span: SourceSpan,
    ) -> Result<IrValue, Diagnostic> {
        if values.len() != signature.parameters.len() {
            return Err(self.error(
                "function argument count changed after semantic analysis",
                span,
            ));
        }
        let return_ty = llvm_type(&signature.return_type, span, self.source)?;
        let arguments = values
            .iter()
            .zip(&signature.parameters)
            .map(|(value, ty)| {
                Ok(format!(
                    "{} {}",
                    llvm_type(ty, span, self.source)?,
                    value.operand
                ))
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?
            .join(", ");
        let call = format!("call {return_ty} @{}({arguments})", function_name(name));
        self.emit_call_result(call, signature.return_type.as_ref().clone())
    }

    fn emit_constructor_call(
        &mut self,
        class_name: &str,
        signature: &FunctionType,
        values: &[IrValue],
        span: SourceSpan,
    ) -> Result<IrValue, Diagnostic> {
        if values.len() != signature.parameters.len() {
            return Err(self.error(
                "constructor argument count changed after semantic analysis",
                span,
            ));
        }
        let arguments = values
            .iter()
            .zip(&signature.parameters)
            .map(|(value, ty)| {
                Ok(format!(
                    "{} {}",
                    llvm_type(ty, span, self.source)?,
                    value.operand
                ))
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?
            .join(", ");
        let result = self.fresh_value();
        self.instruction(&format!(
            "{result} = call ptr @{}({arguments})",
            constructor_name(class_name)
        ));
        Ok(IrValue {
            ty: Type::Class(class_name.to_owned()),
            operand: result,
        })
    }

    fn constructor_signature(
        &self,
        class_name: &str,
        span: SourceSpan,
    ) -> Result<FunctionType, Diagnostic> {
        let info = self
            .typed
            .symbols
            .type_symbols(class_name)
            .ok_or_else(|| self.error("missing semantic class symbols", span))?;
        if let Some(signature) = &info.initializer {
            return Ok(signature.clone());
        }
        let parameters = info
            .field_order
            .iter()
            .map(|name| {
                info.fields.get(name).cloned().ok_or_else(|| {
                    self.error(format!("missing semantic field type for '{name}'"), span)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FunctionType::new(parameters, Type::Void))
    }

    fn emit_call_result(&mut self, call: String, return_type: Type) -> Result<IrValue, Diagnostic> {
        if return_type == Type::Void {
            self.instruction(&call);
            Ok(IrValue {
                ty: Type::Void,
                operand: String::new(),
            })
        } else {
            let result = self.fresh_value();
            self.instruction(&format!("{result} = {call}"));
            Ok(IrValue {
                ty: return_type,
                operand: result,
            })
        }
    }

    fn emit_binary(
        &mut self,
        left: &Expression,
        operator: BinaryOperator,
        right: &Expression,
        span: SourceSpan,
    ) -> Result<IrValue, Diagnostic> {
        if matches!(operator, BinaryOperator::And | BinaryOperator::Or) {
            return self.emit_short_circuit(left, operator, right);
        }
        let left_value = self.emit_expression(left)?;
        let right_value = self.emit_expression(right)?;
        let operand_type = self
            .typed
            .expression_type(left)
            .cloned()
            .unwrap_or(left_value.ty);
        if operator == BinaryOperator::Add && operand_type == Type::String {
            let result = self.fresh_value();
            self.instruction(&format!(
                "{result} = call ptr @{}(ptr {}, ptr {})",
                crate::runtime::windows_x86_64::string_concat_function(),
                left_value.operand,
                right_value.operand
            ));
            return Ok(IrValue {
                ty: Type::String,
                operand: result,
            });
        }
        if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual)
            && operand_type == Type::String
        {
            let comparison = self.fresh_value();
            self.instruction(&format!(
                "{comparison} = call i1 @{}(ptr {}, ptr {})",
                crate::runtime::windows_x86_64::string_equal_function(),
                left_value.operand,
                right_value.operand
            ));
            let result = if operator == BinaryOperator::Equal {
                comparison
            } else {
                let result = self.fresh_value();
                self.instruction(&format!("{result} = xor i1 {comparison}, true"));
                result
            };
            return Ok(IrValue {
                ty: Type::Bool,
                operand: result,
            });
        }

        let left_ty = llvm_type(&operand_type, span, self.source)?;
        let result = self.fresh_value();
        let (instruction, result_ty) = match (&operand_type, operator) {
            (Type::Int, BinaryOperator::Add) => ("add", Type::Int),
            (Type::Int, BinaryOperator::Subtract) => ("sub", Type::Int),
            (Type::Int, BinaryOperator::Multiply) => ("mul", Type::Int),
            (Type::Int, BinaryOperator::Divide) => ("sdiv", Type::Int),
            (Type::Int, BinaryOperator::Remainder) => ("srem", Type::Int),
            (Type::Float, BinaryOperator::Add) => ("fadd", Type::Float),
            (Type::Float, BinaryOperator::Subtract) => ("fsub", Type::Float),
            (Type::Float, BinaryOperator::Multiply) => ("fmul", Type::Float),
            (Type::Float, BinaryOperator::Divide) => ("fdiv", Type::Float),
            (Type::Float, BinaryOperator::Remainder) => ("frem", Type::Float),
            (Type::Int, BinaryOperator::Equal) | (Type::Bool, BinaryOperator::Equal) => {
                ("icmp eq", Type::Bool)
            }
            (Type::Int, BinaryOperator::NotEqual) | (Type::Bool, BinaryOperator::NotEqual) => {
                ("icmp ne", Type::Bool)
            }
            (Type::Int, BinaryOperator::Less) => ("icmp slt", Type::Bool),
            (Type::Int, BinaryOperator::LessEqual) => ("icmp sle", Type::Bool),
            (Type::Int, BinaryOperator::Greater) => ("icmp sgt", Type::Bool),
            (Type::Int, BinaryOperator::GreaterEqual) => ("icmp sge", Type::Bool),
            (Type::Float, BinaryOperator::Equal) => ("fcmp oeq", Type::Bool),
            (Type::Float, BinaryOperator::NotEqual) => ("fcmp une", Type::Bool),
            (Type::Float, BinaryOperator::Less) => ("fcmp olt", Type::Bool),
            (Type::Float, BinaryOperator::LessEqual) => ("fcmp ole", Type::Bool),
            (Type::Float, BinaryOperator::Greater) => ("fcmp ogt", Type::Bool),
            (Type::Float, BinaryOperator::GreaterEqual) => ("fcmp oge", Type::Bool),
            (ty, op) => {
                return Err(self.error(
                    format!("invalid binary operation '{op:?}' for type {ty}"),
                    span,
                ))
            }
        };
        self.instruction(&format!(
            "{result} = {instruction} {left_ty} {}, {}",
            left_value.operand, right_value.operand
        ));
        Ok(IrValue {
            ty: result_ty,
            operand: result,
        })
    }

    fn emit_short_circuit(
        &mut self,
        left: &Expression,
        operator: BinaryOperator,
        right: &Expression,
    ) -> Result<IrValue, Diagnostic> {
        let left_value = self.emit_expression(left)?;
        let left_label = self.current_label.clone();
        let rhs_label = self.fresh_label("logic.rhs");
        let merge_label = self.fresh_label("logic.end");
        let (true_label, false_label, short_value) = if operator == BinaryOperator::And {
            (&rhs_label, &merge_label, "false")
        } else {
            (&merge_label, &rhs_label, "true")
        };
        self.terminate(&format!(
            "br i1 {}, label %{true_label}, label %{false_label}",
            left_value.operand
        ));
        self.label(&rhs_label);
        let right_value = self.emit_expression(right)?;
        let rhs_end = self.current_label.clone();
        if !self.terminated {
            self.terminate(&format!("br label %{merge_label}"));
        }
        self.label(&merge_label);
        let result = self.fresh_value();
        self.instruction(&format!(
            "{result} = phi i1 [ {short_value}, %{left_label} ], [ {}, %{rhs_end} ]",
            right_value.operand
        ));
        Ok(IrValue {
            ty: Type::Bool,
            operand: result,
        })
    }

    fn function_signature(
        &self,
        function: &FunctionDeclaration,
    ) -> Result<FunctionType, Diagnostic> {
        match self.typed.symbols.global(&function.name.name) {
            Some(GlobalSymbol::Function(signature)) => Ok(signature.clone()),
            _ => Err(self.error(
                format!(
                    "missing semantic signature for function '{}'",
                    function.name.name
                ),
                function.name.span,
            )),
        }
    }

    fn is_minimum_int_literal(&self, expression: &Expression) -> bool {
        let ExpressionKind::Literal(Literal::Integer(value)) = &expression.kind else {
            return false;
        };
        value.parse::<u128>().ok() == Some((i64::MAX as u128) + 1)
    }

    fn error(&self, message: impl Into<String>, span: SourceSpan) -> Diagnostic {
        Diagnostic::at(message, self.source.location(span))
    }

    fn instruction(&mut self, instruction: &str) {
        self.output.push_str("  ");
        self.output.push_str(instruction);
        self.output.push('\n');
    }

    fn allocate(&mut self, ty: &str) -> String {
        let address = self.fresh_value();
        self.allocas.push(format!("{address} = alloca {ty}"));
        address
    }

    fn terminate(&mut self, instruction: &str) {
        self.instruction(instruction);
        self.terminated = true;
    }

    fn label(&mut self, label: &str) {
        self.output.push_str(label);
        self.output.push_str(":\n");
        self.current_label = label.to_owned();
        self.terminated = false;
    }

    fn fresh_value(&mut self) -> String {
        let value = format!("%v{}", self.next_value);
        self.next_value += 1;
        value
    }

    fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}.{}", self.next_label);
        self.next_label += 1;
        label
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind(&mut self, name: &str, binding: LocalBinding) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_owned(), binding);
        }
    }

    fn lookup(&self, name: &str) -> Option<LocalBinding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }
}

fn assignment_instruction(operator: AssignmentOperator, ty: &Type) -> Option<&'static str> {
    match (operator, ty) {
        (AssignmentOperator::AddAssign, Type::Int) => Some("add"),
        (AssignmentOperator::SubtractAssign, Type::Int) => Some("sub"),
        (AssignmentOperator::MultiplyAssign, Type::Int) => Some("mul"),
        (AssignmentOperator::AddAssign, Type::Float) => Some("fadd"),
        (AssignmentOperator::SubtractAssign, Type::Float) => Some("fsub"),
        (AssignmentOperator::MultiplyAssign, Type::Float) => Some("fmul"),
        _ => None,
    }
}

fn ungroup_kind(mut kind: &ExpressionKind) -> &ExpressionKind {
    while let ExpressionKind::Group(inner) = kind {
        kind = &inner.kind;
    }
    kind
}

fn block_returns(block: &Block) -> bool {
    block.statements.iter().any(statement_returns)
}

fn statement_returns(statement: &Statement) -> bool {
    match &statement.kind {
        StatementKind::Return(_) => true,
        StatementKind::Block(block) => block_returns(block),
        StatementKind::If(if_statement) => {
            if_statement
                .else_branch
                .as_deref()
                .is_some_and(statement_returns)
                && block_returns(&if_statement.then_branch)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::generate_llvm_ir;
    use crate::lexer;
    use crate::parser;
    use crate::semantic;
    use crate::source::SourceFile;

    fn lower(text: &str) -> Result<String, String> {
        let source = SourceFile::from_text("backend.prnc", text);
        let tokens = lexer::lex(&source).map_err(|errors| errors.to_string())?;
        let parsed = parser::parse(&source, tokens);
        if !parsed.diagnostics.is_empty() {
            return Err(parsed
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"));
        }
        let analyzed = semantic::analyze(&source, &parsed.program);
        if !analyzed.diagnostics.is_empty() {
            return Err(analyzed
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"));
        }
        generate_llvm_ir(&analyzed.typed_program, &source).map_err(|error| error.to_string())
    }

    #[test]
    fn emits_explicit_windows_target_and_recursive_functions() {
        let ir = lower(
            "fn factorial(n: Int) -> Int {\n    if n <= 1 { return 1 }\n    return n * factorial(n - 1)\n}\nfn main() { print(factorial(5)) }",
        )
        .expect("valid recursive program should lower");

        assert!(ir.contains(&format!(
            "target triple = \"{}\"",
            super::WINDOWS_X86_64_TRIPLE
        )));
        assert!(ir.contains("define i64 @princi_fn_666163746f7269616c(i64 %arg0)"));
        assert!(ir.contains("call i64 @princi_fn_666163746f7269616c"));
        assert!(ir.contains("icmp sle i64"));
        assert!(ir.contains("define i32 @main()"));
        assert!(ir.contains("call void @princi_rt_print_int"));
    }

    #[test]
    fn lowers_float_string_boolean_and_short_circuit_operations() {
        let ir = lower(
            r#"fn main() {
    let label = "Hello, " + "Princi"
    let area = 1.5 * 2.0
    let ready = area >= 3.0 && label != "no"
    print(label)
    print(area)
    print(ready)
}"#,
        )
        .expect("primitive program should lower");

        assert!(ir.contains("fadd") || ir.contains("fmul double"));
        assert!(ir.contains("call ptr @princi_rt_string_concat"));
        assert!(ir.contains("call i1 @princi_rt_string_equal"));
        assert!(ir.contains("phi i1"));
        assert!(ir.contains("@princi_rt_print_string"));
        assert!(ir.contains("@princi_rt_print_float"));
        assert!(ir.contains("@princi_rt_print_bool"));
        assert!(ir.contains("@.princi.str."));
    }

    #[test]
    fn resolves_generic_print_builtins_to_typed_runtime_abi_calls() {
        let ir = lower(
            r#"fn main() {
    print(1)
    println(1.5)
    print(true)
    println("text")
}"#,
        )
        .expect("generic print built-ins should lower");

        assert!(ir.contains("call void @princi_rt_print_int(i64 1)"));
        assert!(ir.contains("call void @princi_rt_println_float(double 1.5)"));
        assert!(ir.contains("call void @princi_rt_print_bool(i1 true)"));
        assert!(ir.contains("call void @princi_rt_println_string(ptr"));
    }

    #[test]
    fn emits_control_flow_and_stores_for_ranges_and_mutation() {
        let ir = lower(
            "fn main() {\n    var total = 0\n    for i in 0..4 { total += i }\n    while total < 8 { total += 1 }\n    if total == 8 { print(total) } else { print(0) }\n}",
        )
        .expect("control flow should lower");
        assert!(ir.contains("for.cond."));
        assert!(ir.contains("while.cond."));
        assert!(ir.contains("if.then."));
        assert!(ir.contains("store i64"));
        let last_allocation = ir
            .rfind(" = alloca ")
            .expect("locals and range slots should have storage");
        let first_loop_block = ir
            .find("for.cond.")
            .expect("for-loop CFG should be emitted");
        assert!(
            last_allocation < first_loop_block,
            "all stack slots should be allocated in the function entry block"
        );
    }

    #[test]
    fn reports_struct_declarations_at_their_source_location() {
        let error = lower("struct Point {}\nfn main() {}")
            .expect_err("struct lowering is outside the supported native subset");
        assert!(error.contains("backend.prnc:1:8: error:"));
        assert!(error.contains("structs"));
    }

    #[test]
    fn lowers_class_layout_constructors_fields_self_and_static_method_calls() {
        let ir = lower(
            r#"class User {
    name: String
    age: Int

    init(name: String, age: Int) {
        self.name = name
        self.age = age
    }

    fn birthday() {
        self.age += 1
    }

    fn getAge() -> Int {
        return self.age
    }
}

fn main() {
    var user = User("Alice", 24)
    user.birthday()
    print(user.getAge())
}"#,
        )
        .expect("class program should lower");

        assert!(ir.contains("%princi.class.55736572 = type { ptr, ptr, i64 }"));
        assert!(ir.contains("@princi_init_55736572(ptr %self, ptr %arg0, i64 %arg1)"));
        assert!(ir.contains("@princi_method_55736572_6269727468646179(ptr %self)"));
        assert!(ir.contains("@princi_method_55736572_676574416765(ptr %self)"));
        assert!(ir.contains("call void @princi_init_55736572(ptr %object, ptr %arg0, i64 %arg1)"));
        assert!(ir.contains("call void @princi_method_55736572_6269727468646179(ptr"));
        assert!(ir.contains("call i64 @princi_method_55736572_676574416765(ptr"));
        assert!(ir.contains("@.princi.typeinfo.55736572 = private constant %princi.typeinfo"));
    }

    #[test]
    fn lowers_default_and_named_class_construction_and_field_mutation() {
        let ir = lower(
            r#"class Counter {
    value: Int
}

fn main() {
    var positional = Counter(2)
    var named = Counter { value: 4 }
    positional.value += 1
    named.value = positional.value
    print(named.value)
}"#,
        )
        .expect("default and named class construction should lower");

        assert!(ir.contains("define ptr @princi_new_436f756e746572(i64 %arg0)"));
        assert!(ir.contains("call ptr @princi_rt_alloc_object(i64"));
        assert!(ir.contains("store i64 %arg0, ptr %field.0"));
        assert!(ir.contains("store i64 4, ptr"));
        assert!(ir.contains("store i64 %v"));
    }
}
