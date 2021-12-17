use crate::common::{
    get_data_dir, vsdb_set_base_dir, BranchID, Engine, Prefix, PrefixBytes, VersionID,
    PREFIX_SIZ, RESERVED_ID_CNT,
};
use ruc::*;
use sled::{Config, Db, Iter, Mode, Tree};
use std::ops::{Bound, RangeBounds};

const DATA_SET_NUM: u8 = 8;

pub(crate) struct SledEngine {
    meta: Db,
    areas: Vec<Tree>,
    prefix_allocator: PrefixAllocator,
}

impl Engine for SledEngine {
    fn new() -> Result<Self> {
        let meta = sled_open().c(d!())?;

        let areas = (0..DATA_SET_NUM)
            .map(|idx| meta.open_tree(idx.to_be_bytes()).c(d!()))
            .collect::<Result<Vec<_>>>()?;

        let (prefix_allocator, initial_value) = PrefixAllocator::init();

        if meta.get(prefix_allocator.key).c(d!())?.is_none() {
            meta.insert(prefix_allocator.key, initial_value).c(d!())?;
        }

        Ok(SledEngine {
            meta,
            areas,
            prefix_allocator,
        })
    }

    fn alloc_prefix(&self) -> Prefix {
        crate::parse_prefix!(
            self.meta
                .update_and_fetch(self.prefix_allocator.key, PrefixAllocator::next)
                .unwrap()
                .unwrap()
                .as_ref()
        )
    }

    fn alloc_branch_id(&self) -> BranchID {
        self.alloc_prefix() as BranchID
    }

    fn alloc_version_id(&self) -> VersionID {
        self.alloc_prefix() as VersionID
    }

    fn area_count(&self) -> u8 {
        self.areas.len() as u8
    }

    fn flush(&self) {
        (0..self.areas.len()).for_each(|i| {
            self.areas[i].flush().unwrap();
        });
    }

    fn iter(&self, area_idx: usize, meta_prefix: PrefixBytes) -> SledIter {
        SledIter {
            inner: self.areas[area_idx].scan_prefix(meta_prefix.as_slice()),
        }
    }

    fn range<'a, R: RangeBounds<&'a [u8]>>(
        &'a self,
        area_idx: usize,
        meta_prefix: PrefixBytes,
        bounds: R,
    ) -> SledIter {
        let mut b_lo = meta_prefix.to_vec();
        let l = match bounds.start_bound() {
            Bound::Included(lo) => {
                b_lo.extend_from_slice(lo);
                Bound::Included(b_lo)
            }
            Bound::Excluded(lo) => {
                b_lo.extend_from_slice(lo);
                Bound::Excluded(b_lo)
            }
            Bound::Unbounded => Bound::Unbounded,
        };

        let mut b_hi = meta_prefix.to_vec();
        let h = match bounds.end_bound() {
            Bound::Included(hi) => {
                b_hi.extend_from_slice(hi);
                Bound::Included(b_hi)
            }
            Bound::Excluded(hi) => {
                b_hi.extend_from_slice(hi);
                Bound::Excluded(b_hi)
            }
            Bound::Unbounded => Bound::Unbounded,
        };

        SledIter {
            inner: self.areas[area_idx].range((l, h)),
        }
    }

    fn get(
        &self,
        area_idx: usize,
        meta_prefix: PrefixBytes,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        let mut k = meta_prefix.to_vec();
        k.extend_from_slice(key);
        self.areas[area_idx].get(k).unwrap().map(|iv| iv.to_vec())
    }

    fn insert(
        &self,
        area_idx: usize,
        meta_prefix: PrefixBytes,
        key: &[u8],
        value: &[u8],
    ) -> Option<Vec<u8>> {
        let mut k = meta_prefix.to_vec();
        k.extend_from_slice(key);
        self.areas[area_idx]
            .insert(k, value)
            .unwrap()
            .map(|iv| iv.to_vec())
    }

    fn remove(
        &self,
        area_idx: usize,
        meta_prefix: PrefixBytes,
        key: &[u8],
    ) -> Option<Vec<u8>> {
        let mut k = meta_prefix.to_vec();
        k.extend_from_slice(key);
        self.areas[area_idx]
            .remove(k)
            .unwrap()
            .map(|iv| iv.to_vec())
    }
}

pub struct SledIter {
    inner: Iter,
}

impl Iterator for SledIter {
    type Item = (Vec<u8>, Vec<u8>);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|v| v.unwrap())
            .map(|(ik, iv)| (ik[PREFIX_SIZ..].to_vec(), iv.to_vec()))
    }
}

impl DoubleEndedIterator for SledIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|v| v.unwrap())
            .map(|(ik, iv)| (ik[PREFIX_SIZ..].to_vec(), iv.to_vec()))
    }
}

// key of the prefix allocator in the 'meta'
struct PrefixAllocator {
    key: [u8; 1],
}

impl PrefixAllocator {
    const fn init() -> (Self, PrefixBytes) {
        (
            Self {
                key: 0_u8.to_be_bytes(),
            },
            (RESERVED_ID_CNT + Prefix::MIN).to_be_bytes(),
        )
    }

    fn next(base: Option<&[u8]>) -> Option<[u8; PREFIX_SIZ]> {
        base.map(|bytes| (crate::parse_prefix!(bytes) + 1).to_be_bytes())
    }
}

fn sled_open() -> Result<Db> {
    let dir = get_data_dir();

    let db = Config::new()
        .path(&dir)
        .mode(Mode::HighThroughput)
        .use_compression(true)
        .open()
        .c(d!())?;

    // avoid setting again on an opened DB
    info_omit!(vsdb_set_base_dir(dir));

    Ok(db)
}