use super::parser::*;
use super::stmt::*;
use super::error::*;
use super::expr::*;
use super::object::*;
use super::dictionary::*;
use super::builtins::*;
use super::token::*;
use super::callable::*;
use super::interpreter::*;
use std::str;
use std::path::{Path, PathBuf};
use super::filesystem::*;

pub enum StackMode {
    Int8,
    Int16,
    Int32,
    Int64
}

pub struct Compiler {
    stmts: Vec<Stmt>,
    // contains words and compile-time words
    pub dictionary: Box<Dictionary>,

    mod_name: Option<String>,
    pub stack_mode: StackMode,
    pub filesystem: Box<dyn FileSystemManager>,

    parent_dir: PathBuf,

    halt: bool
}

impl Compiler {
    pub fn builtins() -> Box<Dictionary> {
        let mut builtins = Box::new(Dictionary::new());

        builtins.define("i8", &None, &Object::Callable(Box::new(Int8)));
        builtins.define("i16", &None, &Object::Callable(Box::new(Int16)));
        builtins.define("i32", &None, &Object::Callable(Box::new(Int32)));
        builtins.define("i64", &None, &Object::Callable(Box::new(Int64)));

        builtins
    }

    pub fn from_file(path: &str) -> Result<Self, ErrorList> {
        let fs = LocalFileSystem;
        let source = match fs.read_file(path) {
            Ok(s) => s,
            Err(err) => return Err(ErrorList::new(vec![err]))
        };

        Self::new(&source, path)
    }

    pub fn new(source: &str, path: &str) -> Result<Self, ErrorList> {
        let mut parser = Parser::new(source, path)?;
        let stmts = parser.parse()?;
        Ok(Self {
            stmts,
            dictionary: Self::builtins(),
            mod_name: None,
            stack_mode: StackMode::Int8,
            filesystem: Box::new(LocalFileSystem),
            parent_dir: Path::new(path).parent().unwrap_or(Path::new(path)).to_path_buf(),
            halt: false
        })
    }

    pub fn compile(&mut self) -> Result<Vec<Compiled>, ErrorList> {
        let mut output = vec![];
        let mut errors = vec![];

        let previous_dir = match self.filesystem.current_dir() {
            Ok(pd) => pd,
            Err(err) => {
                errors.push(err);
                return Err(ErrorList::new(errors));
            }
        };
        self.filesystem.set_current_dir(self.parent_dir.to_str().unwrap_or(""));

        for mut stmt in self.stmts.clone() {
            match self.execute(&mut stmt) {
                Ok(bytes) => {
                    output.push(bytes);
                },
                Err(err) => {
                    errors.push(err);
                    break;
                }
            }

            if self.halt {
                break;
            }
        }

        self.filesystem.set_current_dir(previous_dir.to_str().unwrap_or(""));

        if errors.len() > 0 {
            return Err(ErrorList::new(errors));
        }

        return Ok(output);
    }

    fn execute(&mut self, stmt: &mut Stmt) -> BoxResult<Compiled> {
        stmt.accept(self)
    }

    fn evaluate(&mut self, expr: &mut Expr) -> BoxResult<Object> {
        expr.accept(self)
    }

    /// calls an external word (usually platform specific asm block)
    /// and replaces certain pre-defined constants with strings
    /// constants: __ARG__ == current object in question; __WORD__ == current word;
    /// this builds the API from asm to the forth compiler
    /// it is very basic text-replacement similar to C #defines
    /// be aware that an asm block can be anything and the compiler does not
    /// know how to assemble it.
    fn call_word(&mut self, mut token: Token, name: &str, object: &Object) -> BoxResult<Compiled> {
        token.lexeme = name.into();
        let mut call_obj = self.dictionary.get(&token, &self.mod_name)?;
        let compiled = match &mut call_obj {
            Object::Callable(c) => {
                c.compile(self, &token)?
            },
            _ => return Err(Box::new(ExecError::new(ErrorType::UnsupportedObject, token)))
        };

        // apply constants
        let mut cstr = str::from_utf8(&compiled.data)?.to_string();
        cstr = cstr.replace("__ARG__", &object.to_string());
        cstr = cstr.replace("__WORD__", &token.lexeme);

        Ok(Compiled::new(cstr.into_bytes()))
    }
}

impl StmtVisitor for Compiler {
    fn visit_expr(&mut self, stmt: &mut ExprStmt) -> BoxResult<Compiled> {
        let mut object = self.evaluate(&mut stmt.expr)?;

        match &mut object {
            Object::Callable(c) => {
                match c.mode() {
                    DefineMode::Inline => {
                        // call the word in interpreted mode
                        return Ok(c.compile(self, &stmt.expr.token())?);
                    },
                    DefineMode::Regular => {
                        // arg should be the called word
                        return self.call_word(stmt.token(), "call", &Object::Word(stmt.token().lexeme.clone()));
                    },
                    DefineMode::Constant => {
                        // this cant happen for callables
                        return Ok(Compiled::new(vec![]));
                    }
                };
            },
            Object::Number(n) => {
                // in compiled mode we call the push8,16,32,64 words
                // depending on the compiler mode
                let mut token = stmt.token();
                let r;
                match self.stack_mode {
                    StackMode::Int8 => {
                        r = *n & 0xFF;
                        return self.call_word(token, "push8", &Object::Number(r));
                    },
                    StackMode::Int16 => {
                        r = *n & 0xFFFF;
                        return self.call_word(token, "push16", &Object::Number(r));
                    },
                    StackMode::Int32 => {
                        r = *n & 0xFFFFFFFF;
                        return self.call_word(token, "push32", &Object::Number(r));
                    },
                    StackMode::Int64 => {
                        r = *n & 0xFFFFFFFFFFFFFFF;
                        return self.call_word(token, "push64", &Object::Number(r));
                    }
                }
            }
            // TODO support other types at some point!
            _ => return Err(Box::new(ExecError::new(ErrorType::UnsupportedObject, stmt.expr.token())))
        };
    }

    fn visit_block(&mut self, stmt: &mut BlockStmt) -> BoxResult<Compiled> {
        // replace with new env
        let scope = Box::new(Dictionary::new());
        let prev = std::mem::replace(&mut self.dictionary, scope);
        self.dictionary.parent = Some(prev);
        let mut compiled = Compiled::new(vec![]);

        for stmt in &mut stmt.body {
            compiled.data.append(&mut self.execute(stmt)?.data);
        }

        // move env back
        let no_parent = None;
        let parent = std::mem::replace(&mut self.dictionary.parent, no_parent);
        let _ = std::mem::replace(&mut self.dictionary, parent.unwrap());

        Ok(compiled)
    }

    fn visit_define(&mut self, stmt: &mut DefineStmt) -> BoxResult<Compiled> {
        match stmt.mode {
            DefineMode::Regular => {
                let mut compiled_exec = self.execute(&mut stmt.body)?;

                // call compile word
                let token = stmt.token();
                let mut prefix = self.call_word(token, "compile", &Object::Word(stmt.name.lexeme.clone()))?;
                prefix.data.append(&mut compiled_exec.data);
                let compiled = Compiled::new(prefix.data);

                self.dictionary.define(&stmt.name.lexeme,
                    &self.mod_name,
                    &Object::Callable(Box::new(CompiledCallable {
                        compiled: compiled.clone(),
                        mode: stmt.mode
                    })));
                Ok(compiled)
            },
            DefineMode::Inline => {
                let compiled = self.execute(&mut stmt.body)?;
                self.dictionary.define(&stmt.name.lexeme,
                    &self.mod_name,
                    &Object::Callable(Box::new(CompiledCallable {
                        compiled,
                        mode: stmt.mode
                    })));
                Ok(Compiled::new(vec![]))
            },
            DefineMode::Constant => {
                // consts are interpreted. the object on top of the interpreter
                // stack at the end is our value
                let mut interpreter = Interpreter::with(vec![*stmt.body.clone()]);
                interpreter.interprete()?;
                let value = interpreter.pop(&stmt.token())?;
                self.dictionary.define(&stmt.name.lexeme,
                    &self.mod_name,
                    &value
                );
                Ok(Compiled::new(vec![]))
            }
        }
    }

    fn visit_if(&mut self, stmt: &mut IfStmt) -> BoxResult<Compiled> {
        let mut compiled = Compiled::new(vec![]);

        let token = stmt.token();
        match &mut stmt.else_block {
            Some(else_block) => {
                // if-else
                compiled.data.append(&mut self.call_word(token.clone(), "ifelsestart", &Object::Nil)?.data);
                compiled.data.append(&mut self.execute(&mut stmt.then_block)?.data);
                compiled.data.append(&mut self.call_word(token.clone(), "elsestart", &Object::Nil)?.data);
                compiled.data.append(&mut self.execute(else_block)?.data);
                compiled.data.append(&mut self.call_word(token.clone(), "ifend", &Object::Nil)?.data);
            },
            _ => {
                // if only
                compiled.data.append(&mut self.call_word(token.clone(), "ifstart", &Object::Nil)?.data);
                compiled.data.append(&mut self.execute(&mut stmt.then_block)?.data);
                compiled.data.append(&mut self.call_word(token.clone(), "ifend", &Object::Nil)?.data);
            }
        }
        return Ok(compiled);
    }

    fn visit_loop(&mut self, stmt: &mut LoopStmt) -> BoxResult<Compiled> {
        let mut compiled = Compiled::new(vec![]);

        let token = stmt.token();
        compiled.data.append(&mut self.call_word(token.clone(), "loopstart", &Object::Nil)?.data);
        compiled.data.append(&mut self.execute(&mut stmt.block)?.data);
        compiled.data.append(&mut self.call_word(token.clone(), "untilcheck", &Object::Nil)?.data);

        return Ok(compiled);
    }

    fn visit_use(&mut self, stmt: &mut UseStmt) -> BoxResult<Compiled> {
        // compile a module, get all the code and
        // return the compilation output
        // merge dictionaries
        let path = match &stmt.path {
            Object::Str(s) => s,
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, stmt.token())))
        };
        let mut compiler = Compiler::from_file(&path)?;
        compiler.compile()?;
        Ok(Compiled::new(vec![]))
    }

    fn visit_mod(&mut self, stmt: &mut ModStmt) -> BoxResult<Compiled> {
        self.mod_name = Some(stmt.name.lexeme.clone());
        Ok(Compiled::new(vec![]))
    }

    fn visit_asm(&mut self, stmt: &mut AsmStmt) -> BoxResult<Compiled> {
        match &stmt.code {
            Object::Str(code) => Ok(Compiled::new(code.clone().into_bytes())),
            _ => Err(Box::new(ExecError::new(ErrorType::TypeError, stmt.token())))
        }
    }
}

impl ExprVisitor for Compiler {
    fn visit_literal(&mut self, expr: &mut LiteralExpr) -> BoxResult<Object> {
        Ok(expr.literal.literal.clone())
    }

    fn visit_word(&mut self, expr: &mut WordExpr) -> BoxResult<Object> {
        self.dictionary.get(&expr.name, &self.mod_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_define_inline_inx() {
        let mut compiler = Compiler::new(":i ++ :asm \"label: inx\nrts\" ; ++", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "label: inx\nrts\n".to_string()) ;
    }

    #[test]
    fn it_should_call_word() {
        let mut compiler = Compiler::new(":i compile :asm \"__ARG__:\n\" ;
            :i call :asm \"jsr __ARG__\" ; : my_word :asm \"lda #100\nrts\" ; my_word", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "my_word:\nlda #100\nrts\njsr my_word\n".to_string()) ;
    }

    #[test]
    fn it_should_use_if() {
        let mut compiler = Compiler::new("
            :i ifstart :asm \"pla bne :+ \" ;
            :i ifend :asm \" : \" ;
            :i push8 :asm \"lda #__ARG__ pha \" ;
            1 if 2 then
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "lda #1 pha \npla bne :+ lda #2 pha  : \n".to_string()) ;
    }

    #[test]
    fn it_should_use_if_else() {
        let mut compiler = Compiler::new("
            :i ifelsestart :asm \"pla bne :+ \" ;
            :i elsestart :asm \" jmp :++ : \" ;
            :i ifend :asm \" : \" ;
            :i push8 :asm \"lda #__ARG__ pha \" ;
            1 if 2 else 3 then
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "lda #1 pha \npla bne :+ lda #2 pha  jmp :++ : lda #3 pha  : \n".to_string()) ;
    }

    #[test]
    fn it_should_use_loops() {
        let mut compiler = Compiler::new("
            :i loopstart :asm \" : \" ;
            :i untilcheck :asm \"pla ben :- \" ;
            :i push8 :asm \" lda #__ARG__ pha \" ;
            loop 1 until
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, " :  lda #1 pha pla ben :- \n".to_string()) ;
    }

    #[test]
    fn it_should_use_consts() {
        let mut compiler = Compiler::new("
            :c test 2 2 + ;
            :i push8 :asm \"lda #__ARG__ pha\" ;
            test
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "lda #4 pha\n".to_string()) ;
    }
}
