-- diesel-gaussdb 测试数据库初始化脚本
-- 用于创建测试所需的表和数据

-- 创建测试用户表
CREATE TABLE IF NOT EXISTS test_users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    age INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建测试产品表
CREATE TABLE IF NOT EXISTS test_products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price DECIMAL(10,2) NOT NULL,
    description TEXT,
    category_id INTEGER,
    in_stock BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建测试分类表
CREATE TABLE IF NOT EXISTS test_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT
);

-- 创建测试订单表
CREATE TABLE IF NOT EXISTS test_orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES test_users(id),
    total_amount DECIMAL(10,2) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 创建测试订单项表
CREATE TABLE IF NOT EXISTS test_order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER REFERENCES test_orders(id),
    product_id INTEGER REFERENCES test_products(id),
    quantity INTEGER NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL
);

-- 插入测试数据
INSERT INTO test_categories (name, description) VALUES 
    ('Electronics', 'Electronic devices and gadgets'),
    ('Books', 'Books and publications'),
    ('Clothing', 'Apparel and accessories')
ON CONFLICT DO NOTHING;

INSERT INTO test_users (name, email, age) VALUES 
    ('Alice Johnson', 'alice@example.com', 28),
    ('Bob Smith', 'bob@example.com', 35),
    ('Carol Davis', 'carol@example.com', 42)
ON CONFLICT (email) DO NOTHING;

INSERT INTO test_products (name, price, description, category_id) VALUES 
    ('Laptop', 999.99, 'High-performance laptop', 1),
    ('Smartphone', 599.99, 'Latest smartphone model', 1),
    ('Programming Book', 49.99, 'Learn Rust programming', 2),
    ('T-Shirt', 19.99, 'Comfortable cotton t-shirt', 3)
ON CONFLICT DO NOTHING;

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_users_email ON test_users(email);
CREATE INDEX IF NOT EXISTS idx_products_category ON test_products(category_id);
CREATE INDEX IF NOT EXISTS idx_orders_user ON test_orders(user_id);
CREATE INDEX IF NOT EXISTS idx_order_items_order ON test_order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product ON test_order_items(product_id);

-- 创建用于测试复杂类型的表
CREATE TABLE IF NOT EXISTS test_complex_types (
    id SERIAL PRIMARY KEY,
    json_data JSONB,
    array_data INTEGER[],
    text_array TEXT[],
    uuid_field UUID DEFAULT gen_random_uuid(),
    timestamp_field TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    date_field DATE DEFAULT CURRENT_DATE,
    time_field TIME DEFAULT CURRENT_TIME,
    interval_field INTERVAL,
    numeric_field NUMERIC(10,2),
    boolean_field BOOLEAN DEFAULT false
);

-- 插入复杂类型测试数据
INSERT INTO test_complex_types (
    json_data, 
    array_data, 
    text_array, 
    interval_field, 
    numeric_field, 
    boolean_field
) VALUES 
    ('{"name": "test", "value": 123}', '{1,2,3,4,5}', '{"apple","banana","cherry"}', '1 day', 123.45, true),
    ('{"type": "user", "active": true}', '{10,20,30}', '{"red","green","blue"}', '2 hours', 67.89, false)
ON CONFLICT DO NOTHING;

-- 授予权限
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO gaussdb;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO gaussdb;

-- 显示创建的表
\dt

-- 显示表结构
\d test_users
\d test_products
\d test_complex_types
