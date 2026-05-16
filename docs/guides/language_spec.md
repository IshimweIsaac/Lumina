<!-- Merged Language Specification -->

# Extended Backus-Naur Form (EBNF) Specification

This document contains the formal grammar and error registry for Lumina.

## 1. Global Program Structure
```ebnf
program ::= statement* EOF

statement ::= import_stmt
      | fn_decl
      | entity_decl
      | cluster_decl
      | let_stmt
      | rule_decl
      | action_stmt
      | aggregate_decl
      | external_decl
      | NEWLINE

aggregate_decl ::= 'aggregate' IDENT 'over' IDENT '{' NEWLINE (IDENT ':=' aggregate_func NEWLINE)* '}'
aggregate_func ::= ('avg' | 'min' | 'max' | 'sum' | 'count' | 'any' | 'all') '(' IDENT? ')'

cluster_decl ::= 'cluster' '{' NEWLINE 'node_id:' STRING NEWLINE 'bind_addr:' STRING NEWLINE 'peers:' list_literal NEWLINE 'quorum:' NUMBER NEWLINE '}'
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
     | migrate_action
     | deploy_action

show_action ::= 'show' expr
update_action ::= 'update' path 'to' expr
write_action ::= 'write' path '=' expr
create_action ::= 'create' IDENT '{' NEWLINE (IDENT ':' expr NEWLINE)* '}'
delete_action ::= 'delete' IDENT
alert_action ::= 'alert' 'severity' ':' STRING (',' 'message' ':' STRING)? (',' 'source' ':' expr)? (',' 'code' ':' STRING)? (',' 'payload' ':' '{' ... '}')?
migrate_action ::= 'migrate' '{' 'workloads:' expr ',' 'target:' expr '}'
deploy_action ::= 'deploy' '{' 'workloads:' expr ',' 'target:' expr '}'

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
*  ****: The Cluster Release. Added `cluster_decl`, `migrate` and `deploy` actions, native UDP Gossip networking, StateMesh for conflict-free replication, and FxHashMap performance upgrades.
*  ****: Professional Distribution (Homebrew, DEB, NSIS), "Teaching" diagnostics with mentoring hints, Zero-Config `lumina setup` CLI, High-fidelity website and VS Code extension.
*  ****: Multi-condition triggers (`and`), Entity Relationships (`ref`), Frequency triggers (`N times within`), `write` actions, `Timestamp` type and `.age`.
*  ****: LSP, External Entities, `prev()`, Aggregates, `on clear`, Cooldowns, Playground v2.
*  ****: Functions, Modules, String Interpolation, Lists, Go FFI, REPL v2, Diagnostics.
*  ****: Core reactive engine, Entities, Rules, Actions, CLI.

---

## 11. Token Terminals
```ebnf
STRING ::= '"' (interpolated_content)* '"'
interpolated_content ::= char | '{' expr '}'
IDENT ::= LETTER (LETTER | DIGIT | '_')*
NUMBER ::= DIGIT+ ('.' DIGIT+)?
BOOL ::= 'true' | 'false'
```


---

# Lumina System Architecture

The Lumina runtime engine is designed for absolute correctness and deterministic reactivity. Version 2.0, the "Cluster Release," transforms the language from a single-node engine into a distributed mesh orchestrator, supporting high-availability and workload migration.

---

## 1. Compiler Pipeline Overview

Lumina programs are processed through a strictly ordered pipeline. Every stage is designed for zero-allocation performance and strong safety guarantees.

### 1.1 Lexical Analysis (`lumina-lexer`)
Tokenization is performed via the `logos` crate, generating a high-speed Deterministic Finite Automaton (DFA). Version 2.0 adds optimized tokenization for multi-line strings and duration literals.

### 1.2 Syntax Analysis (`lumina-parser`)
The parser maps tokens into a structured Abstract Syntax Tree (AST):
*  **Recursive Descent**: Handles declarative constructs (`entity`, `rule`, `fn`).
*  **Pratt Parsing**: For expressions, managing complex operator precedence and accessor logic.

### 1.3 Semantic Analysis (`lumina-analyzer`)
The analyzer performs two distinct passes:
1. **Declaration Registration**: Records all entities, fields, and pure functions.
2. **Structural Integrity & Typecheck**: Validates expressions and constructs a topological `DependencyGraph`.
3. **Cyclic Dependency Detection**: Ensures all derived fields form a Directed Acyclic Graph (DAG) for deterministic propagation.

---

## 2. The Reactive Engine (`lumina-runtime`)

### 2.1 Snapshot-Based Virtual Machine
Lumina implements a **Self-Healing Guarantee**. Before any destructive action:
1. The VM takes a complete memory **Snapshot**.
2. Evaluation proceeds. If a recursion limit (100) or invariant is breached, the runtime **Automatically Rolls Back** to the snapshot.
3. **Diagnostic Reporting**: Instead of crashing, the engine returns a structured `Diagnostic` object to the host system.

### 2.2 Incremental Aggregates
Fleet-level summaries (`avg`, `sum`, `count`) are updated **incrementally**.
*  **O(1) Evaluation**: When an instance updates, the aggregate counters are adjusted in constant time, rather than re-scanning the entire fleet.
*  **Reactive Flow**: Aggregates are integrated into the main dependency graph, allowing derived fields to depend on fleet-level metrics.

### 2.3 Temporal Engine & Stabilization
the Lumina temporal engine is the most stable version yet.
*  **Unified TimerHeap**: Manages both `every` intervals and `for` duration stabilization.
*  **Edge Detection**: The engine maintains a transition cache, firing rules only on precise state transitions (`becomes`).

### 2.4 The Distributed Cluster Engine (`lumina-cluster`)
Version 2.0 introduces native multi-node orchestration directly integrated into the language runtime.
*  **Gossip Protocol (`UdpTransport`)**: Nodes discover each other and broadcast state changes using a custom, high-speed UDP transport layer. This enables real-time peer health monitoring and message routing.
*  **StateMesh**: A conflict-free replicated data structure. When nodes broadcast state updates, the `StateMesh` resolves conflicts using Last-Write-Wins (LWW) and version vectors, ensuring eventual consistency across the cluster.
*  **Raft-Inspired Election**: The cluster requires a defined `quorum` to elect a leader. This leader orchestrates cross-node actions like `migrate` and `deploy`.
*  **Memory Scaling (`FxHashMap`)**: To support high-throughput network syncing, the core indexers and entity stores have been migrated from standard cryptographically secure maps to `FxHashMap`, allowing for O(1) tracking of 100k+ entities with negligible RAM bloat.

---

## 3. Platform & Distribution

### 3.1 WASM Bridge (`lumina-wasm`)
The WASM layer provides a high-performance interface for browser embedding.
*  **Deterministic Evaluation**: The exact same Rust engine runs in the browser, ensuring simulation parity.
*  **JS Integration**: Optimized serialization allow for sub-millisecond event injection from React or Vanilla JS frontends.

### 3.2 Polyglot FFI (`lumina_ffi`)
The stable C ABI enables integration with any language:
*  **C-Compatible Interface**: Exports functions for creation, ticking, and state export.
*  **Memory Safety**: Enforces strict ownership rules across the FFI boundary to prevent leaks.

### 3.3 LSP v2 (`lumina-lsp`)
the Lumina Language Server provides production-grade IDE support:
*  **Live Diagnostics**: Real-time type checking and cycle detection.
*  **Navigation**: "Go to Definition" and "Find All References" for complex data flows.

---

## 4. Technical Stack
*  **Rust**: Deterministic performance and memory safety.
*  **Logos**: DFA-based high-performance lexing.
*  **Pratt Parsing**: Precedence-climbing expression evaluation.
*  **Snapshot VM**: Atomic state transitions with guaranteed rollback.
*  **Serde**: Efficient serialization for WASM and FFI boundaries.

