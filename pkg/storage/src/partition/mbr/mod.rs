//! MbrTable

mod entry;

use core::marker::PhantomData;

use crate::*;
pub use entry::*;

/// The MBR Table
///
/// The disk is a collection of partitions.
/// MBR (Master Boot Record) is the *first sector* of the disk.
/// The MBR contains information about the partitions.
///
/// [ MBR | Partitions ] [ Partition 1 ] [ Partition 2 ] [ Partition 3 ] [ Partition 4 ]
pub struct MbrTable<T, B>
where
    T: BlockDevice<B> + Clone,
    B: BlockTrait,
{
    inner: T,
    partitions: [MbrPartition; 4],
    _block: PhantomData<B>,
}

impl<T, B> PartitionTable<T, B> for MbrTable<T, B>
where
    T: BlockDevice<B> + Clone,
    B: BlockTrait,
{
    fn parse(inner: T) -> FsResult<Self> {
        let mut block = B::default();
        inner.read_block(0, &mut block)?;

        let mut partitions = Vec::with_capacity(4);
        let buffer = block.as_ref();

        for i in 0..4 {
            partitions.push(
                // FIXME: parse the mbr partition from the buffer
                //      - just ignore other fields for mbr
                // 所以这里应该就是要调用entry里面的那个parse
                // 从0x1BE开始，每个分区占16字节
                MbrPartition::parse(
                    &buffer[0x1BE + i * 16..0x1BE + (i + 1) * 16]
                        .try_into()
                        .expect("Invalid partition entry size"),
                ),
            );

            if partitions[i].is_active() {
                info!("Partition {}: {:#?}", i, partitions[i]);
            }
        }

        Ok(Self {
            inner,
            partitions: partitions.try_into().unwrap(),
            _block: PhantomData,
        })
    }

    fn partitions(&self) -> FsResult<Vec<Partition<T, B>>> {
        let mut parts = Vec::new();

        for part in self.partitions {
            if part.is_active() {
                parts.push(Partition::new(
                    self.inner.clone(),
                    part.begin_lba() as usize,
                    part.total_lba() as usize,
                ));
            }
        }

        Ok(parts)
    }
}
