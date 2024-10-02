struct SemanticLevel {
    function_name: Option<String>, //if None, is global
    variables: Vec<(String, i32)>,
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
        variable_name: String,
        adress: i32,
    ) -> Result<(), &str> {
        for level in &mut self.levels {
            if level.function_name == function_name {
                return if level.variables.iter().any(|b| &b.0 == &variable_name) {
                    Err("Redeclaration of variable")
                } else {
                    level.variables.push((variable_name, adress));
                    Ok(())
                };
            }
        }
        let mut sm: SemanticLevel = SemanticLevel::new(function_name);
        sm.variables.push((variable_name, adress));
        self.levels.push(sm);
        Ok(())
    }
    pub fn new_function(&mut self, function_name: String) -> Result<usize, &str> {
        if self.functions.iter().any(|b| &b.0 == &function_name) {
            Err("function redeclaration")
        } else {
            let l = self.new_label();
            self.functions.push((function_name, l));
            Ok(l)
        }
    }

    pub fn get_var_addr(&self, var: &str, function_name: Option<String>) -> Option<(i32, i32)> {
        for level in &self.levels {
            if level.function_name == function_name {
                for variable in &level.variables {
                    if variable.0 == var {
                        return Some((if function_name.is_some() { 1 } else { 0 }, variable.1));
                    }
                }
            }
        }
        if function_name.is_some(){
            //try to find in global
            self.get_var_addr(var, None)
        }
        else{
            None
        }
    }
    pub fn get_fn_label(&self, function_name: &str) -> Option<usize> {
        self.functions.iter().find(|b| &b.0 == &function_name).map(|b| b.1)
    }
}
