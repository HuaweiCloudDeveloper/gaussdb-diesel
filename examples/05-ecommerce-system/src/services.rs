use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::RunQueryDsl;
use diesel_gaussdb::expression::expression_methods::GaussDBStringExpressionMethods;
use log::info;
use serde_json::json;
use std::str::FromStr;

use crate::database_manager::DatabaseManager;
use crate::models::*;
use crate::schema::*;

/// 演示基础CRUD操作
pub async fn demo_basic_operations(db_manager: &DatabaseManager) -> Result<()> {
    info!("🔧 演示基础CRUD操作");
    
    // 创建产品
    let new_product = NewProduct {
        name: "iPhone 15 Pro".to_string(),
        description: "Latest iPhone with advanced features".to_string(),
        sku: "IPHONE15PRO".to_string(),
        price: BigDecimal::from_str("999.99")?,
        cost: BigDecimal::from_str("600.00")?,
        stock_quantity: 50,
        min_stock_level: 10,
        weight: Some(BigDecimal::from_str("0.221")?),
        dimensions: Some(json!({"length": 159.9, "width": 76.7, "height": 8.25})),
        is_active: Some(true),
        featured: Some(true),
    };
    
    let product_id = db_manager.execute_query(move |conn| {
        diesel::insert_into(products::table)
            .values(&new_product)
            .returning(products::id)
            .get_result::<i32>(conn)
    }).await?;
    
    info!("✅ 创建产品，ID: {}", product_id);
    
    // 读取产品
    let product = db_manager.execute_query(move |conn| {
        products::table
            .filter(products::id.eq(product_id))
            .select(Product::as_select())
            .first(conn)
    }).await?;
    
    info!("📖 读取产品: {}", product.name);
    
    // 更新产品价格
    let updated_price = BigDecimal::from_str("1099.99")?;
    let price_for_log = updated_price.clone(); // 克隆一份用于日志
    db_manager.execute_query(move |conn| {
        diesel::update(products::table.filter(products::id.eq(product_id)))
            .set((
                products::price.eq(&updated_price),
                products::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(conn)
    }).await?;

    info!("🔄 更新产品价格为: {}", price_for_log);
    
    // 将产品添加到分类
    db_manager.execute_query(move |conn| {
        let smartphone_category_id = categories::table
            .filter(categories::name.eq("Smartphones"))
            .select(categories::id)
            .first::<i32>(conn)?;
            
        diesel::insert_into(product_categories::table)
            .values(&NewProductCategory {
                product_id,
                category_id: smartphone_category_id,
            })
            .execute(conn)
    }).await?;
    
    info!("🏷️ 产品已添加到智能手机分类");
    
    Ok(())
}

/// 演示复杂关联查询
pub async fn demo_complex_joins(db_manager: &DatabaseManager) -> Result<()> {
    info!("🔗 演示复杂关联查询");
    
    // 查询产品及其分类信息
    let products_with_categories = db_manager.execute_query(|conn| {
        products::table
            .inner_join(product_categories::table.inner_join(categories::table))
            .select((Product::as_select(), Category::as_select()))
            .load::<(Product, Category)>(conn)
    }).await?;
    
    info!("📦 产品及分类信息:");
    for (product, category) in products_with_categories {
        info!("  - {} 属于分类: {}", product.name, category.name);
    }
    
    // 查询分类层级结构
    let category_hierarchy = db_manager.execute_query(|conn| {
        sql_query("
            WITH RECURSIVE category_tree AS (
                -- 根分类
                SELECT id, name, description, parent_id, 0 as level, 
                       CAST(name AS TEXT) as path
                FROM categories 
                WHERE parent_id IS NULL
                
                UNION ALL
                
                -- 子分类
                SELECT c.id, c.name, c.description, c.parent_id, ct.level + 1,
                       CAST(ct.path || ' > ' || c.name AS TEXT) as path
                FROM categories c
                INNER JOIN category_tree ct ON c.parent_id = ct.id
            )
            SELECT id, name, level, path FROM category_tree ORDER BY path
        ").load::<CategoryHierarchy>(conn)
    }).await?;
    
    info!("🌳 分类层级结构:");
    for cat in category_hierarchy {
        let indent = "  ".repeat(cat.level as usize);
        info!("{}├─ {} ({})", indent, cat.name, cat.path);
    }
    
    Ok(())
}

/// 演示聚合和分析查询
pub async fn demo_analytics_queries(db_manager: &DatabaseManager) -> Result<()> {
    info!("📊 演示聚合和分析查询");
    
    // 产品库存分析
    let inventory_analysis = db_manager.execute_query(|conn| {
        sql_query("
            SELECT 
                COUNT(*) as total_products,
                SUM(stock_quantity) as total_stock,
                AVG(stock_quantity) as avg_stock,
                COUNT(CASE WHEN stock_quantity <= min_stock_level THEN 1 END) as low_stock_count,
                SUM(stock_quantity * price) as total_inventory_value
            FROM products 
            WHERE is_active = true
        ").get_result::<InventoryAnalysis>(conn)
    }).await?;
    
    info!("📈 库存分析:");
    info!("  - 总产品数: {}", inventory_analysis.total_products);
    info!("  - 总库存量: {}", inventory_analysis.total_stock);
    info!("  - 平均库存: {:.2}", inventory_analysis.avg_stock);
    info!("  - 低库存产品: {}", inventory_analysis.low_stock_count);
    info!("  - 库存总价值: ${}", inventory_analysis.total_inventory_value);
    
    // 按分类统计产品
    let category_stats = db_manager.execute_query(|conn| {
        sql_query("
            SELECT 
                c.name as category_name,
                COUNT(p.id) as product_count,
                AVG(p.price) as avg_price,
                MIN(p.price) as min_price,
                MAX(p.price) as max_price,
                SUM(p.stock_quantity) as total_stock
            FROM categories c
            LEFT JOIN product_categories pc ON c.id = pc.category_id
            LEFT JOIN products p ON pc.product_id = p.id AND p.is_active = true
            WHERE c.parent_id IS NOT NULL  -- 只统计子分类
            GROUP BY c.id, c.name
            HAVING COUNT(p.id) > 0
            ORDER BY product_count DESC
        ").load::<CategoryStats>(conn)
    }).await?;
    
    info!("📊 分类统计:");
    for stat in category_stats {
        info!("  - {}: {} 个产品, 平均价格: ${:.2}", 
              stat.category_name, stat.product_count, stat.avg_price);
    }
    
    Ok(())
}

/// 演示窗口函数和排名
pub async fn demo_window_functions(db_manager: &DatabaseManager) -> Result<()> {
    info!("🏆 演示窗口函数和排名");
    
    // 产品价格排名
    let price_rankings = db_manager.execute_query(|conn| {
        sql_query("
            SELECT 
                name,
                price,
                ROW_NUMBER() OVER (ORDER BY price DESC) as price_rank,
                RANK() OVER (ORDER BY price DESC) as price_rank_with_ties,
                DENSE_RANK() OVER (ORDER BY price DESC) as dense_price_rank,
                PERCENT_RANK() OVER (ORDER BY price DESC) as price_percentile,
                price - LAG(price) OVER (ORDER BY price DESC) as price_diff_from_prev
            FROM products 
            WHERE is_active = true
            ORDER BY price DESC
            LIMIT 10
        ").load::<ProductPriceRanking>(conn)
    }).await?;
    
    info!("💰 产品价格排名:");
    for ranking in price_rankings {
        info!("  {}. {} - ${} (排名: {}, 密集排名: {})", 
              ranking.price_rank, ranking.name, ranking.price, 
              ranking.price_rank_with_ties, ranking.dense_price_rank);
    }
    
    // 分类内产品排名
    let category_rankings = db_manager.execute_query(|conn| {
        sql_query("
            SELECT 
                c.name as category_name,
                p.name as product_name,
                p.price,
                ROW_NUMBER() OVER (PARTITION BY c.id ORDER BY p.price DESC) as rank_in_category,
                FIRST_VALUE(p.name) OVER (PARTITION BY c.id ORDER BY p.price DESC) as most_expensive_in_category,
                LAST_VALUE(p.name) OVER (
                    PARTITION BY c.id 
                    ORDER BY p.price DESC 
                    ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
                ) as least_expensive_in_category
            FROM products p
            INNER JOIN product_categories pc ON p.id = pc.product_id
            INNER JOIN categories c ON pc.category_id = c.id
            WHERE p.is_active = true AND c.parent_id IS NOT NULL
            ORDER BY c.name, p.price DESC
        ").load::<CategoryProductRanking>(conn)
    }).await?;
    
    info!("🏷️ 分类内产品排名:");
    let mut current_category = String::new();
    for ranking in category_rankings {
        if ranking.category_name != current_category {
            current_category = ranking.category_name.clone();
            info!("  📂 {}", current_category);
        }
        info!("    {}. {} - ${}", 
              ranking.rank_in_category, ranking.product_name, ranking.price);
    }
    
    Ok(())
}

// 辅助结构体用于查询结果
#[derive(Debug, diesel::QueryableByName)]
struct CategoryHierarchy {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    level: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    path: String,
}

#[derive(Debug, diesel::QueryableByName)]
struct InventoryAnalysis {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total_products: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total_stock: i64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    avg_stock: f64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    low_stock_count: i64,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    total_inventory_value: BigDecimal,
}

#[derive(Debug, diesel::QueryableByName)]
struct CategoryStats {
    #[diesel(sql_type = diesel::sql_types::Text)]
    category_name: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    product_count: i64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    avg_price: f64,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    min_price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    max_price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total_stock: i64,
}

#[derive(Debug, diesel::QueryableByName)]
struct ProductPriceRanking {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    price_rank: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    price_rank_with_ties: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    dense_price_rank: i64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    price_percentile: f64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
    price_diff_from_prev: Option<BigDecimal>,
}

#[derive(Debug, diesel::QueryableByName)]
struct CategoryProductRanking {
    #[diesel(sql_type = diesel::sql_types::Text)]
    category_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    product_name: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    rank_in_category: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    most_expensive_in_category: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    least_expensive_in_category: String,
}

/// 演示搜索和过滤功能
pub async fn demo_search_and_filtering(db_manager: &DatabaseManager) -> Result<()> {
    info!("🔍 演示搜索和过滤功能");

    // 产品名称模糊搜索
    let search_results = db_manager.execute_query(|conn| {
        products::table
            .filter(GaussDBStringExpressionMethods::ilike(products::name, "%phone%"))
            .filter(products::is_active.eq(true))
            .select(Product::as_select())
            .load(conn)
    }).await?;

    info!("📱 搜索包含'phone'的产品:");
    for product in search_results {
        info!("  - {} (${}, 库存: {})", product.name, product.price, product.stock_quantity);
    }

    // 价格范围过滤
    let price_filtered = db_manager.execute_query(|conn| {
        products::table
            .filter(products::price.between(BigDecimal::from_str("100.00").unwrap(), BigDecimal::from_str("1000.00").unwrap()))
            .filter(products::is_active.eq(true))
            .order(products::price.asc())
            .select(Product::as_select())
            .load(conn)
    }).await?;

    info!("💰 价格在$100-$1000之间的产品:");
    for product in price_filtered {
        info!("  - {} - ${}", product.name, product.price);
    }

    // 库存状态过滤
    let low_stock_products = db_manager.execute_query(|conn| {
        products::table
            .filter(products::stock_quantity.le(products::min_stock_level))
            .filter(products::is_active.eq(true))
            .select(Product::as_select())
            .load(conn)
    }).await?;

    info!("⚠️ 低库存产品:");
    for product in low_stock_products {
        info!("  - {} (库存: {}, 最低库存: {})",
              product.name, product.stock_quantity, product.min_stock_level);
    }

    // 使用JSON字段查询
    let products_with_dimensions = db_manager.execute_query(|conn| {
        sql_query("
            SELECT name, dimensions,
                   (dimensions->>'length')::FLOAT as length,
                   (dimensions->>'width')::FLOAT as width,
                   (dimensions->>'height')::FLOAT as height
            FROM products
            WHERE dimensions IS NOT NULL
            AND (dimensions->>'length')::FLOAT > 150
        ").load::<ProductDimensions>(conn)
    }).await?;

    info!("📏 长度超过150mm的产品:");
    for product in products_with_dimensions {
        info!("  - {} ({}x{}x{}mm)",
              product.name, product.length, product.width, product.height);
    }

    Ok(())
}

/// 演示事务处理
pub async fn demo_transaction_processing(db_manager: &DatabaseManager) -> Result<()> {
    info!("💳 演示事务处理");

    // 模拟订单创建事务
    let order_result = db_manager.execute_query(|conn| {
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            // 1. 创建订单
            let new_order = NewOrder {
                customer_id: 1,
                status: "pending".to_string(),
                total_amount: BigDecimal::from_str("1099.99").unwrap(),
                shipping_address: "123 Main St, Anytown, USA".to_string(),
                billing_address: "123 Main St, Anytown, USA".to_string(),
                payment_method: "credit_card".to_string(),
                payment_status: "pending".to_string(),
                order_date: Utc::now().naive_utc(),
            };

            let order_id = diesel::insert_into(orders::table)
                .values(&new_order)
                .returning(orders::id)
                .get_result::<i32>(conn)?;

            // 2. 添加订单项并更新库存
            let product_id = 1; // iPhone 15 Pro
            let quantity = 2;

            // 检查库存
            let current_stock: i32 = products::table
                .filter(products::id.eq(product_id))
                .select(products::stock_quantity)
                .first(conn)?;

            if current_stock < quantity {
                return Err(diesel::result::Error::RollbackTransaction);
            }

            // 获取产品价格
            let unit_price: BigDecimal = products::table
                .filter(products::id.eq(product_id))
                .select(products::price)
                .first(conn)?;

            let total_price = &unit_price * BigDecimal::from(quantity);

            // 创建订单项
            let new_order_item = NewOrderItem {
                order_id,
                product_id,
                quantity,
                unit_price: unit_price.clone(),
                total_price: total_price.clone(),
            };

            diesel::insert_into(order_items::table)
                .values(&new_order_item)
                .execute(conn)?;

            // 更新库存
            diesel::update(products::table.filter(products::id.eq(product_id)))
                .set(products::stock_quantity.eq(current_stock - quantity))
                .execute(conn)?;

            Ok(order_id)
        })
    }).await;

    match order_result {
        Ok(order_id) => {
            info!("✅ 订单创建成功，订单ID: {}", order_id);

            // 查询订单详情
            let order_details = db_manager.execute_query(move |conn| {
                sql_query("
                    SELECT
                        o.id as order_id,
                        o.status,
                        o.total_amount,
                        c.first_name || ' ' || c.last_name as customer_name,
                        p.name as product_name,
                        oi.quantity,
                        oi.unit_price,
                        oi.total_price
                    FROM orders o
                    INNER JOIN customers c ON o.customer_id = c.id
                    INNER JOIN order_items oi ON o.id = oi.order_id
                    INNER JOIN products p ON oi.product_id = p.id
                    WHERE o.id = $1
                ").bind::<diesel::sql_types::Integer, _>(order_id)
                .load::<OrderDetails>(conn)
            }).await?;

            info!("📋 订单详情:");
            for detail in order_details {
                info!("  - 客户: {}, 产品: {}, 数量: {}, 单价: ${}, 总价: ${}",
                      detail.customer_name, detail.product_name, detail.quantity,
                      detail.unit_price, detail.total_price);
            }
        }
        Err(_) => {
            info!("❌ 订单创建失败 - 库存不足");
        }
    }

    Ok(())
}

/// 演示批量操作
pub async fn demo_batch_operations(db_manager: &DatabaseManager) -> Result<()> {
    info!("📦 演示批量操作");

    // 批量插入产品
    let batch_products = vec![
        NewProduct {
            name: "MacBook Pro 16\"".to_string(),
            description: "Professional laptop for developers".to_string(),
            sku: "MBP16".to_string(),
            price: BigDecimal::from_str("2499.99")?,
            cost: BigDecimal::from_str("1800.00")?,
            stock_quantity: 25,
            min_stock_level: 5,
            weight: Some(BigDecimal::from_str("2.140")?),
            dimensions: Some(json!({"length": 355.7, "width": 243.7, "height": 16.8})),
            is_active: Some(true),
            featured: Some(true),
        },
        NewProduct {
            name: "iPad Air".to_string(),
            description: "Powerful tablet for creativity".to_string(),
            sku: "IPADAIR".to_string(),
            price: BigDecimal::from_str("599.99")?,
            cost: BigDecimal::from_str("400.00")?,
            stock_quantity: 40,
            min_stock_level: 8,
            weight: Some(BigDecimal::from_str("0.461")?),
            dimensions: Some(json!({"length": 247.6, "width": 178.5, "height": 6.1})),
            is_active: Some(true),
            featured: Some(false),
        },
        NewProduct {
            name: "AirPods Pro".to_string(),
            description: "Wireless earbuds with noise cancellation".to_string(),
            sku: "AIRPODSPRO".to_string(),
            price: BigDecimal::from_str("249.99")?,
            cost: BigDecimal::from_str("150.00")?,
            stock_quantity: 100,
            min_stock_level: 20,
            weight: Some(BigDecimal::from_str("0.056")?),
            dimensions: Some(json!({"length": 30.9, "width": 21.8, "height": 24.0})),
            is_active: Some(true),
            featured: Some(true),
        },
    ];

    let inserted_count = db_manager.execute_query(move |conn| {
        diesel::insert_into(products::table)
            .values(&batch_products)
            .execute(conn)
    }).await?;

    info!("✅ 批量插入 {} 个产品", inserted_count);

    // 批量更新价格（打折）
    let discount_percent = BigDecimal::from_str("0.10")?; // 10% 折扣
    let updated_count = db_manager.execute_query(move |conn| {
        diesel::update(products::table.filter(products::featured.eq(true)))
            .set(products::price.eq(products::price * (BigDecimal::from(1) - &discount_percent)))
            .execute(conn)
    }).await?;

    info!("🏷️ 为 {} 个特色产品应用10%折扣", updated_count);

    // 批量库存调整
    let stock_adjustment = db_manager.execute_query(|conn| {
        sql_query("
            UPDATE products
            SET stock_quantity = stock_quantity + 10,
                updated_at = CURRENT_TIMESTAMP
            WHERE stock_quantity <= min_stock_level
        ").execute(conn)
    }).await?;

    info!("📈 为 {} 个低库存产品补充库存", stock_adjustment);

    Ok(())
}

// 更多辅助结构体
#[derive(Debug, diesel::QueryableByName)]
struct ProductDimensions {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
    dimensions: Option<serde_json::Value>,
    #[diesel(sql_type = diesel::sql_types::Float)]
    length: f32,
    #[diesel(sql_type = diesel::sql_types::Float)]
    width: f32,
    #[diesel(sql_type = diesel::sql_types::Float)]
    height: f32,
}

#[derive(Debug, diesel::QueryableByName)]
struct OrderDetails {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    order_id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    total_amount: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Text)]
    customer_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    product_name: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    quantity: i32,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    unit_price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    total_price: BigDecimal,
}

/// 演示性能优化查询
pub async fn demo_performance_queries(db_manager: &DatabaseManager) -> Result<()> {
    info!("⚡ 演示性能优化查询");

    // 使用索引优化的查询
    let indexed_search = db_manager.execute_query(|conn| {
        // 创建索引（如果不存在）
        sql_query("CREATE INDEX IF NOT EXISTS idx_products_sku ON products(sku)").execute(conn)?;
        sql_query("CREATE INDEX IF NOT EXISTS idx_products_price ON products(price)").execute(conn)?;
        sql_query("CREATE INDEX IF NOT EXISTS idx_products_active_featured ON products(is_active, featured)").execute(conn)?;

        // 使用索引的快速查询
        products::table
            .filter(products::sku.eq("IPHONE15PRO"))
            .select(Product::as_select())
            .first(conn)
    }).await?;

    info!("🔍 通过SKU快速查找产品: {}", indexed_search.name);

    // 使用EXPLAIN分析查询计划
    let query_plan = db_manager.execute_query(|conn| {
        sql_query("
            EXPLAIN (ANALYZE, BUFFERS)
            SELECT p.name, p.price, c.name as category_name
            FROM products p
            INNER JOIN product_categories pc ON p.id = pc.product_id
            INNER JOIN categories c ON pc.category_id = c.id
            WHERE p.is_active = true AND p.price > 500
            ORDER BY p.price DESC
            LIMIT 10
        ").load::<QueryPlan>(conn)
    }).await?;

    info!("📊 查询执行计划:");
    for plan in query_plan {
        info!("  {}", plan.query_plan);
    }

    // 使用物化视图概念（通过临时表模拟）
    let materialized_view_result = db_manager.execute_query(|conn| {
        // 创建产品统计的"物化视图"
        sql_query("
            CREATE TEMP TABLE IF NOT EXISTS product_stats AS
            SELECT
                p.id,
                p.name,
                p.price,
                p.stock_quantity,
                COALESCE(AVG(pr.rating), 0) as avg_rating,
                COUNT(pr.id) as review_count,
                CASE
                    WHEN p.stock_quantity <= p.min_stock_level THEN 'Low Stock'
                    WHEN p.stock_quantity <= p.min_stock_level * 2 THEN 'Medium Stock'
                    ELSE 'High Stock'
                END as stock_status
            FROM products p
            LEFT JOIN product_reviews pr ON p.id = pr.product_id
            WHERE p.is_active = true
            GROUP BY p.id, p.name, p.price, p.stock_quantity, p.min_stock_level
        ").execute(conn)?;

        // 从"物化视图"查询
        sql_query("
            SELECT name, price, avg_rating, review_count, stock_status
            FROM product_stats
            WHERE avg_rating >= 4.0 OR review_count = 0
            ORDER BY price DESC
            LIMIT 5
        ").load::<ProductStats>(conn)
    }).await?;

    info!("⭐ 高评分或新产品统计:");
    for stat in materialized_view_result {
        info!("  - {} (${}) - 评分: {:.1}, 评论数: {}, 库存状态: {}",
              stat.name, stat.price, stat.avg_rating, stat.review_count, stat.stock_status);
    }

    // 使用分区表概念（按日期分区订单）
    let partitioned_query = db_manager.execute_query(|conn| {
        sql_query("
            -- 模拟分区查询：只查询最近30天的订单
            SELECT
                DATE_TRUNC('day', order_date) as order_day,
                COUNT(*) as order_count,
                SUM(total_amount) as daily_revenue,
                AVG(total_amount) as avg_order_value
            FROM orders
            WHERE order_date >= CURRENT_DATE - INTERVAL '30 days'
            GROUP BY DATE_TRUNC('day', order_date)
            ORDER BY order_day DESC
            LIMIT 10
        ").load::<DailyOrderStats>(conn)
    }).await?;

    info!("📅 最近订单统计:");
    for stat in partitioned_query {
        info!("  - {}: {} 订单, 收入: ${}, 平均订单价值: ${}",
              stat.order_day.format("%Y-%m-%d"), stat.order_count,
              stat.daily_revenue, stat.avg_order_value);
    }

    // 使用CTE（公共表表达式）进行复杂查询
    let cte_query = db_manager.execute_query(|conn| {
        sql_query("
            WITH product_performance AS (
                SELECT
                    p.id,
                    p.name,
                    p.price,
                    p.cost,
                    (p.price - p.cost) as profit_per_unit,
                    ((p.price - p.cost) / p.cost * 100) as profit_margin_percent,
                    p.stock_quantity,
                    COALESCE(SUM(oi.quantity), 0) as total_sold
                FROM products p
                LEFT JOIN order_items oi ON p.id = oi.product_id
                WHERE p.is_active = true
                GROUP BY p.id, p.name, p.price, p.cost, p.stock_quantity
            ),
            performance_ranking AS (
                SELECT *,
                    RANK() OVER (ORDER BY profit_margin_percent DESC) as profit_rank,
                    RANK() OVER (ORDER BY total_sold DESC) as sales_rank,
                    CASE
                        WHEN total_sold > 10 THEN 'High Performer'
                        WHEN total_sold > 5 THEN 'Medium Performer'
                        WHEN total_sold > 0 THEN 'Low Performer'
                        ELSE 'No Sales'
                    END as performance_category
                FROM product_performance
            )
            SELECT
                name,
                price,
                profit_per_unit,
                profit_margin_percent,
                total_sold,
                profit_rank,
                sales_rank,
                performance_category
            FROM performance_ranking
            WHERE profit_margin_percent > 20
            ORDER BY profit_margin_percent DESC
            LIMIT 10
        ").load::<ProductPerformance>(conn)
    }).await?;

    info!("🎯 产品性能分析 (利润率>20%):");
    for perf in cte_query {
        info!("  - {} - 利润率: {:.1}%, 销量: {}, 利润排名: {}, 销量排名: {}, 类别: {}",
              perf.name, perf.profit_margin_percent, perf.total_sold,
              perf.profit_rank, perf.sales_rank, perf.performance_category);
    }

    Ok(())
}

// 更多辅助结构体
#[derive(Debug, diesel::QueryableByName)]
struct QueryPlan {
    #[diesel(sql_type = diesel::sql_types::Text)]
    query_plan: String,
}

#[derive(Debug, diesel::QueryableByName)]
struct ProductStats {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Double)]
    avg_rating: f64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    review_count: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    stock_status: String,
}

#[derive(Debug, diesel::QueryableByName)]
struct DailyOrderStats {
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    order_day: NaiveDateTime,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    order_count: i64,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    daily_revenue: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    avg_order_value: BigDecimal,
}

#[derive(Debug, diesel::QueryableByName)]
struct ProductPerformance {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    profit_per_unit: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Double)]
    profit_margin_percent: f64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total_sold: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    profit_rank: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    sales_rank: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    performance_category: String,
}
