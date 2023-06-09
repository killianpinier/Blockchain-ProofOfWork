pub mod tx_generation;

struct Database {
    db: rocksdb::DB,
}