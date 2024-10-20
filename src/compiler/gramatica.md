<program> ::= <declarations> {<function_def>}

<function_def> ::= "fn" <identifier> "(" <parameter_list> ")" "{" <declarations> <commands> "return" <expression> ";" "}"

<declarations> ::= {<declaration>}

<commands> ::= {<command>}

<vartype> ::= "int" | "ptr"

<parameter_list> ::= [ <vartype> <identifier> {"," <vartype> <identifier>}]

<command_block> ::= "{" <commands> "}"

<declaration> ::= <vartype> <identifier> [ "[" <number> "]" ] { "," <identifier> [ "[" <number> "]" ]} ";"

<attribuition> ::= <identifier> [ "[" <expression> "]" ] "=" <expression>

<expression> ::= <logic_expr> { "||" <logic_expr> }

<logic_expr> ::= <relational_expr> { "&&" <relational_expr> }

<relational_expr> ::= <sum> [ ("<", ">", "<=", ">=", "==", "!=") <sum>]

<sum> ::= <factor> { ( "+" | "-" ) <factor> }

<factor> ::= <operand> { ( "*" | "/" ) <operand> }

<command> ::= [<command_block> | <attribuition> | <if_command> | <while_command> | <print_command>, <read_command>] ";"

<if_command> ::= "if" "(" <expression> ")" <command> ["else" <command>]

<while_command> ::= "while" "(" <expression> ")" <command>

<read_command> ::= "read" "(" <identifier> [ "[" <expression> "]" ] ")" 

<print_command> ::= "print" "(" <argument_list> ")" 

<function_call> ::= <identifier> "(" <argument_list> ")" 

<argument_list> ::= [ <expression> {"," <expression> }]

<operand> ::= <identifier> [ "[" <expression> "]" ] | <number> | "(" <expression> ")" | "-" <operand> | "!" <operand> |  "&" <identifier> [ "[" <expression> "]" ] | "*" <identifier> [ "[" <expression> "]" ] | <function_call>