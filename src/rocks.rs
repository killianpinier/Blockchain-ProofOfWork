use std::marker::PhantomData;
use std::rc::Rc;
use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, Options};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::block::Block;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum DatabaseError {
    RocksDb(#[from] rocksdb::Error),
    Serialize(#[from] Box<bincode::ErrorKind>)
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "database error")
    }
}


pub struct Rocks {
    db: rocksdb::DB,
}

impl Rocks {
    pub fn open(path: &str) -> Result<Rocks> {
        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);

        Ok(Rocks {
            db: rocksdb::DB::open_cf_descriptors(&db_opts, path, Rocks::get_cf_descriptors())?,
        })
    }

    pub fn cf_handle(&self, cf: &str) -> &ColumnFamily {
        self.db.cf_handle(cf).unwrap()
    }

    fn put_cf(&self, cf: &ColumnFamily, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.put_cf(cf, key, value)?;
        Ok(())
    }

    fn get_cf(&self, cf: &ColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let result = self.db.get_cf(cf, key)?;
        Ok(result)
    }
}

impl Rocks {
    fn get_cf_descriptors() -> Vec<ColumnFamilyDescriptor> {
        vec![
            ColumnFamilyDescriptor::new(columns::Block::NAME, Options::default()),
            ColumnFamilyDescriptor::new(columns::BlockHash::NAME, Options::default()),
        ]
    }
}


pub trait ColumnName {
    const NAME: &'static str;
}

pub trait ColumnType {
    type Type: Serialize + DeserializeOwned;
}


pub struct LedgerColumn<T: ColumnName + ColumnType> {
    db: Rc<Rocks>,
    column: PhantomData<T>,
}

impl<T: ColumnName + ColumnType> LedgerColumn<T> {
    pub fn new(db: Rc<Rocks>) -> LedgerColumn<T> {
        LedgerColumn{ db, column: PhantomData }
    }

    fn get_handle(&self) -> &ColumnFamily {
        self.db.cf_handle(T::NAME)
    }

    pub fn put(&self, key: &[u8], value: &T::Type) -> Result<()> {
        let serialized_value = bincode::serialize(value)?;
        self.db.put_cf(self.get_handle(), key, serialized_value.as_slice())?;
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<T::Type>> {
        if let Some(slice) = self.db.get_cf(self.get_handle(), key)? {
            let value = bincode::deserialize(slice.as_slice())?;
            return Ok(Some(value));
        }
        Ok(None)
    }
}


pub mod columns {
    pub const BLOCK_CF: &str = "block";
    pub struct Block;
    
    pub const BLOCK_HASH_CF: &str = "block_hash";
    pub struct BlockHash;
}

impl ColumnName for columns::Block {
    const NAME: &'static str = columns::BLOCK_CF;
}

impl ColumnType for columns::Block {
    type Type = Block;
}

impl ColumnName for columns::BlockHash {
    const NAME: &'static str = columns::BLOCK_HASH_CF;
}

impl ColumnType for columns::BlockHash {
    type Type = [u8; 32];
}