# SQLRustGo 1.0 Storage 模块设计

## 一、OOA 分析

### 1. 用例图

```mermaid
graph LR
    subgraph Executor
        E((Executor))
    end
    
    subgraph Storage模块
        U1[读取数据]
        U2[写入数据]
        U3[扫描数据]
        U4[管理表结构]
    end
    
    E --> U1
    E --> U2
    E --> U3
    E --> U4
    
    U2 -.->|持久化| U4
    U3 -.->|需要| U1
    
    style E fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    style U1 fill:#c8e6c9
    style U2 fill:#fff3e0
    style U3 fill:#fff3e0
    style U4 fill:#f3e5f5
```

### 2. 概念类图

```mermaid
classDiagram
    class 存储引擎 {
        +read(table, key) Result~Record~~ StorageError~
        +write(table, record) Result~()~~ StorageError~
        +scan(table, filter) Result~ScanIterator~~
        +create_table(name, schema) Result~()~~ StorageError~
    }
    
    class 表 {
        +name: String
        +schema: TableSchema
        +rows: Vec~Row~
        +indexes: HashMap~String, BPlusTree~~
    }
    
    class 索引 {
        +name: String
        +column: String
        +tree: BPlusTree~Key, RowIdx~
        +insert(key, row_idx)
        +search(key) Option~RowIdx~
        +range(start, end) Vec~RowIdx~
    }
    
    class 页面 {
        +page_id: PageId
        +data: Vec~u8~
        +dirty: bool
        +pin_count: i32
    }
    
    class 缓冲池 {
        -frames: Vec~PageFrame~
        -lru: LruCache
        -capacity: usize
        +new(capacity) BufferPool
        +get_page(page_id) Result~&Page~~
        +put_page(page) Result~()~~
        +flush_all() Result~()~~
        +evict() Option~Page~
    }
    
    class 事务 {
        +tx_id: u64
        +status: TxStatus
        +operations: Vec~WalOp~
        +timestamp: DateTime
    }
    
    存储引擎 "1" --> "*" 表
    表 "1" --> "*" 索引
    表 "1" --> "*" 页面
    存储引擎 "1" --> "1" 缓冲池
    存储引擎 "1" --> "*" 事务
    
    note for 存储引擎 "存储模块核心入口"
    note for 表 "逻辑上的表结构+数据行"
    note for 索引 "B+树实现的辅助索引"
    note for 页面 "4KB固定大小数据页"
    note for 缓冲池 "LRU策略的页面缓存"
    note for 事务 "WAL事务记录"
```

### 3. 活动图

```mermaid
flowchart TD
    A([开始]) --> B[接收存储请求]
    B --> C{请求类型?}
    C -->|读取| D[读操作<br/>查缓冲池 → 命中返回/未命中从磁盘加载]
    C -->|写入| E[写操作<br/>更新内存 + WAL日志 + 标记脏页]
    C -->|扫描| F[扫描操作<br/>迭代所有行 / 利用索引定位]
    C -->|管理| G[元数据操作<br/>CREATE TABLE/DROP TABLE]
    D --> H[返回结果]
    E --> H
    F --> H
    G --> H
    H --> I([结束])
    
    style A fill:#c8e6c9,stroke:#2e7d32
    style I fill:#c8e6c9,stroke:#2e7d32
    style C fill:#fff9c4
```

---

## 二、OOD 设计

### 1. 设计类图

```mermaid
classDiagram
    class StorageEngine {
        <<interface>>
        +read(table: &str, key: &Key) Result~Record~~ StorageError~
        +write(table: &str, record: Record) Result~()~~ StorageError~
        +scan(table: &str, filter: Option~&Predicate~) Result~ScanIterator~~
        +create_table(name: &str, schema: &TableSchema) Result~()~~ StorageError~
        +drop_table(name: &str) Result~()~~ StorageError~
        +begin_transaction() Result~u64~~ StorageError~
        +commit_transaction(tx_id: u64) Result~()~~ StorageError~
        +rollback_transaction(tx_id: u64) Result~()~~ StorageError~
    }
    
    class MemoryStorage {
        -tables: HashMap~String, TableData~
        -transactions: HashMap~u64, TransactionState~
        +read(table, key) Result~Record~~ StorageError~
        +write(table, record) Result~()~~ StorageError~
        +scan(table, filter) Result~ScanIterator~~
        +create_table(name, schema) Result~()~~ StorageError~
    }
    
    class FileStorage {
        -data_dir: PathBuf
        -tables: HashMap~String, TableData~
        -indexes: HashMap~String, BPlusTree~~
        -buffer_pool: BufferPool
        -wal: Arc~WriteAheadLog~
        -next_tx_id: AtomicU64
        +new() FileStorage
        +with_data_dir(path) FileStorage
        +load_all() Result~()~~ StorageError~
        +save_all() Result~()~~ StorageError~
        +read(table, key) Result~Record~~ StorageError~
        +write(table, record) Result~()~~ StorageError~
        +scan(table, filter) Result~ScanIterator~~
        +create_table(name, schema) Result~()~~ StorageError~
    }
    
    class BufferPool {
        -frames: Vec~PageFrame~
        -lru: VecDeque~PageId~
        -page_map: HashMap~PageId, usize~
        -capacity: usize
        -dirty_pages: HashSet~PageId~
        +new(capacity) BufferPool
        +get_page(page_id) Result~&Page~~ BufferPoolError~
        +put_page(page) Result~()~~ BufferPoolError~
        +mark_dirty(page_id)
        +flush_all() Result~()~~ BufferPoolError~
        +evict() Option~PageFrame~
        -update_lru(page_id)
    }
    
    class TableData {
        +info: TableInfo
        +rows: Vec~Vec~Value~~
        +indexes: HashMap~String, BPlusTree~~
        +to_json() String
        +from_json(json: &str) TableData
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
        +is_primary_key: bool
    }
    
    class BPlusTree {
        -root: Option~Box~Node~
        -order: usize
        -size: usize
        +new() BPlusTree
        +insert(key: i64, value: usize)
        +search(key: i64) Option~usize~
        +delete(key: i64) bool
        +range_query(start, end) Vec~usize~
        +is_empty() bool
        +len() usize
    }
    
    class WriteAheadLog {
        -file: Mutex~File~
        -dir: PathBuf
        -current_segment: Option~PathBuf~
        +new(dir) Result~WriteAheadLog~~ StorageError~
        +append(record: WalRecord) Result~()~~ StorageError~
        +flush() Result~()~~ StorageError~
        +recover() Result~Vec~WalRecord~~ StorageError~
    }
    
    class WalRecord {
        +tx_id: u64
        +timestamp: i64
        +op_type: WalOpType
        +table_name: String
        +row_data: Option~String~
    }
    
    StorageEngine <|.. MemoryStorage : 实现
    StorageEngine <|.. FileStorage : 实现
    FileStorage --> BufferPool : 使用
    FileStorage --> WriteAheadLog : 事务支持
    FileStorage --> TableData : 管理
    TableData *-- TableInfo : 包含
    TableInfo *-- ColumnDefinition : 拥有
    TableData --> BPlusTree : 索引
    WriteAheadLog *-- WalRecord : 存储
    
    note for MemoryStorage "纯内存存储，测试/开发场景"
    note for FileStorage "JSON文件持久化，1.0生产方案"
    note for BufferPool "LRU页面缓存，1.0标记未来功能"
    note for WriteAheadLog "预写日志，事务崩溃恢复"
```

### 2. 顺序图

```mermaid
sequenceDiagram
    actor Executor as Executor
    participant Storage as FileStorage
    participant BP as BufferPool
    participant WAL as WriteAheadLog
    participant Disk as JSON文件
    
    Note over Executor,Disk: 读路径
    Executor->>Storage: scan("users", filter)
    Storage->>BP: get_page(users_page_1)
    alt 缓冲池命中
        BP-->>Storage: Page(rows数据)
    else 缓冲池未命中
        Storage->>Disk: 读users.json
        Disk-->>Storage: 文件内容
        Storage->>BP: put_page(users_page_1)
    end
    Storage-->>Executor: ScanIterator
    
    Note over Executor,Disk: 写路径
    Executor->>Storage: write("users", new_row)
    Storage->>WAL: append(Insert操作记录)
    WAL->>Disk: 写WAL日志文件
    Disk-->>WAL: fsync完成
    Storage->>Storage: 更新内存rows + 更新索引
    Storage->>BP: mark_dirty(users_page)
    Storage->>Disk: 写入users.json
    Disk-->>Storage: 写入完成
    Storage-->>Executor: Ok(())
```

### 3. 状态图

```mermaid
stateDiagram-v2
    [*] --> Idle
    
    Idle --> Reading : read()或scan()调用
    Reading --> Caching : 查缓冲池
    Caching --> Reading : 命中
    Caching --> DiskRead : 未命中→读磁盘
    DiskRead --> Reading : 加载到缓冲池
    Reading --> Idle : 读完成
    
    Idle --> Writing : write()调用
    Writing --> WalAppend : 写WAL日志
    WalAppend --> DiskWrite : fsync WAL
    DiskWrite --> MemoryUpdate : 更新内存
    MemoryUpdate --> DiskFlush : 刷脏页到磁盘
    DiskFlush --> Idle : 写完成
    
    Idle --> Managing : create_table/drop_table
    Managing --> Idle : 管理完成
    
    Idle --> TxBegin : begin_transaction()
    TxBegin --> TxActive
    TxActive --> TxCommit : commit()
    TxActive --> TxRollback : rollback()
    TxCommit --> Idle : 持久化
    TxRollback --> Idle : 丢弃
    
    Writing --> Error : 写失败
    Reading --> Error : 读失败
    Error --> Idle : 重置
```

### 4. 组件图

```mermaid
graph TB
    subgraph Storage模块
        subgraph 存储引擎
            S1[FileStorage]
            S2[MemoryStorage]
        end
        
        subgraph 索引层
            I1[BPlusTree索引]
        end
        
        subgraph 缓冲层
            BP[BufferPool<br/>LRU缓存]
        end
        
        subgraph 事务层
            WAL[WriteAheadLog]
        end
        
        subgraph 持久化层
            F1[JSON表文件]
            F2[WAL日志文件]
        end
    end
    
    subgraph Common
        C1[TableData]
        C2[TableInfo]
        C3[ColumnDefinition]
        C4[Value]
        C5[SqlError]
    end
    
    S1 --> I1
    S1 --> BP
    S1 --> WAL
    S1 --> F1
    WAL --> F2
    S2 --> C1
    
    S1 --> C1
    I1 --> C5
    C1 --> C2
    C2 --> C3
    
    style S1 fill:#e8f5e9
    style S2 fill:#e8f5e9
    style I1 fill:#fff3e0
    style BP fill:#f3e5f5
    style WAL fill:#fff9c4
    style F1 fill:#e0e0e0
    style F2 fill:#e0e0e0
    style C1 fill:#e3f2fd
```

---

## 三、详细设计文档

### 1. 模块概述

Storage 模块是 SQLRustGo 的数据持久化核心，负责表数据的存储、读取、扫描和索引管理。1.0版本采用 **JSON文件存储** 格式，结合 **B+Tree索引** 加速查询，通过 **WAL预写日志** 保障事务一致性。

**设计目标：**
- 1.0版本：JSON文件存储 + B+Tree索引 + WAL事务
- 1.1版本：二进制页存储 + BufferPool缓存
- 1.2版本：MVCC + 分布式存储

### 2. 核心功能

| 功能 | 描述 | 1.0状态 |
|------|------|---------|
| 表存储 | JSON文件格式持久化表数据 | ✅ 实现 |
| 表加载 | 启动时自动加载所有表 | ✅ 实现 |
| 行读取 | 按索引或位置读取单行 | ✅ 实现 |
| 行写入 | 插入/更新/删除行 | ✅ 实现 |
| 全表扫描 | 迭代所有行数据 | ✅ 实现 |
| B+Tree索引 | INTEGER类型索引 | ✅ 实现 |
| WAL事务 | 预写日志崩溃恢复 | ✅ 实现 |
| 缓冲池 | LRU页面缓存 | ❌ 标记未来功能 |
| 页面管理 | 4KB固定页 | ❌ 标记未来功能 |
| 表创建 | CREATE TABLE持久化 | ✅ 实现 |
| 表删除 | DROP TABLE持久化 | ✅ 实现 |

### 3. 类与接口设计

#### 3.1 核心接口

```rust
pub trait StorageEngine {
    fn read(&self, table: &str, key: &Value) -> SqlResult<Option<Vec<Value>>>;
    fn write(&mut self, table: &str, row: Vec<Value>) -> SqlResult<()>;
    fn scan(&self, table: &str, filter: Option<&Predicate>) -> SqlResult<Vec<Vec<Value>>>;
    fn update(&mut self, table: &str, key: &Value, new_row: Vec<Value>) -> SqlResult<()>;
    fn delete(&mut self, table: &str, key: &Value) -> SqlResult<()>;
    fn create_table(&mut self, name: &str, schema: TableInfo) -> SqlResult<()>;
    fn drop_table(&mut self, name: &str) -> SqlResult<()>;
    fn get_table(&self, name: &str) -> Option<&TableData>;
    fn get_table_mut(&mut self, name: &str) -> Option<&mut TableData>;
    fn persist_table(&self, name: &str) -> SqlResult<()>;
}
```

#### 3.2 FileStorage 实现

```rust
pub struct FileStorage {
    pub data_dir: PathBuf,
    tables: HashMap<String, TableData>,
    indexes: HashMap<String, HashMap<String, BPlusTree<i64, usize>>>,
    wal: Option<Arc<Mutex<WriteAheadLog>>>,
}

impl FileStorage {
    pub fn new() -> Self {
        Self {
            data_dir: PathBuf::from("data"),
            tables: HashMap::new(),
            indexes: HashMap::new(),
            wal: None,
        }
    }
    
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self { data_dir, tables: HashMap::new(), indexes: HashMap::new(), wal: None }
    }
    
    pub fn load_all(&mut self) -> SqlResult<()> {
        if !self.data_dir.exists() {
            std::fs::create_dir_all(&self.data_dir)?;
            return Ok(());
        }
        for entry in std::fs::read_dir(&self.data_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)?;
                let table: TableData = serde_json::from_str(&content)?;
                let name = table.info.name.clone();
                self.tables.insert(name, table);
            }
        }
        Ok(())
    }
    
    pub fn save_all(&self) -> SqlResult<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        for (name, table) in &self.tables {
            let json = table.to_json();
            let path = self.data_dir.join(format!("{}.json", name));
            std::fs::write(path, json)?;
        }
        Ok(())
    }
    
    pub fn create_index(&mut self, table: &str, column: &str) -> SqlResult<()> {
        let table_data = self.tables.get(table)
            .ok_or_else(|| SqlError::TableNotFound(table.to_string()))?;
        let col_idx = table_data.info.columns.iter()
            .position(|c| c.name == column)
            .ok_or_else(|| SqlError::ColumnNotFound(column.to_string()))?;
        
        let mut tree: BPlusTree<i64, usize> = BPlusTree::new();
        for (row_idx, row) in table_data.rows.iter().enumerate() {
            if let Some(Value::Integer(key)) = row.get(col_idx) {
                tree.insert(*key, row_idx);
            }
        }
        
        self.indexes
            .entry(table.to_string())
            .or_default()
            .insert(column.to_string(), tree);
        Ok(())
    }
}
```

#### 3.3 TableData 结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub info: TableInfo,
    pub rows: Vec<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    #[serde(default)]
    pub default_value: Option<Value>,
    #[serde(default)]
    pub is_primary_key: bool,
}

impl TableData {
    pub fn new(name: String, columns: Vec<ColumnDefinition>) -> Self {
        Self {
            info: TableInfo { name, columns },
            rows: Vec::new(),
        }
    }
    
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
    
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
```

#### 3.4 B+Tree 实现（简化版）

```rust
pub struct BPlusTree<K: Ord + Clone, V: Clone> {
    root: Option<Box<Node<K, V>>>,
    order: usize,
    size: usize,
}

enum Node<K, V> {
    Internal { keys: Vec<K>, children: Vec<Box<Node<K, V>>> },
    Leaf { entries: Vec<(K, V)>, next: Option<Box<Node<K, V>>> },
}

impl<K: Ord + Clone, V: Clone> BPlusTree<K, V> {
    pub fn new() -> Self {
        Self { root: None, order: 4, size: 0 }
    }
    
    pub fn insert(&mut self, key: K, value: V) {
        match &mut self.root {
            None => {
                let mut leaf = Node::Leaf { entries: vec![(key, value)], next: None };
                self.root = Some(Box::new(leaf));
                self.size = 1;
            }
            Some(root) => {
                self.insert_recursive(root, key, value);
                self.size += 1;
            }
        }
    }
    
    pub fn search(&self, key: &K) -> Option<&V> {
        let mut current = self.root.as_ref();
        while let Some(node) = current {
            match node.as_ref() {
                Node::Leaf { entries, .. } => {
                    return entries.iter().find(|(k, _)| k == key).map(|(_, v)| v);
                }
                Node::Internal { keys, children } => {
                    let mut idx = 0;
                    while idx < keys.len() && key > &keys[idx] { idx += 1; }
                    current = children[idx].as_ref();
                }
            }
        }
        None
    }
    
    pub fn delete(&mut self, key: &K) -> bool {
        if let Some(root) = &mut self.root {
            if self.delete_recursive(root, key) {
                self.size -= 1;
                return true;
            }
        }
        false
    }
    
    pub fn range_query(&self, start: &K, end: &K) -> Vec<V> {
        let mut result = Vec::new();
        let mut current = self.root.as_ref();
        while let Some(node) = current {
            match node.as_ref() {
                Node::Leaf { entries, .. } => {
                    for (k, v) in entries {
                        if k >= start && k <= end { result.push(v.clone()); }
                    }
                    break;
                }
                Node::Internal { keys, children } => {
                    let mut idx = 0;
                    while idx < keys.len() && start > &keys[idx] { idx += 1; }
                    current = children[idx].as_ref();
                }
            }
        }
        result
    }
    
    pub fn len(&self) -> usize { self.size }
    pub fn is_empty(&self) -> bool { self.size == 0 }
}
```

#### 3.5 WAL 实现

```rust
pub struct WriteAheadLog {
    file: Mutex<File>,
    dir: PathBuf,
    current_segment: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalRecord {
    pub tx_id: u64,
    pub timestamp: i64,
    pub op_type: WalOpType,
    pub table_name: String,
    pub row_data: Option<String>,
    pub prev_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalOpType {
    Begin,
    Insert,
    Update,
    Delete,
    Commit,
    Rollback,
    Checkpoint,
}

impl WriteAheadLog {
    pub fn new(dir: &str) -> SqlResult<Self> {
        let wal_dir = PathBuf::from(dir);
        std::fs::create_dir_all(&wal_dir)?;
        let segment_path = wal_dir.join("wal-000001.log");
        let file = OpenOptions::new().create(true).append(true).open(&segment_path)?;
        Ok(Self {
            file: Mutex::new(file),
            dir: wal_dir,
            current_segment: Some(segment_path),
        })
    }
    
    pub fn append(&self, record: WalRecord) -> SqlResult<()> {
        let mut file = self.file.lock().unwrap();
        let json = serde_json::to_string(&record)?;
        writeln!(file, "{}", json)?;
        Ok(())
    }
    
    pub fn flush(&self) -> SqlResult<()> {
        let file = self.file.lock().unwrap();
        file.sync_all()?;
        Ok(())
    }
    
    pub fn recover(&self) -> SqlResult<Vec<WalRecord>> {
        let mut records = Vec::new();
        if let Some(segment) = &self.current_segment {
            if segment.exists() {
                let content = std::fs::read_to_string(segment)?;
                for line in content.lines() {
                    if let Ok(record) = serde_json::from_str::<WalRecord>(line) {
                        records.push(record);
                    }
                }
            }
        }
        Ok(records)
    }
}
```

### 4. 执行流程

```mermaid
flowchart TD
    subgraph 写入流程
        A[Executor.write] --> B{开启事务?}
        B -->|是| C[WAL.append Begin]
        B -->|否| D[直接写]
        C --> D
        D --> E{有索引?}
        E -->|是| F[更新B+Tree索引]
        E -->|否| G[直接更新rows]
        F --> G
        G --> H[WAL.append 操作日志]
        H --> I[内存修改]
        I --> J{事务?}
        J -->|是| K[WAL.append Commit + fsync]
        J -->|否| L[persist 写JSON文件]
        K --> L
        L --> M[返回Ok]
    end
    
    subgraph 读取流程
        N[Executor.scan] --> O{有匹配索引?}
        O -->|是| P[B+Tree.search/range]
        P --> Q[按row_idx定位行]
        O -->|否| R[全表迭代rows]
        Q --> S[Predicate过滤]
        R --> S
        S --> T[返回匹配行]
    end
    
    style A fill:#e8f5e9
    style N fill:#e3f2fd
    style M fill:#c8e6c9
    style T fill:#bbdefb
```

### 5. 事务处理

#### 5.1 WAL 事务流程

```mermaid
flowchart TD
    A[BEGIN tx_id=1] --> B[WAL: Begin]
    B --> C[执行 INSERT]
    C --> D[WAL: Insert row_data]
    D --> E[执行 UPDATE]
    E --> F[WAL: Update prev_state → new_state]
    F --> G[COMMIT]
    G --> H[WAL: Commit]
    H --> I[fsync WAL文件]
    I --> J[持久化数据到JSON]
    J --> K[事务完成]
    
    subgraph 崩溃恢复
        C1[重启] --> C2[扫描WAL日志]
        C2 --> C3[找到未Commit的tx=1]
        C3 --> C4[执行Rollback]
        C4 --> C5[恢复到事务前状态]
    end
    
    style A fill:#c8e6c9
    style K fill:#c8e6c9
    style C1 fill:#ffcdd2
    style C5 fill:#ffcdd2
```

#### 5.2 事务状态机

```mermaid
stateDiagram-v2
    [*] --> Idle
    
    Idle --> Active : BEGIN
    Active --> Active : 执行操作+WAL
    Active --> Committed : COMMIT+fsync+持久化
    Active --> Aborted : ROLLBACK+恢复
    Committed --> [*]
    Aborted --> [*]
    
    note right of Active: 所有操作记录WAL
    note right of Committed: fsync保证持久化
    note right of Aborted: WAL重放恢复
```

#### 5.3 TransactionManager 集成

```rust
pub struct TransactionManager {
    wal: Arc<WriteAheadLog>,
    active_txs: HashMap<u64, TransactionState>,
    next_tx_id: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct TransactionState {
    pub tx_id: u64,
    pub status: TxStatus,
    pub operations: Vec<WalOpType>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub enum TxStatus {
    Active,
    Committed,
    RolledBack,
}

impl TransactionManager {
    pub fn new(wal: Arc<WriteAheadLog>) -> Self {
        Self { wal, active_txs: HashMap::new(), next_tx_id: AtomicU64::new(0) }
    }
    
    pub fn begin(&mut self) -> SqlResult<u64> {
        let tx_id = self.next_tx_id.fetch_add(1, Ordering::SeqCst) + 1;
        self.wal.append(WalRecord {
            tx_id, timestamp: now(), op_type: WalOpType::Begin,
            table_name: String::new(), row_data: None, prev_state: None,
        })?;
        self.active_txs.insert(tx_id, TransactionState {
            tx_id, status: TxStatus::Active, operations: vec![WalOpType::Begin], created_at: now(),
        });
        Ok(tx_id)
    }
    
    pub fn commit(&mut self, tx_id: u64) -> SqlResult<()> {
        let record = WalRecord {
            tx_id, timestamp: now(), op_type: WalOpType::Commit,
            table_name: String::new(), row_data: None, prev_state: None,
        };
        self.wal.append(record)?;
        self.wal.flush()?;
        if let Some(state) = self.active_txs.get_mut(&tx_id) {
            state.status = TxStatus::Committed;
        }
        Ok(())
    }
    
    pub fn rollback(&mut self, tx_id: u64) -> SqlResult<()> {
        let record = WalRecord {
            tx_id, timestamp: now(), op_type: WalOpType::Rollback,
            table_name: String::new(), row_data: None, prev_state: None,
        };
        self.wal.append(record)?;
        if let Some(state) = self.active_txs.get_mut(&tx_id) {
            state.status = TxStatus::RolledBack;
        }
        Ok(())
    }
    
    pub fn recover(&mut self) -> SqlResult<()> {
        let records = self.wal.recover()?;
        let mut tx_ops: HashMap<u64, Vec<WalRecord>> = HashMap::new();
        for record in records {
            tx_ops.entry(record.tx_id).or_default().push(record);
        }
        for (tx_id, ops) in tx_ops {
            let last_op = ops.last();
            if let Some(record) = last_op {
                match record.op_type {
                    WalOpType::Commit => { /* 已提交，数据已持久化 */ }
                    WalOpType::Rollback => { /* 已回滚，无需操作 */ }
                    _ => { /* 未完成事务，需回滚 */
                        self.rollback(tx_id)?;
                    }
                }
            }
        }
        Ok(())
    }
}
```

### 6. 性能考虑

| 方面 | 考虑 | 1.0实现 |
|------|------|---------|
| **存储格式** | JSON格式，人类可读，便于调试 | ✅ 实现 |
| **持久化时机** | 每次写操作后立即fsync | ✅ 实现 |
| **索引类型** | B+Tree，支持范围查询 | ✅ 实现 |
| **索引类型限制** | 仅支持INTEGER键 | ⚠️ 1.0限制 |
| **缓冲池** | LRU缓存，减少磁盘I/O | ❌ 标记未来功能 |
| **页面管理** | 4KB固定页，更高效的持久化 | ❌ 标记未来功能 |
| **批量操作** | 批量INSERT/UPDATE减少fsync次数 | ⚠️ 1.0支持单次 |
| **WAL分段** | 日志分段，限制单个文件大小 | ⚠️ 1.0单文件 |
| **并发安全** | Mutex保护共享状态 | ✅ 使用Mutex |
| **崩溃恢复** | WAL重放未完成事务 | ✅ 实现 |

### 7. 1.0版本实现清单

| 序号 | 组件 | 实现内容 | 优先级 |
|------|------|----------|--------|
| 1 | FileStorage | JSON文件存储引擎 | P0 |
| 2 | TableData / TableInfo | 表结构+数据行 | P0 |
| 3 | ColumnDefinition | 列定义结构 | P0 |
| 4 | BPlusTree | B+Tree索引 | P0 |
| 5 | 索引创建/查询 | create_index + search/range | P1 |
| 6 | WriteAheadLog | WAL预写日志 | P1 |
| 7 | TransactionManager | 事务管理器 | P1 |
| 8 | WAL崩溃恢复 | recover() + 事务重放 | P1 |
| 9 | MemoryStorage | 纯内存存储 | P2 |
| 10 | BufferPool | LRU缓冲池 | P2（标记未来） |
| 11 | Page/PageFrame | 页面管理 | P2（标记未来） |
