use super::error::CompileError;

#[derive(Clone, Copy)]
pub enum VarType{
    Int,
    Ptr,
    Array
}
pub struct Variable{
    name: String,
    address: i32
}
impl  Variable {
    pub fn new(name:String, address:i32)->Variable{
        Variable { name, address }
    }
}

struct SemanticLevel {
    function_name: Option<String>, //if None, is global
    variables: Vec<Variable>,
}
impl SemanticLevel {
    pub fn new(function_name: Option<String>) -> SemanticLevel {
        SemanticLevel {
            function_name,
            variables: Vec::with_capacity(16),
        }
    }
}
pub struct SimbolTable {
    label_count: usize,
    functions: Vec<(String, usize)>,
    levels: Vec<SemanticLevel>,
}

impl SimbolTable {
    pub fn new() -> SimbolTable {
        SimbolTable {
            label_count: 0,
            functions: Vec::with_capacity(16),
            levels: Vec::with_capacity(16),
        }
    }
    pub fn new_label(&mut self) -> usize {
        let n = self.label_count;
        self.label_count += 1;
        n
    }
    pub fn new_variable(
        &mut self,
        function_name: Option<String>,
        variable: Variable
    ) -> Result<(), CompileError> {
        for level in &mut self.levels {
            if level.function_name == function_name {
                return if level.variables.iter().any(|b| &b.name == &variable.name) {
                    Err(CompileError::Semantic(format!("Redeclaração da variavel '{}'",variable.name)))
                } else {
                    level.variables.push(variable);
                    Ok(())
                };
            }
        }
        let mut sm: SemanticLevel = SemanticLevel::new(function_name);
        sm.variables.push(variable);
        self.levels.push(sm);
        Ok(())
    }
    pub fn new_function(&mut self, function_name: String) -> Result<usize, CompileError> {
        if self.functions.iter().any(|b| &b.0 == &function_name) {
            Err(CompileError::Semantic(format!("Redeclaração da função '{}'",function_name)))
        } else {
            let l = self.new_label();
            self.functions.push((function_name, l));
            Ok(l)
        }
    }

    pub fn get_var_addr_and_type(&self, var: &str, function_name: Option<String>) -> Option<(i32, i32)> {
        for level in &self.levels {
            if level.function_name == function_name {
                for variable in &level.variables {
                    if variable.name == var {
                        return Some((if function_name.is_some() { 1 } else { 0 }, variable.address));
                    }
                }
            }
        }
        if function_name.is_some(){
            //try to find in global
            self.get_var_addr_and_type(var, None)
        }
        else{
            None
        }
    }
    pub fn get_fn_label(&self, function_name: &str) -> Option<usize> {
        self.functions.iter().find(|b| &b.0 == &function_name).map(|b| b.1)
    }
}
