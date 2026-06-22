# SQLRustGo 1.0 OOA 分析

## 1. 用例图

### 1.1 Parser 模块用例图

```mermaid
graph LR
    subgraph 客户端
        Client((客户端))
    end
    
    subgraph Parser模块
        direction TB
        U1[词法分析]
        U2[语法分析]
        U3[生成AST]
        U4[SQL验证]
    end
    
    Client --> U1
    Client --> U2
    Client --> U3
    Client --> U4
    
    U1 -.->|包含| U2
    U2 -.->|包含| U3
    U4 -.->|扩展| U2
    
    style Client fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    style U1 fill:#c8e6c9
    style U2 fill:#c8e6c9
    style U3 fill:#c8e6c9
    style U4 fill:#fff3e0,stroke:#ff9800,stroke-dasharray: 5 5
```

### 1.2 完整系统用例图

```mermaid
graph TB
    subgraph 参与者
        Client((客户端))
        User((最终用户))
    end
    
    subgraph SQLRustGo系统
        direction TB
        
        subgraph 解析层
            UC1[词法分析]
            UC2[语法分析]
            UC3[SQL验证]
        end
        
        subgraph 执行层
            UC4[查询执行]
            UC5[数据插入]
            UC6[数据更新]
            UC7[数据删除]
        end
        
        subgraph 存储层
            UC8[表创建]
            UC9[表删除]
            UC10[持久化存储]
        end
        
        subgraph 事务层
            UC11[事务开始]
            UC12[事务提交]
            UC13[事务回滚]
        end
    end
    
    Client --> UC1
    Client --> UC2
    Client --> UC4
    Client --> UC5
    Client --> UC6
    Client --> UC7
    Client --> UC8
    Client --> UC9
    
    User --> UC10
    User --> UC11
    User --> UC12
    User --> UC13
    
    UC1 ==> UC2
    UC2 ==> UC4
    
    style Client fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    style User fill:#fff3e0,stroke:#ff9800,stroke-width:2px
```

---

## 2. 概念类图

### 2.1 Parser 模块概念类图

```mermaid
classDiagram
    class SQL语句 {
        +String content
        +parse()
    }
    
    class 词法单元 {
        +TokenType type
        +String value
        +int line
        +int col
    }
    
    class 抽象语法树 {
        +Node root
        +Vec~Node~ nodes
        +addNode()
        +getRoot()
    }
    
    class 语法节点 {
        +NodeType type
        +String name
        +Vec~Node~ children
        +Node parent
        +addChild()
        +hasChildren()
    }
    
    class 语法错误 {
        +String message
        +int line
        +int col
        +String hint
    }
    
    SQL语句 "1" --> "*" 词法单元 : 包含
    词法单元 "*" --> "1" 抽象语法树 : 构建
    抽象语法树 "1" --> "*" 语法节点 : 由...组成
    语法错误 "*" -- "1" 抽象语法树 : 关联
    语法节点 --> 语法节点 : 父子关系
    
    note for SQL语句 "用户输入的原始SQL文本"
    note for 词法单元 "词法分析的输出，如 SELECT, FROM, WHERE"
    note for 抽象语法树 "SQL语句的结构化表示"
    note for 语法节点 "AST中的单个节点，如TableNode, WhereNode"
    note for 语法错误 "解析过程中产生的错误信息"
```

### 2.2 数据类型概念类图

```mermaid
classDiagram
    class Value {
        <<enum>>
        +Null
        +Boolean(bool)
        +Integer(i64)
        +Float(f64)
        +Text(String)
        +Blob(Vec~u8~)
    }
    
    class ColumnDefinition {
        +String name
        +String data_type
        +bool nullable
    }
    
    class TableInfo {
        +String name
        +Vec~ColumnDefinition~ columns
    }
    
    class TableData {
        +TableInfo info
        +Vec~Vec~Value~~ rows
    }
    
    class ExecutionResult {
        +u64 rows_affected
        +Vec~String~ columns
        +Vec~Vec~Value~~ rows
    }
    
    class SqlError {
        <<enum>>
        +ParseError
        +ExecutionError
        +TypeMismatch
        +TableNotFound
    }
    
    TableData *-- TableInfo : 包含
    TableInfo *-- ColumnDefinition : 拥有
    TableData o-- Value : 包含
    ExecutionResult o-- Value : 包含
    SqlError ..> Value : 可能返回
```

---

## 3. 活动图

### 3.1 SQL 解析活动图

```mermaid
flowchart TD
    A([开始]) --> B[接收SQL语句]
    B --> C[词法分析生成Token流]
    C --> D{语法正确?}
    D -->|是| E[语法分析构建AST]
    D -->|否| F[生成语法错误]
    E --> G{语义正确?}
    G -->|是| H[输出AST]
    G -->|否| I[生成语义错误]
    F --> J([结束])
    H --> J
    I --> J
    
    style A fill:#c8e6c9,stroke:#2e7d32
    style J fill:#ffcdd2,stroke:#c62828
    style D fill:#fff9c4
    style G fill:#fff9c4
```

### 3.2 INSERT 语句执行活动图

```mermaid
flowchart TD
    A([开始]) --> B[接收INSERT语句AST]
    B --> C[解析表名和列名]
    C --> D{表存在?}
    D -->|是| E[验证列是否存在]
    D -->|否| F[返回TableNotFound错误]
    E --> G{列匹配?}
    G -->|是| H[验证值类型]
    G -->|否| I[返回ColumnNotFound错误]
    H --> J{类型匹配?}
    J -->|是| K[插入行数据到存储]
    J -->|否| L[返回TypeMismatch错误]
    K --> M[持久化数据到磁盘]
    M --> N[返回成功结果]
    
    F --> Z([结束])
    I --> Z
    L --> Z
    N --> Z
    Z([结束])
    
    style A fill:#c8e6c9,stroke:#2e7d32
    style Z fill:#ffcdd2,stroke:#c62828
```

### 3.3 SELECT 语句执行活动图

```mermaid
flowchart TD
    A([开始]) --> B[接收SELECT语句AST]
    B --> C[解析目标表名]
    C --> D{表存在?}
    D -->|否| E[返回TableNotFound错误]
    D -->|是| F{有索引?}
    F -->|是| G[使用B+Tree索引查询]
    F -->|否| H[全表扫描]
    G --> I[WHERE过滤]
    H --> I
    I --> J[ORDER BY排序]
    J --> K[生成结果集]
    K --> L[返回ExecutionResult]
    
    E --> Z([结束])
    L --> Z
    
    style A fill:#c8e6c9,stroke:#2e7d32
    style Z fill:#ffcdd2,stroke:#c62828
```

### 3.4 事务执行活动图

```mermaid
flowchart TD
    A([开始]) --> B[BEGIN事务]
    B --> C[记录WAL BEGIN日志]
    C --> D[执行操作]
    D --> E{还有操作?}
    E -->|是| F[记录WAL操作日志]
    F --> D
    E -->|否| G{COMMIT or ROLLBACK?}
    G -->|COMMIT| H[记录WAL COMMIT日志]
    G -->|ROLLBACK| I[记录WAL ROLLBACK日志]
    H --> J[持久化数据]
    I --> K[丢弃内存修改]
    J --> L([事务结束])
    K --> L
    
    style A fill:#c8e6c9,stroke:#2e7d32
    style L fill:#c8e6c9,stroke:#2e7d32
```

---

## 4. 模块职责映射

| 模块 | 用例 | 概念类 | 活动 |
|------|------|--------|------|
| **Lexer** | 词法分析 | SQL语句、词法单元 | 词法分析生成Token流 |
| **Parser** | 语法分析、SQL验证 | 抽象语法树、语法节点、语法错误 | 语法分析构建AST |
| **Executor** | 查询执行、数据CRUD | ExecutionResult | SELECT/INSERT/UPDATE/DELETE执行流程 |
| **Storage** | 表管理、持久化 | TableData、TableInfo、ColumnDefinition、Value | 数据存取和持久化 |
| **Transaction** | 事务管理 | - | 事务执行流程 |
| **Types** | - | Value、SqlError、SqlResult | - |
