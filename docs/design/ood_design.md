# SQLRustGo 1.0 OOD 设计

## 1. 设计类图

### 1.1 Parser 模块设计类图

```mermaid
classDiagram
    class Lexer {
        <<interface>>
        +tokenize(sql: &str) Result~Vec~Token~~ LexError
    }
    
    class Parser {
        <<interface>>
        +parse(tokens: &[Token]) Result~AST~ ParseError
        +validate(ast: &AST) Result~()~~ ValidationError
    }
    
    class SqlLexer {
        -keywords: HashSet~String~
        -symbols: HashSet~char~
        +tokenize(sql: &str) Result~Vec~Token~~ LexError
    }
    
    class SqlParser {
        -lexer: Box~dyn Lexer~
        +parse(tokens: &[Token]) Result~AST~ ParseError
        +validate(ast: &AST) Result~()~~ ValidationError
    }
    
    class Token {
        +token_type: TokenType
        +value: String
        +position: Position
    }
    
    class AST {
        +root: Box~dyn Node~
        +statements: Vec~Box~dyn Node~~
        +accept(visitor: &mut dyn Visitor) Result~()~~ Error
    }
    
    class TokenType {
        <<enum>>
        +Keyword
        +Identifier
        +StringLiteral
        +NumberLiteral
        +Operator
        +Delimiter
        +EOF
    }
    
    class Position {
        +line: usize
        +col: usize
    }
    
    class Node {
        <<interface>>
        +accept(visitor: &mut dyn Visitor) Result~()~~ Error
    }
    
    Lexer <|.. SqlLexer : 实现
    Parser <|.. SqlParser : 实现
    SqlParser --> Lexer : 依赖
    SqlParser --> AST : 产生
    AST --> Token : 使用
    Token --> TokenType : 拥有
    Token --> Position : 拥有
    AST --> Node : 组合
    
    note for Lexer "词法分析器接口"
    note for Parser "语法分析器接口"
    note for SqlLexer "SQL词法分析器实现"
    note for SqlParser "SQL语法分析器实现"
```

### 1.2 Executor 模块设计类图

```mermaid
classDiagram
    class ExecutionEngine {
        -storage: FileStorage
        -indexes: HashMap~String, BPlusTree~
        +execute(stmt: Statement) Result~ExecutionResult~~ SqlError
        +execute_select(stmt: SelectStatement) Result~ExecutionResult~~ SqlError
        +execute_insert(stmt: InsertStatement) Result~ExecutionResult~~ SqlError
        +execute_update(stmt: UpdateStatement) Result~ExecutionResult~~ SqlError
        +execute_delete(stmt: DeleteStatement) Result~ExecutionResult~~ SqlError
        +execute_create_table(stmt: CreateTableStatement) Result~ExecutionResult~~ SqlError
        +execute_drop_table(stmt: DropTableStatement) Result~ExecutionResult~~ SqlError
    }
    
    class ExecutionResult {
        +rows_affected: u64
        +columns: Vec~String~
        +rows: Vec~Vec~Value~~
    }
    
    class SelectStatement {
        +columns: Vec~SelectItem~
        +table: String
        +where: Option~Expression~
        +order_by: Option~Vec~OrderByItem~~
    }
    
    class InsertStatement {
        +table: String
        +columns: Option~Vec~String~~
        +values: Vec~Vec~Value~~
    }
    
    class UpdateStatement {
        +table: String
        +updates: Vec~UpdateItem~
        +where: Option~Expression~
    }
    
    class DeleteStatement {
        +table: String
        +where: Option~Expression~
    }
    
    class CreateTableStatement {
        +name: String
        +columns: Vec~ColumnDefinition~
    }
    
    class DropTableStatement {
        +name: String
    }
    
    ExecutionEngine --> ExecutionResult : 返回
    ExecutionEngine --> SelectStatement : 处理
    ExecutionEngine --> InsertStatement : 处理
    ExecutionEngine --> UpdateStatement : 处理
    ExecutionEngine --> DeleteStatement : 处理
    ExecutionEngine --> CreateTableStatement : 处理
    ExecutionEngine --> DropTableStatement : 处理
```

### 1.3 Storage 模块设计类图

```mermaid
classDiagram
    class Storage {
        <<interface>>
        +get_table(name: &str) Option~&TableData~
        +get_table_mut(name: &str) Option~&mut TableData~
        +insert_table(name: String, data: TableData) Result~()~~ SqlError
        +drop_table(name: &str) Result~()~~ SqlError
        +persist_table(name: &str) Result~()~~ SqlError
    }
    
    class FileStorage {
        -data_dir: PathBuf
        -tables: HashMap~String, TableData~
        +new() FileStorage
        +with_data_dir(path: PathBuf) FileStorage
        +load_all() Result~()~~ SqlError
        +save_all() Result~()~~ SqlError
    }
    
    class MemoryStorage {
        -tables: HashMap~String, TableData~
        +new() MemoryStorage
    }
    
    class TableData {
        +info: TableInfo
        +rows: Vec~Vec~Value~~
    }
    
    class TableInfo {
        +name: String
        +columns: Vec~ColumnDefinition~
    }
    
    class ColumnDefinition {
        +name: String
        +data_type: String
        +nullable: bool
        +default_value: Option~Value~
    }
    
    class BPlusTree {
        -root: Option~Box~Node~
        -order: usize
        +new() BPlusTree
        +insert(key: i64, value: usize)
        +search(key: i64) Option~usize~
        +delete(key: i64) bool
        +range_query(start: i64, end: i64) Vec~usize~
    }
    
    Storage <|.. FileStorage : 实现
    Storage <|.. MemoryStorage : 实现
    FileStorage --> TableData : 管理
    MemoryStorage --> TableData : 管理
    TableData *-- TableInfo : 包含
    TableInfo *-- ColumnDefinition : 包含
    FileStorage --> BPlusTree : 使用索引
```

### 1.4 Transaction 模块设计类图

```mermaid
classDiagram
    class TransactionManager {
        -wal: Arc~WriteAheadLog~
        -active_txs: HashMap~u64, TransactionState~
        +begin() Result~u64~~ SqlError
        +commit(tx_id: u64) Result~()~~ SqlError
        +rollback(tx_id: u64) Result~()~~ SqlError
        +is_active(tx_id: u64) bool
        +get_state(tx_id: u64) Option~&TransactionState~
    }
    
    class WriteAheadLog {
        -file: File
        -path: PathBuf
        -records: Vec~WalRecord~
        +new(path: &str) Result~WriteAheadLog~~ SqlError
        +append(record: WalRecord) Result~()~~ SqlError
        +flush() Result~()~~ SqlError
        +recover() Result~Vec~WalRecord~~ SqlError
    }
    
    class WalRecord {
        +tx_id: u64
        +timestamp: DateTime
        +operation: WalOp
        +data: String
    }
    
    class TransactionState {
        +tx_id: u64
        +status: TxStatus
        +operations: Vec~WalOp~
        +created_at: DateTime
    }
    
    class TxStatus {
        <<enum>>
        +Active
        +Committed
        +RolledBack
    }
    
    class WalOp {
        <<enum>>
        +Begin
        +Insert(table, row_data)
        +Update(table, old_row, new_row)
        +Delete(table, row_data)
        +Commit
        +Rollback
    }
    
    TransactionManager --> WriteAheadLog : 依赖
    TransactionManager --> TransactionState : 管理
    WriteAheadLog --> WalRecord : 存储
    WalRecord --> WalOp : 包含
    TransactionState --> TxStatus : 使用
    WalRecord --> TxStatus : 关联
```

---

## 2. 顺序图

### 2.1 SQL 解析顺序图

```mermaid
sequenceDiagram
    participant Client as 客户端
    participant Parser as SqlParser
    participant Lexer as SqlLexer
    participant AST as AST对象
    
    Client->>Parser: parse("SELECT * FROM users")
    Parser->>Lexer: tokenize("SELECT * FROM users")
    Lexer-->>Parser: [Token流: SELECT, *, FROM, users, EOF]
    Parser->>Parser: 语法分析(Shift-Reduce)
    Parser->>AST: 构建SelectStatement节点
    AST-->>Parser: 返回AST
    Parser-->>Client: 返回AST
```

### 2.2 INSERT 执行顺序图

```mermaid
sequenceDiagram
    participant Client as 客户端
    participant Executor as ExecutionEngine
    participant Storage as FileStorage
    participant WAL as WriteAheadLog
    participant Disk as 磁盘
    
    Client->>Executor: execute(INSERT INTO users VALUES (...))
    Executor->>Storage: get_table("users")
    Storage-->>Executor: TableData
    
    opt 事务开启
        Executor->>WAL: append(Begin)
        WAL->>Disk: 写入WAL日志
    end
    
    Executor->>Storage: 验证表结构
    Storage-->>Executor: 验证通过
    Executor->>Storage: 插入行数据
    Executor->>WAL: append(Insert操作)
    WAL->>Disk: 写入WAL日志
    
    opt 事务提交
        Executor->>WAL: append(Commit)
        WAL->>Disk: 写入WAL日志
    end
    
    Executor->>Storage: persist_table("users")
    Storage->>Disk: 写入JSON文件
    Disk-->>Storage: 写入完成
    Storage-->>Executor: 持久化成功
    Executor-->>Client: ExecutionResult{rows_affected: 1}
```

### 2.3 SELECT 执行顺序图（含索引优化）

```mermaid
sequenceDiagram
    participant Client as 客户端
    participant Executor as ExecutionEngine
    participant Storage as FileStorage
    participant BTree as BPlusTree
    participant Disk as 磁盘
    
    Client->>Executor: execute(SELECT * FROM users WHERE id = 5)
    Executor->>Storage: get_table("users")
    Storage-->>Executor: TableData
    
    alt 有索引且条件匹配
        Executor->>BTree: search(5)
        BTree-->>Executor: row_index = 2
        Executor->>Storage: get_row(2)
        Storage-->>Executor: 直接返回该行
    else 无索引
        Executor->>Executor: 全表扫描
        loop 遍历每一行
            Executor->>Executor: WHERE条件求值
        end
    end
    
    Executor->>Executor: 过滤结果行
    Executor-->>Client: ExecutionResult{rows: [...]，columns: [...]}
```

### 2.4 事务提交顺序图

```mermaid
sequenceDiagram
    participant Client as 客户端
    participant TxMgr as TransactionManager
    participant WAL as WriteAheadLog
    participant Storage as FileStorage
    participant Disk as 磁盘
    
    Client->>TxMgr: begin()
    TxMgr->>WAL: append(Begin)
    WAL->>Disk: fsync
    TxMgr-->>Client: tx_id = 1
    
    Client->>TxMgr: execute(操作1)
    TxMgr->>WAL: append(Op1)
    
    Client->>TxMgr: execute(操作2)
    TxMgr->>WAL: append(Op2)
    
    Client->>TxMgr: commit(tx_id=1)
    TxMgr->>WAL: append(Commit)
    WAL->>Disk: fsync WAL日志
    
    TxMgr->>Storage: persist_table(所有修改表)
    Storage->>Disk: 写数据文件
    
    TxMgr-->>Client: 提交成功
```

---

## 3. 状态图

### 3.1 解析器状态图

```mermaid
stateDiagram-v2
    [*] --> Idle
    
    Idle --> Tokenizing: 开始词法分析
    Tokenizing --> Parsing: 词法分析完成
    Tokenizing --> LexError: 词法错误
    LexError --> Idle: 重置
    
    Parsing --> Validating: 语法分析完成
    Parsing --> ParseError: 语法错误
    ParseError --> Idle: 重置
    
    Validating --> Success: 验证通过
    Validating --> SemanticError: 语义错误
    SemanticError --> Idle: 重置
    
    Success --> Idle: 重置
```

### 3.2 事务状态图

```mermaid
stateDiagram-v2
    [*] --> Idle
    
    Idle --> Active: BEGIN
    
    Active --> Active: 执行操作
    Active --> Committed: COMMIT
    Active --> RolledBack: ROLLBACK
    
    Committed --> [*]
    RolledBack --> [*]
    
    note right of Active: 所有操作记录WAL
    note right of Committed: 数据持久化到磁盘
    note right of RolledBack: 内存修改丢弃
```

### 3.3 B+树操作状态图

```mermaid
stateDiagram-v2
    [*] --> Empty
    
    Empty --> NonEmpty: 第一次insert
    NonEmpty --> NonEmpty: insert新键
    NonEmpty --> Full: 节点满
    Full --> Split: 分裂
    Split --> NonEmpty: 分裂完成
    
    NonEmpty --> NonEmpty: search
    NonEmpty --> NonEmpty: range_query
    
    NonEmpty --> NonEmpty: delete存在的键
    NonEmpty --> Error: delete不存在的键
    Error --> NonEmpty: 继续操作
```

---

## 4. 组件图

### 4.1 系统组件图

```mermaid
graph TB
    subgraph SQLRustGo
        subgraph Parser组件
            P1[Lexer模块]
            P2[Parser模块]
            P1 --> P2
        end
        
        subgraph Executor组件
            E1[ExecutionEngine]
            E2[QueryOptimizer]
            E1 --> E2
        end
        
        subgraph Storage组件
            S1[FileStorage]
            S2[BPlusTree]
            S3[TableData]
            S1 --> S2
            S1 --> S3
        end
        
        subgraph Transaction组件
            T1[TransactionManager]
            T2[WriteAheadLog]
            T1 --> T2
        end
        
        subgraph Common组件
            C1[Value类型]
            C2[SqlError]
            C3[SqlResult]
            C4[Statement AST]
        end
    end
    
    P2 --> C4
    E1 --> C4
    E1 --> S1
    E1 --> C1
    E1 --> C2
    T1 --> T2
    T2 --> C2
    S1 --> C1
    S1 --> C2
    
    style P1 fill:#f3e5f5
    style P2 fill:#f3e5f5
    style E1 fill:#e8f5e9
    style E2 fill:#e8f5e9
    style S1 fill:#fce4ec
    style S2 fill:#fce4ec
    style S3 fill:#fce4ec
    style T1 fill:#fff9c4
    style T2 fill:#fff9c4
    style C1 fill:#e3f2fd
    style C2 fill:#e3f2fd
    style C3 fill:#e3f2fd
    style C4 fill:#e3f2fd
```

### 4.2 组件依赖关系

```mermaid
graph LR
    subgraph Parser
        L[Lexer]
        Pa[Parser]
    end
    
    subgraph Executor
        Ex[ExecutionEngine]
    end
    
    subgraph Storage
        St[FileStorage]
    end
    
    subgraph Transaction
        Tx[TransactionManager]
        Wal[WAL]
    end
    
    subgraph Common
        Ty[Types]
        Er[Errors]
        Stmt[Statements]
    end
    
    Pa --> L
    Ex --> Pa
    Ex --> St
    Ex --> Stmt
    Tx --> Wal
    
    L --> Ty
    Pa --> Ty
    Pa --> Stmt
    St --> Ty
    Ex --> Ty
    Ex --> Er
    St --> Er
    Wal --> Er
    Tx --> Er
    
    style L fill:#f3e5f5
    style Pa fill:#f3e5f5
    style Ex fill:#e8f5e9
    style St fill:#fce4ec
    style Tx fill:#fff9c4
    style Wal fill:#fff9c4
    style Ty fill:#e3f2fd
    style Er fill:#e3f2fd
    style Stmt fill:#e3f2fd
```

### 4.3 组件职责表

| 组件 | 依赖 | 被依赖 | 职责 |
|------|------|--------|------|
| **Lexer** | Types | Parser | SQL词法分析，生成Token流 |
| **Parser** | Types, Statements | Executor | SQL语法分析，生成AST |
| **Executor** | Parser, Storage, Types, Errors | main | 查询执行引擎 |
| **Storage** | Types, Errors | Executor | 文件存储和持久化 |
| **Transaction** | WAL, Errors | Executor(可选) | 事务管理 |
| **WAL** | Errors | Transaction | 预写日志 |
| **Types** | 无 | 所有组件 | 基础类型定义 |
| **Errors** | 无 | 所有组件 | 错误类型定义 |
| **Statements** | Types | Parser, Executor | AST节点类型 |
