//!
//! # Common components
//!

#![allow(dead_code)]

pub(crate) mod ende;
pub(crate) mod engines;

use {
    engines::Engine,
    once_cell::sync::Lazy,
    parking_lot::Mutex,
    ruc::*,
    std::{
        env, fs,
        mem::size_of,
        sync::atomic::{AtomicBool, Ordering},
    },
};

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

pub(crate) type RawBytes = Box<[u8]>;
pub(crate) type RawKey = RawBytes;
pub(crate) type RawValue = RawBytes;

pub(crate) type Prefix = u64;
pub(crate) type PrefixBytes = [u8; PREFIX_SIZ];
pub(crate) const PREFIX_SIZ: usize = size_of::<Prefix>();

pub(crate) type BranchID = u64;
pub(crate) type VersionID = u64;

/// Avoid making mistakes between branch name and version name.
#[derive(Clone, Copy, Debug)]
pub struct BranchName<'a>(pub &'a [u8]);
/// Avoid making mistakes between branch name and version name.
#[derive(Clone, Copy, Debug)]
pub struct ParentBranchName<'a>(pub &'a [u8]);
/// Avoid making mistakes between branch name and version name.
#[derive(Clone, Copy, Debug)]
pub struct VersionName<'a>(pub &'a [u8]);

const RESERVED_ID_CNT: Prefix = 4096_0000;
pub(crate) const BIGGEST_RESERVED_ID: Prefix = RESERVED_ID_CNT - 1;
pub(crate) const NULL: BranchID = BIGGEST_RESERVED_ID as BranchID;

pub(crate) const INITIAL_BRANCH_ID: BranchID = 0;
pub(crate) const INITIAL_BRANCH_NAME: &[u8] = b"main";

/// The initial verison along with each new instance.
pub const INITIAL_VERSION: VersionName<'static> = VersionName([0u8; 0].as_slice());

/// How many ancestral branches at most one new branch can have.
pub const BRANCH_ANCESTORS_LIMIT: usize = 128;

// default value for reserved number when pruning old data
pub(crate) const RESERVED_VERSION_NUM_DEFAULT: usize = 10;

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

const BASE_DIR_VAR: &str = "VSDB_BASE_DIR";

static VSDB_BASE_DIR: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(gen_data_dir()));

static VSDB_CUSTOM_DIR: Lazy<String> = Lazy::new(|| {
    let d = VSDB_BASE_DIR.lock().clone() + "/__CUSTOM__";
    fs::create_dir_all(&d).unwrap();
    env::set_var("VSDB_CUSTOM_DIR", &d);
    d
});

#[cfg(all(feature = "sled_engine", not(feature = "rocks_engine")))]
pub(crate) static VSDB: Lazy<VsDB<engines::Sled>> = Lazy::new(|| pnk!(VsDB::new()));

#[cfg(all(feature = "rocks_engine", not(feature = "sled_engine")))]
pub(crate) static VSDB: Lazy<VsDB<engines::RocksDB>> = Lazy::new(|| pnk!(VsDB::new()));

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

/// Parse bytes to a specified integer type.
#[macro_export(crate)]
macro_rules! parse_int {
    ($bytes: expr, $ty: ty) => {{
        let array: [u8; std::mem::size_of::<$ty>()] = $bytes[..].try_into().unwrap();
        <$ty>::from_be_bytes(array)
    }};
}

/// Parse bytes to a `Prefix` type.
#[macro_export(crate)]
macro_rules! parse_prefix {
    ($bytes: expr) => {
        $crate::parse_int!($bytes, $crate::common::Prefix)
    };
}

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

pub(crate) struct VsDB<T: Engine> {
    db: T,
}

impl<T: Engine> VsDB<T> {
    #[inline(always)]
    fn new() -> Result<Self> {
        Ok(Self {
            db: T::new().c(d!())?,
        })
    }

    #[inline(always)]
    pub(crate) fn alloc_branch_id(&self) -> BranchID {
        self.db.alloc_branch_id()
    }

    #[inline(always)]
    pub(crate) fn alloc_version_id(&self) -> VersionID {
        self.db.alloc_version_id()
    }

    #[inline(always)]
    fn flush(&self) {
        self.db.flush()
    }
}

/////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////

#[inline(always)]
fn gen_data_dir() -> String {
    // Compatible with Windows OS?
    let d = env::var(BASE_DIR_VAR)
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.vsdb", h)))
        .unwrap_or_else(|_| "/tmp/.vsdb".to_owned());
    fs::create_dir_all(&d).unwrap();
    d
}

/// ${VSDB_CUSTOM_DIR}
#[inline(always)]
pub fn vsdb_get_custom_dir() -> String {
    VSDB_CUSTOM_DIR.clone()
}

/// ${VSDB_BASE_DIR}
#[inline(always)]
pub fn vsdb_get_base_dir() -> String {
    VSDB_BASE_DIR.lock().clone()
}

/// Set ${VSDB_BASE_DIR} manually.
#[inline(always)]
pub fn vsdb_set_base_dir(dir: String) -> Result<()> {
    static HAS_INITED: AtomicBool = AtomicBool::new(false);

    if HAS_INITED.swap(true, Ordering::Relaxed) {
        Err(eg!("VSDB has been initialized !!"))
    } else {
        env::set_var(BASE_DIR_VAR, &dir);
        *VSDB_BASE_DIR.lock() = dir;
        Ok(())
    }
}

/// Flush data to disk, may take a long time.
#[inline(always)]
pub fn vsdb_flush() {
    VSDB.flush();
}

macro_rules! impl_from_for_name {
    ($target: tt) => {
        impl<'a> From<&'a [u8]> for $target<'a> {
            fn from(t: &'a [u8]) -> Self {
                $target(t)
            }
        }
        impl<'a> From<&'a Vec<u8>> for $target<'a> {
            fn from(t: &'a Vec<u8>) -> Self {
                $target(t.as_slice())
            }
        }
        impl<'a> From<&'a str> for $target<'a> {
            fn from(t: &'a str) -> Self {
                $target(t.as_bytes())
            }
        }
        impl<'a> From<&'a String> for $target<'a> {
            fn from(t: &'a String) -> Self {
                $target(t.as_bytes())
            }
        }
    };
    ($target: tt, $($t: tt),+) => {
        impl_from_for_name!($target);
        impl_from_for_name!($($t), +);
    };
}

impl_from_for_name!(BranchName, ParentBranchName, VersionName);

impl Default for BranchName<'static> {
    fn default() -> Self {
        BranchName(INITIAL_BRANCH_NAME)
    }
}
