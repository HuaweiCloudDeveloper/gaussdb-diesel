mod models;
mod schema;
mod database_manager;
mod services;

use anyhow::Result;
use chrono::Utc;
use diesel::{RunQueryDsl, Connection};
use log::info;
use std::env;
use tokio::sync::oneshot;

use diesel_gaussdb::GaussDBConnection;
use database_manager::DatabaseManager;
use services::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let database_url = env::var("GAUSSDB_URL")
        .unwrap_or_else(|_| "host=localhost port=5434 user=gaussdb password=Gaussdb@123 dbname=postgres".to_string());

    info!("🚀 启动 Diesel-GaussDB 电商系统示例");
    
    let db_manager = DatabaseManager::new(database_url);

    // 初始化数据库
    initialize_database(&db_manager).await?;
    
    // 创建示例数据
    create_sample_data(&db_manager).await?;
    
    // 运行各种示例查询
    info!("📊 运行电商系统示例查询...");
    
    // 1. 基础CRUD操作
    demo_basic_operations(&db_manager).await?;
    
    // 2. 复杂关联查询
    demo_complex_joins(&db_manager).await?;
    
    // 3. 聚合和分析查询
    demo_analytics_queries(&db_manager).await?;
    
    // 4. 窗口函数和排名
    demo_window_functions(&db_manager).await?;
    
    // 5. 全文搜索和过滤
    demo_search_and_filtering(&db_manager).await?;
    
    // 6. 事务处理
    demo_transaction_processing(&db_manager).await?;
    
    // 7. 批量操作
    demo_batch_operations(&db_manager).await?;
    
    // 8. 性能优化查询
    demo_performance_queries(&db_manager).await?;

    info!("✅ 电商系统示例完成！");
    Ok(())
}

/// 初始化数据库表
async fn initialize_database(db_manager: &DatabaseManager) -> Result<()> {
    info!("初始化数据库表...");
    
    db_manager.execute_query(|conn| {
        diesel::sql_query("
            -- 创建客户表
            CREATE TABLE IF NOT EXISTS customers (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) UNIQUE NOT NULL,
                first_name VARCHAR(100) NOT NULL,
                last_name VARCHAR(100) NOT NULL,
                phone VARCHAR(20),
                date_of_birth DATE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            -- 创建分类表（支持层级结构）
            CREATE TABLE IF NOT EXISTS categories (
                id SERIAL PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                description TEXT,
                parent_id INTEGER REFERENCES categories(id),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            -- 创建供应商表
            CREATE TABLE IF NOT EXISTS suppliers (
                id SERIAL PRIMARY KEY,
                name VARCHAR(200) NOT NULL,
                contact_person VARCHAR(100) NOT NULL,
                email VARCHAR(255) NOT NULL,
                phone VARCHAR(20) NOT NULL,
                address TEXT NOT NULL,
                payment_terms VARCHAR(50) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            -- 创建产品表
            CREATE TABLE IF NOT EXISTS products (
                id SERIAL PRIMARY KEY,
                name VARCHAR(200) NOT NULL,
                description TEXT NOT NULL,
                sku VARCHAR(50) UNIQUE NOT NULL,
                price DECIMAL(10,2) NOT NULL,
                cost DECIMAL(10,2) NOT NULL,
                stock_quantity INTEGER NOT NULL DEFAULT 0,
                min_stock_level INTEGER NOT NULL DEFAULT 0,
                weight DECIMAL(8,3),
                dimensions JSONB,
                is_active BOOLEAN DEFAULT true,
                featured BOOLEAN DEFAULT false,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            -- 创建产品分类关联表
            CREATE TABLE IF NOT EXISTS product_categories (
                id SERIAL PRIMARY KEY,
                product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
                category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(product_id, category_id)
            );

            -- 创建订单表
            CREATE TABLE IF NOT EXISTS orders (
                id SERIAL PRIMARY KEY,
                customer_id INTEGER NOT NULL REFERENCES customers(id),
                status VARCHAR(50) NOT NULL DEFAULT 'pending',
                total_amount DECIMAL(12,2) NOT NULL,
                shipping_address TEXT NOT NULL,
                billing_address TEXT NOT NULL,
                payment_method VARCHAR(50) NOT NULL,
                payment_status VARCHAR(50) NOT NULL DEFAULT 'pending',
                order_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                shipped_date TIMESTAMP,
                delivered_date TIMESTAMP,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            -- 创建订单项表
            CREATE TABLE IF NOT EXISTS order_items (
                id SERIAL PRIMARY KEY,
                order_id INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
                product_id INTEGER NOT NULL REFERENCES products(id),
                quantity INTEGER NOT NULL CHECK (quantity > 0),
                unit_price DECIMAL(10,2) NOT NULL,
                total_price DECIMAL(12,2) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            -- 创建产品评论表
            CREATE TABLE IF NOT EXISTS product_reviews (
                id SERIAL PRIMARY KEY,
                product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
                customer_id INTEGER NOT NULL REFERENCES customers(id),
                rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
                title VARCHAR(200) NOT NULL,
                comment TEXT NOT NULL,
                helpful_votes INTEGER DEFAULT 0,
                verified_purchase BOOLEAN DEFAULT false,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            -- 创建供应订单表
            CREATE TABLE IF NOT EXISTS supply_orders (
                id SERIAL PRIMARY KEY,
                supplier_id INTEGER NOT NULL REFERENCES suppliers(id),
                product_id INTEGER NOT NULL REFERENCES products(id),
                quantity INTEGER NOT NULL CHECK (quantity > 0),
                unit_cost DECIMAL(10,2) NOT NULL,
                total_cost DECIMAL(12,2) NOT NULL,
                status VARCHAR(50) NOT NULL DEFAULT 'pending',
                order_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                expected_delivery DATE,
                actual_delivery DATE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
        ").execute(conn)
    }).await?;
    
    info!("✅ 数据库表初始化完成");
    Ok(())
}

/// 创建示例数据
async fn create_sample_data(db_manager: &DatabaseManager) -> Result<()> {
    info!("创建示例数据...");
    
    // 创建分类数据
    db_manager.execute_query(|conn| {
        diesel::sql_query("
            INSERT INTO categories (name, description, parent_id) VALUES
            ('Electronics', 'Electronic devices and accessories', NULL),
            ('Clothing', 'Fashion and apparel', NULL),
            ('Books', 'Books and literature', NULL),
            ('Smartphones', 'Mobile phones and accessories', 1),
            ('Laptops', 'Portable computers', 1),
            ('Men''s Clothing', 'Clothing for men', 2),
            ('Women''s Clothing', 'Clothing for women', 2),
            ('Fiction', 'Fiction books', 3),
            ('Non-Fiction', 'Non-fiction books', 3)
            ON CONFLICT DO NOTHING
        ").execute(conn)
    }).await?;
    
    // 创建客户数据
    db_manager.execute_query(|conn| {
        diesel::sql_query("
            INSERT INTO customers (email, first_name, last_name, phone, date_of_birth) VALUES
            ('john.doe@email.com', 'John', 'Doe', '+1-555-0101', '1985-03-15'),
            ('jane.smith@email.com', 'Jane', 'Smith', '+1-555-0102', '1990-07-22'),
            ('bob.johnson@email.com', 'Bob', 'Johnson', '+1-555-0103', '1988-11-08'),
            ('alice.brown@email.com', 'Alice', 'Brown', '+1-555-0104', '1992-05-30'),
            ('charlie.wilson@email.com', 'Charlie', 'Wilson', '+1-555-0105', '1987-09-12')
            ON CONFLICT (email) DO NOTHING
        ").execute(conn)
    }).await?;
    
    // 创建供应商数据
    db_manager.execute_query(|conn| {
        diesel::sql_query("
            INSERT INTO suppliers (name, contact_person, email, phone, address, payment_terms) VALUES
            ('TechCorp Inc.', 'Mike Chen', 'mike@techcorp.com', '+1-555-1001', '123 Tech Street, Silicon Valley, CA', 'Net 30'),
            ('Fashion World Ltd.', 'Sarah Johnson', 'sarah@fashionworld.com', '+1-555-1002', '456 Fashion Ave, New York, NY', 'Net 15'),
            ('BookMasters Publishing', 'David Lee', 'david@bookmasters.com', '+1-555-1003', '789 Literature Blvd, Boston, MA', 'Net 45')
            ON CONFLICT DO NOTHING
        ").execute(conn)
    }).await?;

    // 创建初始产品数据
    db_manager.execute_query(|conn| {
        diesel::sql_query("
            INSERT INTO products (name, description, sku, price, cost, stock_quantity, min_stock_level, weight, dimensions, is_active, featured) VALUES
            ('iPhone 15 Pro', 'Latest iPhone with advanced features', 'IPHONE15PRO', 999.99, 600.00, 50, 10, 0.221, '{\"length\": 159.9, \"width\": 76.7, \"height\": 8.25}', true, true),
            ('Samsung Galaxy S24', 'Premium Android smartphone', 'GALAXYS24', 899.99, 550.00, 30, 8, 0.196, '{\"length\": 158.5, \"width\": 75.9, \"height\": 8.6}', true, true),
            ('MacBook Pro 16\"', 'Professional laptop for developers', 'MBP16', 2499.99, 1800.00, 25, 5, 2.140, '{\"length\": 355.7, \"width\": 243.7, \"height\": 16.8}', true, true),
            ('Dell XPS 13', 'Ultrabook for professionals', 'DELLXPS13', 1299.99, 900.00, 20, 5, 1.200, '{\"length\": 295.7, \"width\": 198.7, \"height\": 14.8}', true, false),
            ('iPad Air', 'Powerful tablet for creativity', 'IPADAIR', 599.99, 400.00, 40, 8, 0.461, '{\"length\": 247.6, \"width\": 178.5, \"height\": 6.1}', true, false)
            ON CONFLICT (sku) DO NOTHING
        ").execute(conn)
    }).await?;

    // 将产品分配到分类
    db_manager.execute_query(|conn| {
        diesel::sql_query("
            INSERT INTO product_categories (product_id, category_id)
            SELECT p.id, c.id
            FROM products p, categories c
            WHERE (p.sku IN ('IPHONE15PRO', 'GALAXYS24') AND c.name = 'Smartphones')
               OR (p.sku IN ('MBP16', 'DELLXPS13') AND c.name = 'Laptops')
               OR (p.sku = 'IPADAIR' AND c.name = 'Electronics')
            ON CONFLICT (product_id, category_id) DO NOTHING
        ").execute(conn)
    }).await?;

    // 创建一些示例评论
    db_manager.execute_query(|conn| {
        diesel::sql_query("
            INSERT INTO product_reviews (product_id, customer_id, rating, title, comment, helpful_votes, verified_purchase)
            SELECT p.id, c.id, rating, title, comment, helpful_votes, verified_purchase
            FROM products p, customers c,
            (VALUES
                ('IPHONE15PRO', 1, 5, 'Excellent phone!', 'Amazing camera quality and performance. Highly recommended!', 15, true),
                ('IPHONE15PRO', 2, 4, 'Great but expensive', 'Love the features but the price is quite high.', 8, true),
                ('GALAXYS24', 3, 5, 'Best Android phone', 'Superior display and battery life. Perfect for work and play.', 12, true),
                ('MBP16', 4, 5, 'Perfect for development', 'Fast compilation times and excellent display for coding.', 20, true),
                ('IPADAIR', 5, 4, 'Good for creativity', 'Great for drawing and note-taking, but could use more storage.', 6, false)
            ) AS reviews(sku, customer_id, rating, title, comment, helpful_votes, verified_purchase)
            WHERE p.sku = reviews.sku AND c.id = reviews.customer_id
            ON CONFLICT DO NOTHING
        ").execute(conn)
    }).await?;

    info!("✅ 示例数据创建完成");
    Ok(())
}
