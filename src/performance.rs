//! Performance optimization utilities for diesel-gaussdb
//!
//! This module provides performance optimization features including
//! query caching, connection pooling optimizations, and batch operations.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Query cache for frequently executed queries
#[derive(Debug)]
pub struct QueryCache {
    cache: Arc<Mutex<HashMap<String, CachedQuery>>>,
    max_size: usize,
    ttl: Duration,
}

/// Cached query information
#[derive(Debug, Clone)]
struct CachedQuery {
    sql: String,
    created_at: Instant,
    hit_count: u64,
    last_accessed: Instant,
}

impl QueryCache {
    /// Create a new query cache
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_size,
            ttl,
        }
    }
    
    /// Get a cached query
    pub fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();
        
        if let Some(cached) = cache.get_mut(key) {
            // Check if cache entry is still valid
            if cached.created_at.elapsed() < self.ttl {
                cached.hit_count += 1;
                cached.last_accessed = Instant::now();
                return Some(cached.sql.clone());
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }
        
        None
    }
    
    /// Put a query in the cache
    pub fn put(&self, key: String, sql: String) {
        let mut cache = self.cache.lock().unwrap();
        
        // Remove expired entries
        self.cleanup_expired(&mut cache);
        
        // If cache is full, remove least recently used entry
        if cache.len() >= self.max_size {
            self.evict_lru(&mut cache);
        }
        
        let cached_query = CachedQuery {
            sql,
            created_at: Instant::now(),
            hit_count: 0,
            last_accessed: Instant::now(),
        };
        
        cache.insert(key, cached_query);
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        let total_hits: u64 = cache.values().map(|q| q.hit_count).sum();
        
        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
            total_hits,
            hit_rate: if cache.len() > 0 { 
                total_hits as f64 / cache.len() as f64 
            } else { 
                0.0 
            },
        }
    }
    
    /// Clean up expired entries
    fn cleanup_expired(&self, cache: &mut HashMap<String, CachedQuery>) {
        let now = Instant::now();
        cache.retain(|_, query| now.duration_since(query.created_at) < self.ttl);
    }
    
    /// Evict least recently used entry
    fn evict_lru(&self, cache: &mut HashMap<String, CachedQuery>) {
        if let Some((lru_key, _)) = cache
            .iter()
            .min_by_key(|(_, query)| query.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            cache.remove(&lru_key);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current cache size
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Total cache hits
    pub total_hits: u64,
    /// Cache hit rate
    pub hit_rate: f64,
}

/// Batch operation builder for improved performance
#[derive(Debug)]
pub struct BatchBuilder {
    operations: Vec<BatchOperation>,
    max_batch_size: usize,
}

/// Individual batch operation
#[derive(Debug, Clone)]
pub enum BatchOperation {
    Insert { table: String, values: Vec<String> },
    Update { table: String, set_clause: String, where_clause: String },
    Delete { table: String, where_clause: String },
}

impl BatchBuilder {
    /// Create a new batch builder
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            operations: Vec::new(),
            max_batch_size,
        }
    }
    
    /// Add an insert operation
    pub fn insert(mut self, table: impl Into<String>, values: Vec<String>) -> Self {
        self.operations.push(BatchOperation::Insert {
            table: table.into(),
            values,
        });
        self
    }
    
    /// Add an update operation
    pub fn update(
        mut self, 
        table: impl Into<String>, 
        set_clause: impl Into<String>,
        where_clause: impl Into<String>
    ) -> Self {
        self.operations.push(BatchOperation::Update {
            table: table.into(),
            set_clause: set_clause.into(),
            where_clause: where_clause.into(),
        });
        self
    }
    
    /// Add a delete operation
    pub fn delete(
        mut self, 
        table: impl Into<String>, 
        where_clause: impl Into<String>
    ) -> Self {
        self.operations.push(BatchOperation::Delete {
            table: table.into(),
            where_clause: where_clause.into(),
        });
        self
    }
    
    /// Build the batch SQL statements
    pub fn build(self) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current_batch = Vec::new();

        for operation in self.operations {
            current_batch.push(operation);

            if current_batch.len() >= self.max_batch_size {
                statements.push(Self::build_batch_sql_static(&current_batch));
                current_batch.clear();
            }
        }

        // Handle remaining operations
        if !current_batch.is_empty() {
            statements.push(Self::build_batch_sql_static(&current_batch));
        }

        statements
    }
    
    /// Build SQL for a batch of operations
    fn build_batch_sql_static(batch: &[BatchOperation]) -> String {
        let mut sql = String::new();
        
        for (i, operation) in batch.iter().enumerate() {
            if i > 0 {
                sql.push_str(";\n");
            }
            
            match operation {
                BatchOperation::Insert { table, values } => {
                    sql.push_str(&format!(
                        "INSERT INTO {} VALUES {}",
                        table,
                        values.join(", ")
                    ));
                }
                BatchOperation::Update { table, set_clause, where_clause } => {
                    sql.push_str(&format!(
                        "UPDATE {} SET {} WHERE {}",
                        table, set_clause, where_clause
                    ));
                }
                BatchOperation::Delete { table, where_clause } => {
                    sql.push_str(&format!(
                        "DELETE FROM {} WHERE {}",
                        table, where_clause
                    ));
                }
            }
        }
        
        sql
    }
}

/// Connection pool optimization settings
#[derive(Debug, Clone)]
pub struct PoolOptimization {
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Maximum number of connections allowed
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: Duration,
    /// Idle timeout for connections
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
}

impl Default for PoolOptimization {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Duration::from_secs(1800), // 30 minutes
        }
    }
}

impl PoolOptimization {
    /// Create optimized settings for high-throughput scenarios
    pub fn high_throughput() -> Self {
        Self {
            min_connections: 5,
            max_connections: 50,
            connection_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300), // 5 minutes
            max_lifetime: Duration::from_secs(3600), // 1 hour
        }
    }
    
    /// Create optimized settings for low-latency scenarios
    pub fn low_latency() -> Self {
        Self {
            min_connections: 10,
            max_connections: 20,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(120), // 2 minutes
            max_lifetime: Duration::from_secs(900), // 15 minutes
        }
    }
    
    /// Create optimized settings for resource-constrained environments
    pub fn resource_constrained() -> Self {
        Self {
            min_connections: 1,
            max_connections: 5,
            connection_timeout: Duration::from_secs(60),
            idle_timeout: Duration::from_secs(1200), // 20 minutes
            max_lifetime: Duration::from_secs(7200), // 2 hours
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_query_cache() {
        let cache = QueryCache::new(2, Duration::from_secs(1));
        
        // Test cache miss
        assert!(cache.get("key1").is_none());
        
        // Test cache put and hit
        cache.put("key1".to_string(), "SELECT 1".to_string());
        assert_eq!(cache.get("key1"), Some("SELECT 1".to_string()));
        
        // Test cache stats
        let stats = cache.stats();
        assert_eq!(stats.size, 1);
        assert_eq!(stats.total_hits, 1);
    }
    
    #[test]
    fn test_cache_expiration() {
        let cache = QueryCache::new(10, Duration::from_millis(50));
        
        cache.put("key1".to_string(), "SELECT 1".to_string());
        assert!(cache.get("key1").is_some());
        
        // Wait for expiration
        thread::sleep(Duration::from_millis(100));
        assert!(cache.get("key1").is_none());
    }
    
    #[test]
    fn test_batch_builder() {
        let batch = BatchBuilder::new(10)
            .insert("users", vec!["(1, 'Alice')".to_string(), "(2, 'Bob')".to_string()])
            .update("users", "name = 'Charlie'", "id = 1")
            .delete("users", "id = 2")
            .build();
        
        assert_eq!(batch.len(), 1);
        assert!(batch[0].contains("INSERT INTO users"));
        assert!(batch[0].contains("UPDATE users"));
        assert!(batch[0].contains("DELETE FROM users"));
    }
    
    #[test]
    fn test_pool_optimization_presets() {
        let high_throughput = PoolOptimization::high_throughput();
        assert_eq!(high_throughput.max_connections, 50);
        
        let low_latency = PoolOptimization::low_latency();
        assert_eq!(low_latency.min_connections, 10);
        
        let resource_constrained = PoolOptimization::resource_constrained();
        assert_eq!(resource_constrained.max_connections, 5);
    }
}
