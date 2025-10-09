use crate::db::DBInner;
use crate::error::Result;
use crate::types::{DbItem, SetOptions};
use bytes::Bytes;
use std::collections::HashMap;

/// Operation type for atomic batches
#[derive(Debug, Clone)]
pub enum BatchOperation {
    Insert {
        key: Bytes,
        value: Bytes,
        opts: Option<SetOptions>,
    },
    Delete {
        key: Bytes,
    },
}

/// Atomic batch for executing multiple operations together
pub struct AtomicBatch {
    operations: Vec<BatchOperation>,
    operation_count: usize,
}

impl AtomicBatch {
    /// Create a new empty batch
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            operation_count: 0,
        }
    }

    /// Add an insert operation to the batch
    pub fn insert(
        &mut self,
        key: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
        opts: Option<SetOptions>,
    ) -> Result<()> {
        let key = Bytes::copy_from_slice(key.as_ref());
        let value = Bytes::copy_from_slice(value.as_ref());

        self.operations
            .push(BatchOperation::Insert { key, value, opts });
        self.operation_count += 1;

        Ok(())
    }

    /// Add a delete operation to the batch
    pub fn delete(&mut self, key: impl AsRef<[u8]>) -> Result<()> {
        let key = Bytes::copy_from_slice(key.as_ref());

        self.operations.push(BatchOperation::Delete { key });
        self.operation_count += 1;

        Ok(())
    }

    /// Get the number of operations in the batch
    pub fn len(&self) -> usize {
        self.operation_count
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.operation_count == 0
    }

    /// Clear all operations from the batch
    pub fn clear(&mut self) {
        self.operations.clear();
        self.operation_count = 0;
    }

    /// Apply all operations in the batch atomically to the database
    pub(crate) fn apply(&self, db_inner: &mut DBInner) -> Result<HashMap<Bytes, Option<Bytes>>> {
        let mut results = HashMap::new();

        // Apply all operations
        for operation in &self.operations {
            match operation {
                BatchOperation::Insert { key, value, opts } => {
                    let item = if let Some(opts) = opts {
                        if let Some(ttl) = opts.ttl {
                            DbItem::with_ttl(key.clone(), value.clone(), ttl)
                        } else if let Some(expires_at) = opts.expires_at {
                            DbItem::with_expiration(key.clone(), value.clone(), expires_at)
                        } else {
                            DbItem::new(key.clone(), value.clone())
                        }
                    } else {
                        DbItem::new(key.clone(), value.clone())
                    };

                    let old_item = db_inner.insert_item(key.clone(), item);
                    results.insert(key.clone(), old_item.map(|item| item.value));
                }
                BatchOperation::Delete { key } => {
                    let old_item = db_inner.remove_item(key);
                    results.insert(key.clone(), old_item.map(|item| item.value));
                }
            }
        }

        // Write all operations to AOF if persisting
        if let Some(ref mut aof_file) = db_inner.aof_file {
            for operation in &self.operations {
                match operation {
                    BatchOperation::Insert { key, value, opts } => {
                        // TODO: Write insert operation to AOF
                        // aof_file.write_insert(key, value, opts)?;
                    }
                    BatchOperation::Delete { key } => {
                        // TODO: Write delete operation to AOF
                        // aof_file.write_delete(key)?;
                    }
                }
            }

            // Sync based on policy
            match db_inner.config.sync_policy {
                crate::types::SyncPolicy::Always => {
                    let _ = aof_file.sync();
                }
                crate::types::SyncPolicy::EverySecond => {
                    // Background sync will handle this
                }
                crate::types::SyncPolicy::Never => {
                    // No sync
                }
            }
        }

        Ok(results)
    }
}

impl Default for AtomicBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_batch_creation() {
        let batch = AtomicBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_batch_operations() {
        let mut batch = AtomicBatch::new();

        batch.insert("key1", "value1", None).unwrap();
        batch
            .insert(
                "key2",
                "value2",
                Some(SetOptions::with_ttl(Duration::from_secs(60))),
            )
            .unwrap();
        batch.delete("key3").unwrap();

        assert_eq!(batch.len(), 3);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batch_clear() {
        let mut batch = AtomicBatch::new();

        batch.insert("key1", "value1", None).unwrap();
        batch.insert("key2", "value2", None).unwrap();

        assert_eq!(batch.len(), 2);

        batch.clear();

        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());
    }
}
