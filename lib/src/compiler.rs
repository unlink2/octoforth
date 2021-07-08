use super::parser::*;
use super::stmt::*;
use super::error::*;
use super::expr::*;
use super::object::*;
use super::dictionary::*;
use super::token::*;
use super::callable::*;
use super::interpreter::*;
use std::str;
use std::path::{Path, PathBuf};
use super::filesystem::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Compiler {
    stmts: Vec<Stmt>,
    // contains words and compile-time words
    pub dictionary: Box<Dictionary>,

    mod_name: Option<String>,
    pub filesystem: Box<dyn FileSystemManager>,

    // tracks which modules have already been compiled
    // skips compilation of such modules
    pub module_tracker: Rc<RefCell<HashMap<String, Box<Dictionary>>>>,

    parent_dir: PathBuf,

    halt: bool
}

impl Compiler {
    pub fn builtins() -> Box<Dictionary> {
        let builtins = Box::new(Dictionary::new());

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
            filesystem: Box::new(LocalFileSystem),
            module_tracker: Rc::new(RefCell::new(HashMap::new())),
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
        let mut call_obj = self.dictionary.get_any(&token, vec![&None, &self.mod_name])?;
        let compiled = match &mut call_obj {
            Object::Callable(c) => {
                c.compile(self, &token)?
            },
            _ => return Err(Box::new(ExecError::new(ErrorType::UnsupportedObject, token)))
        };

        // apply constants
        let mut cstr = str::from_utf8(&compiled.data)?.to_string();
        match object {
            Object::Callable(_) | Object::Word(_) => cstr = cstr.replace("__ARG__",
                &Dictionary::get_full_name(&object.to_string(),
                &self.mod_name).replace("::", "__mod__")),
            _ => cstr = cstr.replace("__ARG__", &object.to_string())
        }
        cstr = cstr.replace("__WORD__", &Dictionary::get_full_name(&token.lexeme,
                &self.mod_name).replace("::", "__"));

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
                let token = stmt.token();
                return self.call_word(token, "push_default", &Object::Number(*n));
            },
            Object::Real(n) => {
                let token = stmt.token();
                return self.call_word(token, "push_real", &Object::Real(*n));
            },
            Object::Str(n) => {
                let token = stmt.token();
                return self.call_word(token, "push_str", &Object::Str(n.clone()));
            },
            Object::TypedWord(tw) => {
                let token = stmt.token();
                return self.call_word(token, &tw.word, &tw.value);
            },
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
                let mut prefix = self.call_word(token.clone(), "compile", &Object::Word(stmt.name.lexeme.clone()))?;
                let mut postfix = self.call_word(token, "return", &Object::Word(stmt.name.lexeme.clone()))?;
                prefix.data.append(&mut compiled_exec.data);
                prefix.data.append(&mut postfix.data);
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
                compiled.data.append(&mut self.call_word(token.clone(), "__ifelse", &Object::Nil)?.data);
                compiled.data.append(&mut self.execute(&mut stmt.then_block)?.data);
                compiled.data.append(&mut self.call_word(token.clone(), "__else", &Object::Nil)?.data);
                compiled.data.append(&mut self.execute(else_block)?.data);
                compiled.data.append(&mut self.call_word(token.clone(), "__then", &Object::Nil)?.data);
            },
            _ => {
                // if only
                compiled.data.append(&mut self.call_word(token.clone(), "__if", &Object::Nil)?.data);
                compiled.data.append(&mut self.execute(&mut stmt.then_block)?.data);
                compiled.data.append(&mut self.call_word(token.clone(), "__then", &Object::Nil)?.data);
            }
        }
        return Ok(compiled);
    }

    fn visit_loop(&mut self, stmt: &mut LoopStmt) -> BoxResult<Compiled> {
        let mut compiled = Compiled::new(vec![]);

        let token = stmt.token();
        compiled.data.append(&mut self.call_word(token.clone(), "__loop", &Object::Nil)?.data);
        compiled.data.append(&mut self.execute(&mut stmt.block)?.data);
        compiled.data.append(&mut self.call_word(token.clone(), "__until", &Object::Nil)?.data);

        return Ok(compiled);
    }

    fn visit_use(&mut self, stmt: &mut UseStmt) -> BoxResult<Compiled> {
        // compile a module, get all the code and
        // return the compilation output
        // merge dictionaries
        // modules are ketp track of and only compiled once
        // using simple hashing
        let path = match &stmt.path {
            Object::Str(s) => s,
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, stmt.token())))
        };

        let fs = LocalFileSystem;
        let source = fs.read_file(path)?;

        // only compile if we do not have the compiled code already!
        if !self.module_tracker.as_ref().borrow().contains_key(&source) {
            let mut compiler = Compiler::new(&source, &path)?;
            compiler.module_tracker = self.module_tracker.clone();
            let mut compiled = compiler.compile()?;

            let flattened = Compiled::flatten_bytes(&mut compiled);

            // keep track of compilation result in the tracker for later use
            self.module_tracker.as_ref().borrow_mut().insert(source.clone(), compiler.dictionary.clone());
            // merge dictionaries
            self.dictionary.as_mut().extend(compiler.dictionary.as_ref());

            return Ok(flattened);
        } else {
            // if we already did the compilation just add it
            self.dictionary.as_mut().extend(&self.module_tracker.borrow()[&source]);
            Ok(Compiled::new(vec![]))
        }
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

    fn visit_tick(&mut self, stmt: &mut TickStmt) -> BoxResult<Compiled> {
        // tick pushes a word to the stack
        let word = self.evaluate(&mut stmt.word)?;
        match &word {
            Object::Callable(_) => self.call_word(stmt.token(), "__tick",
                &Object::Word(stmt.word.token().lexeme.clone())),
            _ => return Err(Box::new(ExecError::new(ErrorType::TypeError, stmt.token())))
        }
    }
}

impl ExprVisitor for Compiler {
    fn visit_literal(&mut self, expr: &mut LiteralExpr) -> BoxResult<Object> {
        Ok(expr.literal.literal.clone())
    }

    fn visit_word(&mut self, expr: &mut WordExpr) -> BoxResult<Object> {
        self.dictionary.get_any(&expr.name, vec![&None, &self.mod_name])
    }

    fn visit_unary(&mut self, expr: &mut UnaryExpr) -> BoxResult<Object> {
        let obj = self.evaluate(&mut expr.right)?;
        match expr.op.token_type {
            TokenType::I8 => {
                Ok(Object::TypedWord(TypedWord::new(
                            obj.mask(i8::MAX as ObjNumber, &expr.token())?, "push_i8")))
            },
            TokenType::I16 => {
                Ok(Object::TypedWord(TypedWord::new(
                            obj.mask(i16::MAX as ObjNumber, &expr.token())?, "push_i16")))
            },
            TokenType::I32 => {
                Ok(Object::TypedWord(TypedWord::new(
                            obj.mask(i32::MAX as ObjNumber, &expr.token())?, "push_i32")))
            },
            TokenType::I64 => {
                Ok(Object::TypedWord(TypedWord::new(
                            obj.mask(i64::MAX as ObjNumber, &expr.token())?, "push_i64")))
            },
            _ => {
                // should not happen if parser works!
                return Err(Box::new(
                        ExecError::new(ErrorType::UnexpectedToken, expr.op.clone())));
            }
        }
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
            :i call :asm \"jsr __ARG__\" ;
            :i return :asm \"rts\" ;
            : my_word :asm \"lda #100\n\" ;
            my_word", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "my_word:\nlda #100\nrts\njsr my_word\n".to_string()) ;
    }

    #[test]
    fn it_should_use_if() {
        let mut compiler = Compiler::new("
            :i __if :asm \"pla bne :+ \" ;
            :i __then :asm \" : \" ;
            :i push_default :asm \"lda #__ARG__ pha \" ;
            1 if 2 then
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "lda #1 pha \npla bne :+ lda #2 pha  : \n".to_string()) ;
    }

    #[test]
    fn it_should_use_if_else() {
        let mut compiler = Compiler::new("
            :i __ifelse :asm \"pla bne :+ \" ;
            :i __else :asm \" jmp :++ : \" ;
            :i __then :asm \" : \" ;
            :i push_default :asm \"lda #__ARG__ pha \" ;
            1 if 2 else 3 then
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "lda #1 pha \npla bne :+ lda #2 pha  jmp :++ : lda #3 pha  : \n".to_string()) ;
    }

    #[test]
    fn it_should_use_loops() {
        let mut compiler = Compiler::new("
            :i __loop :asm \" : \" ;
            :i __until :asm \"pla ben :- \" ;
            :i push_default :asm \" lda #__ARG__ pha \" ;
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
            :i push_default :asm \"lda #__ARG__ pha\" ;
            test
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "lda #4 pha\n".to_string()) ;
    }

    #[test]
    fn it_should_use_mod_keyword() {
        let mut compiler = Compiler::new("
            :i no_mod :asm \"nomod\" ;
            :mod Tests
            :i compile :asm \":\" ;
            :i call :asm \":\" ;
            :i return :asm \"\nrts\n\" ;
            : test :asm \"lda #00\" ;
            Tests::test
            no_mod
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, ":lda #00\nrts\n\n:\nnomod\n".to_string()) ;
    }

    #[test]
    fn it_should_apply_mod_to_arg_and_word() {
        let mut compiler = Compiler::new("
            :i compile :asm \"arg: __ARG__ word: __WORD__\n\" ;
            :i return :asm \"\nrts\n\" ;
            : no_mod :asm \"nomod\" ;
            :mod Tests
            :i call :asm \":\" ;
            : mod :asm \"mod\" ;
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output,
            "arg: no_mod word: compile\nnomod\nrts\n\narg: Tests__mod__mod word: Tests__compile\nmod\nrts\n\n"
            .to_string()) ;
    }

    #[test]
    fn it_should_push_real() {
        let mut compiler = Compiler::new("
            :i push_real :asm \"__ARG__\n\" ;
            3.1415
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output,"3.1415\n\n"
            .to_string()) ;
    }

    #[test]
    fn it_should_push_str() {
        let mut compiler = Compiler::new("
            :i push_str :asm \"__ARG__\n\" ;
            \"Hello World\"
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output,"Hello World\n\n"
            .to_string()) ;
    }

    #[test]
    fn it_should_push_word_addr_with_tick() {
        let mut compiler = Compiler::new("
            :i compile :asm \"__ARG__ \" ;
            :i return :asm \"rts \" ;
            :i __tick :asm \"lda __ARG__ \" ;
            : myword :asm \"lda #100 \" ;
            ' myword
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output,"myword lda #100 rts \nlda myword \n"
            .to_string()) ;
    }

    #[test]
    fn it_should_use_typed_word_with_annotation() {
        let mut compiler = Compiler::new("
            :i compile :asm \"__ARG__ \" ;
            :i return :asm \"rts \" ;
            :i push_default :asm \"lda __ARG__ \" ;
            :i push_i16 :asm \"lda __ARG__i16 \" ;
            :i16 257
            255
            ", "").unwrap();
        let result = compiler.compile().unwrap();
        let output = Compiled::flatten(result).unwrap();

        assert_eq!(output, "lda 257i16 \nlda 255 \n"
            .to_string()) ;
    }
}
