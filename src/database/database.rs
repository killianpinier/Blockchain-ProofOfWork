use std::error::Error;
use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, DB, Options};
use crate::block::Block;

pub struct Database {
    db: rocksdb::DB,
}

impl Database {
    pub fn open(path: &str) -> Result<Database, Box<dyn Error>> {
        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);
        Ok(Database{ db: DB::open_cf_descriptors(&db_opts, path, Database::get_cf_descriptors())? })
    }

    fn put_cf(&self, cf: &ColumnFamily, key: &[u8], value: &[u8]) -> Result<(), Box<dyn Error>> {
        self.db.put_cf(cf, key, value)?;
        Ok(())
    }

    fn get_cf(&self, cf: &ColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, Box<dyn Error>>{
        let result = self.db.get_cf(cf, key)?;
        Ok(result)
    }

    fn get_cf_descriptors() -> Vec<ColumnFamilyDescriptor> {
        vec![
            ColumnFamilyDescriptor::new("blocks", Options::default())
        ]
    }

    // --- Block management
    pub fn put_block(&self, block: &Block) -> Result<(), Box<dyn Error>> {
        match bincode::serialize(block) {
            Ok(serialized_block) => {
                if let Err(e) = self.put_cf(self.db.cf_handle("blocks").unwrap(), block.get_hash(), serialized_block.as_slice()) {
                    return Err(e);
                }
                Ok(())
            }
            Err(e) => Err(e)
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::block::Block;
    use crate::database::database::Database;

    #[test]
    fn put_block_test() {
        let db = Database::open("database-test").unwrap();
        let mut block = Block::new();
        block.set_index(3);
        block.calculate_hash();

        assert!(db.put_block(&block).is_ok());
    }
}