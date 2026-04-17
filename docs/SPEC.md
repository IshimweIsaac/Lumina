# Extended Backus-Naur Form (EBNF) Specification (v1.8.0)

This document contains the formal grammar and error registry for Lumina v1.8.0.

## 1. Global Program Structure
```ebnf
program ::= statement* EOF

statement ::= import_stmt
            | fn_decl
            | entity_decl
            | let_stmt
            | rule_decl
            | action_stmt
            | aggregate_decl
            | external_decl
            | NEWLINE

aggregate_decl ::= 'aggregate' IDENT 'over' IDENT '{' NEWLINE (IDENT ':=' aggregate_func NEWLINE)* '}'
aggregate_func ::= ('avg' | 'min' | 'max' | 'sum' | 'count' | 'any' | 'all') '(' IDENT? ')'
```

## 2. Modules and Functions
```ebnf
import_stmt ::= 'import' STRING NEWLINE

fn_decl ::= 'fn' IDENT '(' (fn_param (',' fn_param)*)? ')' '->' type '{' expr '}'

fn_param ::= IDENT ':' type
```

## 3. Entity and Composition Forms
```ebnf
entity_decl ::= 'entity' IDENT '{' NEWLINE field* '}'

field ::= metadata* (stored_field | derived_field | ref_field)
stored_field ::= IDENT ':' type NEWLINE
derived_field ::= IDENT ':=' expr NEWLINE
ref_field ::= IDENT ':' 'ref' IDENT NEWLINE

metadata ::= '@doc' STRING NEWLINE
           | '@range' NUMBER 'to' NUMBER NEWLINE
           | '@affects' IDENT (',' IDENT)* NEWLINE

type ::= 'Text' | 'Number' | 'Boolean' | 'Timestamp' | IDENT | type '[]'
```

## 4. Abstract Values and Creation
```ebnf
let_stmt ::= 'let' IDENT '=' (expr | entity_init) NEWLINE

entity_init ::= IDENT '{' NEWLINE (IDENT ':' expr NEWLINE)* '}'
```

## 5. Rules and Temporal Logic
```ebnf
rule_decl ::= 'rule' STRING ( 'for' '(' IDENT ':' IDENT ')' )? '{' NEWLINE
              ( 'when' condition ( 'and' condition )* NEWLINE
              | 'every' duration NEWLINE )
              ('then' action NEWLINE)+ 
              ('on clear' '{' action+ '}')?
              ('cooldown' duration)?
              '}'

condition ::= fleet_trigger | frequency_trigger | expr

fleet_trigger ::= ('any' | 'all') IDENT '.' IDENT 'becomes' expr ('for' duration)?

frequency_trigger ::= expr 'becomes' expr NUMBER 'times' 'within' duration

duration ::= NUMBER ('s' | 'm' | 'h' | 'd')
```

## 6. External Entities
```ebnf
external_decl ::= 'external' 'entity' IDENT '{' NEWLINE
                  field*
                  '}' sync_config

sync_config ::= 'sync' 'on' IDENT NEWLINE
              | 'sync' ':' STRING 'on' ':' ("realtime" | "poll" | "webhook") NEWLINE
              ('poll_interval' ':' duration NEWLINE)?
```

## 7. Runtime Actions
```ebnf
action ::= show_action
         | update_action
         | write_action
         | create_action
         | delete_action
         | alert_action

show_action ::= 'show' expr
update_action ::= 'update' path 'to' expr
write_action ::= 'write' path '=' expr
create_action ::= 'create' IDENT '{' NEWLINE (IDENT ':' expr NEWLINE)* '}'
delete_action ::= 'delete' IDENT
alert_action ::= 'alert' 'severity' ':' STRING (',' 'message' ':' STRING)? (',' 'source' ':' expr)? (',' 'code' ':' STRING)? (',' 'payload' ':' '{' ... '}')?

path ::= IDENT ('.' IDENT)*
```

## 8. Expressions (Pratt Operator Precedence)
```ebnf
expr ::= or_expr ('becomes' expr)? ('for' duration)?

or_expr ::= and_expr ('or' and_expr)*
and_expr ::= not_expr ('and' not_expr)*
not_expr ::= 'not' not_expr | cmp_expr
cmp_expr ::= add_expr (cmp_op add_expr)?
add_expr ::= mul_expr (('+' | '-') mul_expr)*
mul_expr ::= unary_expr (('*' | '/' | 'mod') unary_expr)*
unary_expr ::= '-' primary | primary

primary ::= NUMBER | STRING | BOOL | IDENT | path_access | prev_expr | age_expr
          | '(' expr ')'
          | if_expr
          | call_expr
          | list_literal
          | index_expr

prev_expr ::= 'prev' '(' IDENT ')'
age_expr ::= path '.age'
list_literal ::= '[' (expr (',' expr)*)? ']'
index_expr ::= primary '[' expr ']'

if_expr ::= 'if' expr 'then' expr 'else' expr
path_access ::= primary '.' IDENT
call_expr ::= IDENT '(' (expr (',' expr)*)? ')'
cmp_op ::= '==' | '!=' | '>' | '<' | '>=' | '<='
```

## 9. Error Code Registry

| Code | Type | Description |
|---|---|---|
| L001 | Analyzer | Unknown identifier / Duplicate entity |
| L002 | Analyzer | Type mismatch |
| L003 | Analyzer | Derived field cycle detected |
| L004 | Analyzer | Cannot assign to derived field |
| L011 | Analyzer | Duplicate function name |
| L012 | Analyzer | Unknown function call |
| L024 | Analyzer | `prev()` applied to derived field |
| L025 | Analyzer | Nested `prev()` call |
| L026 | Analyzer | `any`/`all` applied to non-boolean |
| L034 | Analyzer | Cooldown duration zero or negative |
| L035 | Analyzer | Multi-condition trigger limit exceeded (>3) |
| L036 | Analyzer | `ref` field references unknown entity |
| L037 | Analyzer | Circular `ref` detected |
| L038 | Analyzer | `write` used on non-external entity |
| L041 | Analyzer | `now()` used in derived field (invalid context) |
| L042 | Analyzer | `.age` compared to non-duration |
| R001 | Runtime | Access to deleted instance |
| R002 | Runtime | Division by zero |
| R003 | Runtime | Rule re-entrancy / infinite loop |
| R006 | Runtime | `@range` violation |
| R009 | Runtime | Derived field write attempt |

---

## 10. Version History
*   **v1.8.0**: Professional Distribution (Homebrew, DEB, NSIS), "Teaching" diagnostics with mentoring hints, Zero-Config `lumina setup` CLI, High-fidelity website and VS Code extension.
*   **v1.6**: Multi-condition triggers (`and`), Entity Relationships (`ref`), Frequency triggers (`N times within`), `write` actions, `Timestamp` type and `.age`.
*   **v1.5**: LSP, External Entities, `prev()`, Aggregates, `on clear`, Cooldowns, Playground v2.
*   **v1.4**: Functions, Modules, String Interpolation, Lists, Go FFI, REPL v2, Diagnostics.
*   **v1.3**: Core reactive engine, Entities, Rules, Actions, CLI.

---

## 11. Token Terminals
```ebnf
STRING ::= '"' (interpolated_content)* '"'
interpolated_content ::= char | '{' expr '}'
IDENT ::= LETTER (LETTER | DIGIT | '_')*
NUMBER ::= DIGIT+ ('.' DIGIT+)?
BOOL ::= 'true' | 'false'
```
