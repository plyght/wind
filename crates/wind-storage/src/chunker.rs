use crate::Oid;
use fastcdc::ronomon::FastCDC;

pub struct Chunk {
    pub data: Vec<u8>,
    pub oid: Oid,
    pub offset: u64,
    pub length: usize,
}

pub struct Chunker {
    min_size: usize,
    avg_size: usize,
    max_size: usize,
}

impl Default for Chunker {
    fn default() -> Self {
        Self {
            min_size: 4 * 1024,
            avg_size: 64 * 1024,
            max_size: 256 * 1024,
        }
    }
}

impl Chunker {
    pub fn new(min_size: usize, avg_size: usize, max_size: usize) -> Self {
        Self {
            min_size,
            avg_size,
            max_size,
        }
    }

    pub fn chunk_bytes(&self, data: &[u8]) -> Vec<Chunk> {
        if data.is_empty() {
            return vec![];
        }

        let chunker = FastCDC::new(data, self.min_size, self.avg_size, self.max_size);
        let mut chunks = Vec::new();
        let mut offset = 0u64;

        for entry in chunker {
            let chunk_data = data[entry.offset..entry.offset + entry.length].to_vec();
            let oid = Oid::hash_bytes(&chunk_data);

            chunks.push(Chunk {
                data: chunk_data,
                oid,
                offset,
                length: entry.length,
            });

            offset += entry.length as u64;
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_data() {
        let chunker = Chunker::default();
        let chunks = chunker.chunk_bytes(&[]);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_small_data() {
        let chunker = Chunker::default();
        let data = vec![0u8; 1024];
        let chunks = chunker.chunk_bytes(&data);
        assert!(chunks.len() >= 1);
    }

    #[test]
    fn test_large_data() {
        let chunker = Chunker::default();
        let data = vec![0u8; 1024 * 1024];
        let chunks = chunker.chunk_bytes(&data);
        assert!(chunks.len() > 1);

        let total: usize = chunks.iter().map(|c| c.length).sum();
        assert_eq!(total, data.len());
    }

    #[test]
    fn test_deduplication() {
        let chunker = Chunker::default();
        let data = vec![0u8; 100 * 1024];
        let chunks1 = chunker.chunk_bytes(&data);
        let chunks2 = chunker.chunk_bytes(&data);

        for (c1, c2) in chunks1.iter().zip(chunks2.iter()) {
            assert_eq!(c1.oid, c2.oid);
        }
    }
}
