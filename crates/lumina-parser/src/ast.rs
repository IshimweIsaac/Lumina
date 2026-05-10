use lumina_lexer::token::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

// ── Top-level program ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub span: Span,
}

impl Program {
    pub fn imports(&self) -> impl Iterator<Item = &ImportDecl> {
        self.statements.iter().filter_map(|s| {
            if let Statement::Import(i) = s {
                Some(i)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Entity(EntityDecl),
    ExternalEntity(ExternalEntityDecl),
    Let(LetStmt),
    Rule(RuleDecl),
    Action(Action),
    Fn(FnDecl),
    Import(ImportDecl),
    Aggregate(AggregateDecl),
    PluginImport(PluginImportDecl),
    Provider(ProviderDecl),
    Cluster(ClusterDecl),
}

// ── Function declaration ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<FnParam>,
    pub returns: LuminaType,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnParam {
    pub name: String,
    pub type_: LuminaType,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDecl {
    pub path: String,
    /// v1.9: Optional LSL namespace segments, e.g. ["LSL", "datacenter", "Server"]
    pub namespace: Option<Vec<String>>,
    pub span: Span,
}

// ── Plugin import declaration (v1.8) ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginImportDecl {
    pub path: String,
    pub alias: String,
    pub span: Span,
}

// ── Provider declaration (v1.9) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDecl {
    pub protocol: String,
    pub config: Vec<ProviderConfigEntry>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigEntry {
    pub key: String,
    pub value: Expr,
    pub span: Span,
}

// ── Cluster declaration (v2.0) ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterDecl {
    pub node_id: String,
    pub bind_addr: String,
    pub peers: Vec<String>,
    pub quorum: u32,
    pub election_timeout: Duration,
    pub span: Span,
}

// ── Entity declaration ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDecl {
    pub name: String,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Field {
    Stored(StoredField),
    Derived(DerivedField),
    Ref(RefField),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefField {
    pub name: String,
    pub target_entity: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredField {
    pub name: String,
    pub ty: LuminaType,
    pub metadata: FieldMetadata,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedField {
    pub name: String,
    pub expr: Expr,
    pub metadata: FieldMetadata,
    pub span: Span,
}

// ── Field metadata (@doc / @range / @affects) ──────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMetadata {
    pub doc: Option<String>,
    pub range: Option<(f64, f64)>,
    pub affects: Vec<String>,
}

// ── Type system ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LuminaType {
    Text,
    Number,
    Boolean,
    Timestamp,
    Duration,
    Secret,
    Entity(String),
    List(Box<LuminaType>),
}

// ── External entity ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEntityDecl {
    pub name: String,
    pub fields: Vec<Field>,
    pub sync_path: String,
    pub sync_strategy: SyncStrategy,
    pub sync_fields: Vec<String>,
    pub poll_interval: Option<Duration>,
    pub sync_timeout: Option<Duration>,
    pub fallible: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStrategy {
    Realtime,
    Poll,
    Webhook,
}

// ── Let statement ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LetStmt {
    pub name: String,
    pub value: LetValue,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LetValue {
    Expr(Expr),
    EntityInit(EntityInit),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInit {
    pub entity_name: String,
    pub fields: Vec<(String, Expr)>,
    pub span: Span,
}

// ── Rule declaration ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDecl {
    pub name: String,
    pub param: Option<RuleParam>,
    pub trigger: RuleTrigger,
    pub actions: Vec<Action>,
    pub cooldown: Option<Duration>,
    pub on_clear: Option<Vec<Action>>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleParam {
    pub name: String,
    pub entity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleTrigger {
    When(Vec<Condition>),
    Any(FleetCondition),
    All(FleetCondition),
    Every(Duration),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub expr: Expr,
    pub becomes: Option<Expr>,
    pub for_duration: Option<Duration>,
    pub frequency: Option<Frequency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetCondition {
    pub entity: String,
    pub field: String,
    pub becomes: Expr,
    pub for_duration: Option<Duration>,
    pub frequency: Option<Frequency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frequency {
    pub count: u32,
    pub within: Duration,
    pub span: Span,
}

// ── Duration (for temporal rules) ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duration {
    pub value: f64,
    pub unit: TimeUnit,
}

impl Duration {
    pub fn to_seconds(&self) -> f64 {
        match self.unit {
            TimeUnit::Seconds => self.value,
            TimeUnit::Minutes => self.value * 60.0,
            TimeUnit::Hours => self.value * 3600.0,
            TimeUnit::Days => self.value * 86400.0,
        }
    }

    pub fn to_std_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(self.to_seconds())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

// ── Actions ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Show(Expr),
    Update {
        target: FieldPath,
        value: Expr,
    },
    Write {
        target: FieldPath,
        value: Expr,
    },
    Create {
        entity: String,
        fields: Vec<(String, Expr)>,
    },
    Delete(String),
    Alert(AlertAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAction {
    pub severity: Expr,
    pub message: Expr,
    pub source: Option<Expr>,
    pub code: Option<Expr>,
    pub payload: Vec<(String, Expr)>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPath {
    pub instance: String,
    pub field: String,
    pub sub_field: Option<String>,
    pub span: Span,
}

// ── Expressions ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number(f64),
    Text(String),
    Bool(bool),
    Ident(String),
    FieldAccess {
        obj: Box<Expr>,
        field: String,
        span: Span,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
        span: Span,
    },
    If {
        cond: Box<Expr>,
        then_: Box<Expr>,
        else_: Box<Expr>,
        span: Span,
    },
    InterpolatedString(Vec<StringSegment>),
    Call {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    ListLiteral(Vec<Expr>),
    Index {
        list: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Duration(Duration),
    Prev {
        field: String,
        span: Span,
    },
    /// v2.0: Access cluster state — `cluster.{node_id}.{field}` or `cluster.all.{field}`
    ClusterAccess {
        node_id: String,
        field: String,
        span: Span,
    },
    /// v2.0: Orchestration — `migrate(workloads, to: target)`
    Migrate {
        workloads: Box<Expr>,
        target: Box<Expr>,
        span: Span,
    },
    /// v2.0: Orchestration — `evacuate(entities)`
    Evacuate {
        entities: Box<Expr>,
        span: Span,
    },
    /// v2.0: Orchestration — `deploy(spec)`
    Deploy {
        spec: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringSegment {
    Literal(String),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    And,
    Or,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Gt => ">",
            BinOp::Lt => "<",
            BinOp::Ge => ">=",
            BinOp::Le => "<=",
            BinOp::And => "and",
            BinOp::Or => "or",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnOp {
    Neg,
    Not,
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnOp::Neg => "-",
            UnOp::Not => "not",
        };
        write!(f, "{}", s)
    }
}

// ── Aggregates ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateDecl {
    pub name: String,
    pub over: String,
    pub fields: Vec<AggregateField>,
    pub scope: AggregateScope,
    pub span: Span,
}

/// v2.0: Scope for aggregate computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregateScope {
    /// Local node only (v1.x default)
    Local,
    /// Across the entire cluster
    Cluster,
    /// Scoped to a specific region
    Region(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateField {
    pub name: String,
    pub expr: AggregateExpr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregateExpr {
    Avg(String),
    Min(String),
    Max(String),
    Sum(String),
    Count(Option<String>),
    Any(String),
    All(String),
}
