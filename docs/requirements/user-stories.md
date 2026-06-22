# 用户故事 

 ## US-001: 执行 SELECT 查询 
 作为用户 
 我想要执行SELECT查询语句 
 以便于从数据库中检索数据 

 验收标准： 
 - 支持基本的SELECT语句（如 `SELECT * FROM table`） 
 - 支持WHERE条件过滤（如 `SELECT * FROM table WHERE id=1`） 
 - 支持ORDER BY排序（如 `SELECT * FROM table ORDER BY name ASC`） 
 - 返回查询结果集与行数统计 

 --- 

 ## US-002: 执行 INSERT 操作 
 作为用户 
 我想要执行INSERT操作 
 以便于向数据库中添加新数据 

 验收标准： 
 - 支持单条INSERT语句（如 `INSERT INTO table (col1, col2) VALUES (val1, val2)`） 
 - 支持批量INSERT语句 
 - 插入成功后返回受影响行数 
 - 插入重复主键时返回错误提示 

 --- 

 ## US-003: 执行 UPDATE 操作 
 作为用户 
 我想要执行UPDATE操作 
 以便于修改数据库中已有数据 

 验收标准： 
 - 支持带WHERE条件的UPDATE语句（如 `UPDATE table SET col1=val WHERE id=1`） 
 - 无WHERE条件时更新全表数据 
 - 更新成功后返回受影响行数 
 - 更新不存在数据时返回0行影响 

 --- 

 ## US-004: 执行 DELETE 操作 
 作为用户 
 我想要执行DELETE操作 
 以便于从数据库中删除数据 

 验收标准： 
 - 支持带WHERE条件的DELETE语句（如 `DELETE FROM table WHERE id=1`） 
 - 无WHERE条件时删除全表数据 
 - 删除成功后返回受影响行数 
 - 删除不存在数据时返回0行影响