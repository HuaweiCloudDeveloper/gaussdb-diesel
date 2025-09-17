use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use bigdecimal::BigDecimal;

use crate::schema::*;

// Customer Models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = customers)]
pub struct Customer {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = customers)]
pub struct NewCustomer {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
}

// Product Models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = products)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub sku: String,
    pub price: BigDecimal,
    pub cost: BigDecimal,
    pub stock_quantity: i32,
    pub min_stock_level: i32,
    pub weight: Option<BigDecimal>,
    pub dimensions: Option<JsonValue>,
    pub is_active: bool,
    pub featured: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub name: String,
    pub description: String,
    pub sku: String,
    pub price: BigDecimal,
    pub cost: BigDecimal,
    pub stock_quantity: i32,
    pub min_stock_level: i32,
    pub weight: Option<BigDecimal>,
    pub dimensions: Option<JsonValue>,
    pub is_active: Option<bool>,
    pub featured: Option<bool>,
}

// Category Models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = categories)]
pub struct NewCategory {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
}

// Order Models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: i32,
    pub customer_id: i32,
    pub status: String,
    pub total_amount: BigDecimal,
    pub shipping_address: String,
    pub billing_address: String,
    pub payment_method: String,
    pub payment_status: String,
    pub order_date: NaiveDateTime,
    pub shipped_date: Option<NaiveDateTime>,
    pub delivered_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub customer_id: i32,
    pub status: String,
    pub total_amount: BigDecimal,
    pub shipping_address: String,
    pub billing_address: String,
    pub payment_method: String,
    pub payment_status: String,
    pub order_date: NaiveDateTime,
}

// Order Item Models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = order_items)]
pub struct OrderItem {
    pub id: i32,
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: BigDecimal,
    pub total_price: BigDecimal,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = order_items)]
pub struct NewOrderItem {
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: BigDecimal,
    pub total_price: BigDecimal,
}

// Product Review Models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = product_reviews)]
pub struct ProductReview {
    pub id: i32,
    pub product_id: i32,
    pub customer_id: i32,
    pub rating: i32,
    pub title: String,
    pub comment: String,
    pub helpful_votes: i32,
    pub verified_purchase: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = product_reviews)]
pub struct NewProductReview {
    pub product_id: i32,
    pub customer_id: i32,
    pub rating: i32,
    pub title: String,
    pub comment: String,
    pub verified_purchase: bool,
}

// Supplier Models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = suppliers)]
pub struct Supplier {
    pub id: i32,
    pub name: String,
    pub contact_person: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub payment_terms: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = suppliers)]
pub struct NewSupplier {
    pub name: String,
    pub contact_person: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub payment_terms: String,
}

// Supply Order Models
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = supply_orders)]
pub struct SupplyOrder {
    pub id: i32,
    pub supplier_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_cost: BigDecimal,
    pub total_cost: BigDecimal,
    pub status: String,
    pub order_date: NaiveDateTime,
    pub expected_delivery: Option<NaiveDate>,
    pub actual_delivery: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = supply_orders)]
pub struct NewSupplyOrder {
    pub supplier_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_cost: BigDecimal,
    pub total_cost: BigDecimal,
    pub status: String,
    pub order_date: NaiveDateTime,
    pub expected_delivery: Option<NaiveDate>,
}

// Product Category Junction
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = product_categories)]
pub struct ProductCategory {
    pub id: i32,
    pub product_id: i32,
    pub category_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = product_categories)]
pub struct NewProductCategory {
    pub product_id: i32,
    pub category_id: i32,
}

// Complex Query Result Types
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductWithCategory {
    pub product: Product,
    pub categories: Vec<Category>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderWithItems {
    pub order: Order,
    pub customer: Customer,
    pub items: Vec<OrderItemWithProduct>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemWithProduct {
    pub order_item: OrderItem,
    pub product: Product,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductAnalytics {
    pub product_id: i32,
    pub product_name: String,
    pub total_sold: i64,
    pub total_revenue: BigDecimal,
    pub avg_rating: Option<f64>,
    pub review_count: i64,
    pub current_stock: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerAnalytics {
    pub customer_id: i32,
    pub customer_name: String,
    pub total_orders: i64,
    pub total_spent: BigDecimal,
    pub avg_order_value: BigDecimal,
    pub last_order_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesReport {
    pub period: String,
    pub total_orders: i64,
    pub total_revenue: BigDecimal,
    pub avg_order_value: BigDecimal,
    pub top_products: Vec<ProductAnalytics>,
}
