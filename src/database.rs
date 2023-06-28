use std::marker::PhantomData;
use std::rc::Rc;
use bincode::deserialize;
use crate::block::Block;
use crate::rocks::{Rocks, LedgerColumn, columns, Result, ColumnName, ColumnType};

pub enum BlockHashKeys {
    Genesis,
    LastBlock,
}

impl BlockHashKeys {
    pub fn to_bytes(&self) -> &[u8] {
        match self {
            BlockHashKeys::Genesis => b"genesis",
            BlockHashKeys::LastBlock => b"last_block"
        }
    }
}

pub struct Database {
    db: Rc<Rocks>,
    block_cf: LedgerColumn<columns::Block>,
    block_hash_cf: LedgerColumn<columns::BlockHash>,
}

impl Database {
    pub fn open(path: &str) -> Result<Database> {
        let db = Rc::new(Rocks::open(path)?);
        let block_cf = LedgerColumn::new(Rc::clone(&db));
        let block_hash_cf = LedgerColumn::new(Rc::clone(&db));

        Ok(Database {
            db,
            block_cf,
            block_hash_cf,
        })
    }

    pub fn get_block(&self, hash: &[gu8; 32]) -> Result<Option<Block>> {
        self.block_cf.get(hash)
    }

    pub fn get_last_block(&self) -> Result<Option<Block>> {
        if let Some(block_hash) = self.block_hash_cf.get(BlockHashKeys::LastBlock.to_bytes())? {
            if let Some(block) = self.block_cf.get(&block_hash)? {
                return Ok(Some(block));
            }
        }
        Ok(None)
    }

    pub fn put_block(&self, block: &Block) -> Result<()> {
        self.block_cf.put(block.get_hash(), block)
    }
}

#[cfg(test)]

mod tests {
    use crate::block::Block;
    use crate::database::{Database, LastBlockHash};

    #[test]
    fn add_meta() {
        let storage = Database::open("database-test").unwrap();
        let mut block = Block::new();
        block.calculate_hash();

        let meta1 = block.get_hash();
        storage.meta_cf.put(b"last_block", &meta1.to_vec());

        let mut meta1_from_db = storage.meta_cf.get(b"last_block").unwrap().unwrap();


        assert_eq!(meta1.to_vec(), meta1_from_db)
    }

    //#[test]
    fn add_block() {
        let storage = Database::open("database-test").unwrap();

        let mut block = Block::new();
        block.set_index(0);
        block.calculate_hash();
        storage.block_cf.put(block.get_hash(), &block).unwrap();

        let block_from_db = storage.block_cf.get(block.get_hash()).unwrap().unwrap();

        assert_eq!(block.get_hash(), block_from_db.get_hash())
    }
}
