// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Int4,
        name -> Varchar,
        description -> Nullable<Text>,
        parent_id -> Nullable<Int4>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    customers (id) {
        id -> Int4,
        email -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
        phone -> Nullable<Varchar>,
        date_of_birth -> Nullable<Date>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    order_items (id) {
        id -> Int4,
        order_id -> Int4,
        product_id -> Int4,
        quantity -> Int4,
        unit_price -> Numeric,
        total_price -> Numeric,
        created_at -> Timestamp,
    }
}

diesel::table! {
    orders (id) {
        id -> Int4,
        customer_id -> Int4,
        status -> Varchar,
        total_amount -> Numeric,
        shipping_address -> Text,
        billing_address -> Text,
        payment_method -> Varchar,
        payment_status -> Varchar,
        order_date -> Timestamp,
        shipped_date -> Nullable<Timestamp>,
        delivered_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    product_categories (id) {
        id -> Int4,
        product_id -> Int4,
        category_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    product_reviews (id) {
        id -> Int4,
        product_id -> Int4,
        customer_id -> Int4,
        rating -> Int4,
        title -> Varchar,
        comment -> Text,
        helpful_votes -> Int4,
        verified_purchase -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    products (id) {
        id -> Int4,
        name -> Varchar,
        description -> Text,
        sku -> Varchar,
        price -> Numeric,
        cost -> Numeric,
        stock_quantity -> Int4,
        min_stock_level -> Int4,
        weight -> Nullable<Numeric>,
        dimensions -> Nullable<Jsonb>,
        is_active -> Bool,
        featured -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    suppliers (id) {
        id -> Int4,
        name -> Varchar,
        contact_person -> Varchar,
        email -> Varchar,
        phone -> Varchar,
        address -> Text,
        payment_terms -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    supply_orders (id) {
        id -> Int4,
        supplier_id -> Int4,
        product_id -> Int4,
        quantity -> Int4,
        unit_cost -> Numeric,
        total_cost -> Numeric,
        status -> Varchar,
        order_date -> Timestamp,
        expected_delivery -> Nullable<Date>,
        actual_delivery -> Nullable<Date>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(categories -> categories (parent_id));
diesel::joinable!(order_items -> orders (order_id));
diesel::joinable!(order_items -> products (product_id));
diesel::joinable!(orders -> customers (customer_id));
diesel::joinable!(product_categories -> categories (category_id));
diesel::joinable!(product_categories -> products (product_id));
diesel::joinable!(product_reviews -> customers (customer_id));
diesel::joinable!(product_reviews -> products (product_id));
diesel::joinable!(supply_orders -> products (product_id));
diesel::joinable!(supply_orders -> suppliers (supplier_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    customers,
    order_items,
    orders,
    product_categories,
    product_reviews,
    products,
    suppliers,
    supply_orders,
);
