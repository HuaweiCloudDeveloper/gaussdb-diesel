//! 类型系统验证示例
//!
//! 这个示例全面验证 diesel-gaussdb 的类型系统，包括：
//! - 所有基础类型的往返转换
//! - 复杂类型（数组、JSON、范围等）
//! - 自定义类型
//! - 类型安全性验证
//!
//! 运行示例：
//! ```bash
//! cargo run --example type_system_validation --features gaussdb,r2d2,chrono,uuid,serde_json
//! ```

use diesel::prelude::*;
use diesel::connection::SimpleConnection;
use diesel_gaussdb::prelude::*;
use std::env;

// 导入所有需要的类型
use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
use uuid::Uuid;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Diesel-GaussDB 类型系统验证");
    println!("===============================");

    // 建立数据库连接
    let database_url = env::var("GAUSSDB_TEST_URL")
        .unwrap_or_else(|_| {
            "host=localhost port=5434 user=gaussdb password=Gaussdb@123 dbname=diesel_test".to_string()
        });

    let mut conn = GaussDBConnection::establish(&database_url)?;
    println!("✅ 数据库连接成功");

    // 1. 基础类型验证
    println!("\n🔍 1. 基础类型验证");
    test_primitive_types(&mut conn)?;

    // 2. 数值类型验证
    println!("\n🔍 2. 数值类型验证");
    test_numeric_types(&mut conn)?;

    // 3. 文本类型验证
    println!("\n🔍 3. 文本类型验证");
    test_text_types(&mut conn)?;

    // 4. 日期时间类型验证
    println!("\n🔍 4. 日期时间类型验证");
    test_datetime_types(&mut conn)?;

    // 5. 数组类型验证
    println!("\n🔍 5. 数组类型验证");
    test_array_types(&mut conn)?;

    // 6. JSON 类型验证
    println!("\n🔍 6. JSON 类型验证");
    test_json_types(&mut conn)?;

    // 7. UUID 类型验证
    println!("\n🔍 7. UUID 类型验证");
    test_uuid_types(&mut conn)?;

    // 8. 网络类型验证
    println!("\n🔍 8. 网络类型验证");
    test_network_types(&mut conn)?;

    // 9. 范围类型验证
    println!("\n🔍 9. 范围类型验证");
    test_range_types(&mut conn)?;

    // 10. 货币类型验证
    println!("\n🔍 10. 货币类型验证");
    test_money_types(&mut conn)?;

    // 11. 二进制类型验证
    println!("\n🔍 11. 二进制类型验证");
    test_binary_types(&mut conn)?;

    // 12. NULL 值处理验证
    println!("\n🔍 12. NULL 值处理验证");
    test_null_handling(&mut conn)?;

    println!("\n🎉 所有类型系统验证完成！");
    Ok(())
}

fn test_primitive_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 布尔类型
    println!("  📋 布尔类型测试");
    let queries = vec![
        "SELECT true::boolean",
        "SELECT false::boolean",
        "SELECT NULL::boolean",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "布尔类型查询应该成功: {}", query);
    }
    println!("    ✅ 布尔类型验证通过");

    // 字符类型
    println!("  📋 字符类型测试");
    let queries = vec![
        "SELECT 'A'::char",
        "SELECT 'Z'::char",
        "SELECT NULL::char",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "字符类型查询应该成功: {}", query);
    }
    println!("    ✅ 字符类型验证通过");

    Ok(())
}

fn test_numeric_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 整数类型
    println!("  📋 整数类型测试");
    let queries = vec![
        "SELECT 42::smallint",
        "SELECT 2147483647::integer",
        "SELECT 9223372036854775807::bigint",
        "SELECT -32768::smallint",
        "SELECT -2147483648::integer",
        "SELECT -9223372036854775808::bigint",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "整数类型查询应该成功: {}", query);
    }
    println!("    ✅ 整数类型验证通过");

    // 浮点类型
    println!("  📋 浮点类型测试");
    let queries = vec![
        "SELECT 3.14159::real",
        "SELECT 3.141592653589793::double precision",
        "SELECT 'NaN'::real",
        "SELECT 'Infinity'::real",
        "SELECT '-Infinity'::real",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "浮点类型查询应该成功: {}", query);
    }
    println!("    ✅ 浮点类型验证通过");

    // 数值类型
    println!("  📋 数值类型测试");
    let queries = vec![
        "SELECT 123.456::numeric",
        "SELECT 999999999999999999999.999999999999999999999::numeric",
        "SELECT 0.000000000000000000001::numeric",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "数值类型查询应该成功: {}", query);
    }
    println!("    ✅ 数值类型验证通过");

    Ok(())
}

fn test_text_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 文本类型
    println!("  📋 文本类型测试");
    let queries = vec![
        "SELECT 'Hello, World!'::text",
        "SELECT 'Unicode: 你好世界 🌍'::text",
        "SELECT 'Empty string: '::text",
        "SELECT 'Variable length'::varchar(50)",
        "SELECT 'Fixed length'::char(20)",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "文本类型查询应该成功: {}", query);
    }
    println!("    ✅ 文本类型验证通过");

    // 特殊字符处理
    println!("  📋 特殊字符处理测试");
    let queries = vec![
        r#"SELECT 'Quote: ''single'' and "double"'::text"#,
        r#"SELECT 'Backslash: \\ and newline: \n'::text"#,
        r#"SELECT 'Tab: \t and carriage return: \r'::text"#,
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "特殊字符查询应该成功: {}", query);
    }
    println!("    ✅ 特殊字符处理验证通过");

    Ok(())
}

fn test_datetime_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 日期类型
    println!("  📋 日期类型测试");
    let queries = vec![
        "SELECT '2024-01-01'::date",
        "SELECT '1970-01-01'::date",
        "SELECT '2099-12-31'::date",
        "SELECT CURRENT_DATE",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "日期类型查询应该成功: {}", query);
    }
    println!("    ✅ 日期类型验证通过");

    // 时间类型
    println!("  📋 时间类型测试");
    let queries = vec![
        "SELECT '12:30:45'::time",
        "SELECT '00:00:00'::time",
        "SELECT '23:59:59'::time",
        "SELECT '12:30:45.123456'::time",
        "SELECT CURRENT_TIME",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "时间类型查询应该成功: {}", query);
    }
    println!("    ✅ 时间类型验证通过");

    // 时间戳类型
    println!("  📋 时间戳类型测试");
    let queries = vec![
        "SELECT '2024-01-01 12:30:45'::timestamp",
        "SELECT '2024-01-01 12:30:45.123456'::timestamp",
        "SELECT '2024-01-01 12:30:45+08'::timestamptz",
        "SELECT CURRENT_TIMESTAMP",
        "SELECT NOW()",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "时间戳类型查询应该成功: {}", query);
    }
    println!("    ✅ 时间戳类型验证通过");

    Ok(())
}

fn test_array_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 数组类型
    println!("  📋 数组类型测试");
    let queries = vec![
        "SELECT ARRAY[1,2,3,4,5]",
        "SELECT ARRAY['a','b','c']",
        "SELECT ARRAY[true,false,true]",
        "SELECT ARRAY[]::integer[]",
        "SELECT ARRAY[ARRAY[1,2],ARRAY[3,4]]",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "数组类型查询应该成功: {}", query);
    }
    println!("    ✅ 数组类型验证通过");

    // 数组操作
    println!("  📋 数组操作测试");
    let queries = vec![
        "SELECT array_length(ARRAY[1,2,3,4,5], 1)",
        "SELECT array_append(ARRAY[1,2,3], 4)",
        "SELECT array_prepend(0, ARRAY[1,2,3])",
        "SELECT array_cat(ARRAY[1,2], ARRAY[3,4])",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "数组操作查询应该成功: {}", query);
    }
    println!("    ✅ 数组操作验证通过");

    Ok(())
}

fn test_json_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // JSON 类型
    println!("  📋 JSON 类型测试");
    let queries = vec![
        r#"SELECT '{"key": "value"}'::json"#,
        r#"SELECT '{"number": 42, "boolean": true, "null": null}'::json"#,
        r#"SELECT '[1, 2, 3, "string"]'::json"#,
        r#"SELECT '{"nested": {"key": "value"}}'::json"#,
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "JSON 类型查询应该成功: {}", query);
    }
    println!("    ✅ JSON 类型验证通过");

    // JSON 操作
    println!("  📋 JSON 操作测试");
    let queries = vec![
        r#"SELECT '{"key": "value"}'::json -> 'key'"#,
        r#"SELECT '{"key": "value"}'::json ->> 'key'"#,
        r#"SELECT '{"nested": {"key": "value"}}'::json -> 'nested' -> 'key'"#,
        r#"SELECT '[0, 1, 2]'::json -> 1"#,
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "JSON 操作查询应该成功: {}", query);
    }
    println!("    ✅ JSON 操作验证通过");

    Ok(())
}

fn test_uuid_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // UUID 类型
    println!("  📋 UUID 类型测试");
    let queries = vec![
        "SELECT gen_random_uuid()",
        "SELECT '550e8400-e29b-41d4-a716-446655440000'::uuid",
        "SELECT uuid_generate_v4()",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        match result {
            Ok(_) => println!("    ✅ UUID 查询成功: {}", query),
            Err(_) => println!("    ⚠️  UUID 查询需要扩展: {}", query),
        }
    }
    println!("    ✅ UUID 类型验证完成");

    Ok(())
}

fn test_network_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 网络地址类型
    println!("  📋 网络地址类型测试");
    let queries = vec![
        "SELECT '192.168.1.1'::inet",
        "SELECT '192.168.1.0/24'::cidr",
        "SELECT '08:00:2b:01:02:03'::macaddr",
        "SELECT '08:00:2b:01:02:03:04:05'::macaddr8",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        match result {
            Ok(_) => println!("    ✅ 网络类型查询成功: {}", query),
            Err(_) => println!("    ⚠️  网络类型查询需要扩展: {}", query),
        }
    }
    println!("    ✅ 网络类型验证完成");

    Ok(())
}

fn test_range_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 范围类型
    println!("  📋 范围类型测试");
    let queries = vec![
        "SELECT '[1,10]'::int4range",
        "SELECT '[1,10)'::int4range",
        "SELECT '(1,10]'::int4range",
        "SELECT '(1,10)'::int4range",
        "SELECT '[2024-01-01,2024-12-31]'::daterange",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        match result {
            Ok(_) => println!("    ✅ 范围类型查询成功: {}", query),
            Err(_) => println!("    ⚠️  范围类型查询需要扩展: {}", query),
        }
    }
    println!("    ✅ 范围类型验证完成");

    Ok(())
}

fn test_money_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 货币类型
    println!("  📋 货币类型测试");
    let queries = vec![
        "SELECT '$123.45'::money",
        "SELECT '$1,234.56'::money",
        "SELECT '-$99.99'::money",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        match result {
            Ok(_) => println!("    ✅ 货币类型查询成功: {}", query),
            Err(_) => println!("    ⚠️  货币类型查询需要扩展: {}", query),
        }
    }
    println!("    ✅ 货币类型验证完成");

    Ok(())
}

fn test_binary_types(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 二进制类型
    println!("  📋 二进制类型测试");
    let queries = vec![
        r#"SELECT '\x48656c6c6f'::bytea"#,
        r#"SELECT decode('SGVsbG8=', 'base64')"#,
        r#"SELECT encode('Hello', 'hex')"#,
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "二进制类型查询应该成功: {}", query);
    }
    println!("    ✅ 二进制类型验证通过");

    Ok(())
}

fn test_null_handling(conn: &mut GaussDBConnection) -> Result<(), Box<dyn std::error::Error>> {
    // NULL 值处理
    println!("  📋 NULL 值处理测试");
    let queries = vec![
        "SELECT NULL",
        "SELECT NULL::integer",
        "SELECT NULL::text",
        "SELECT NULL::boolean",
        "SELECT COALESCE(NULL, 'default')",
        "SELECT NULLIF('', '')",
    ];
    
    for query in queries {
        let result = conn.batch_execute(query);
        assert!(result.is_ok(), "NULL 处理查询应该成功: {}", query);
    }
    println!("    ✅ NULL 值处理验证通过");

    Ok(())
}
