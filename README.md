![GitHub top language](https://img.shields.io/github/languages/top/ccmlm/VSDB)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ccmlm/VSDB/Rust)
![GitHub issues](https://img.shields.io/github/issues-raw/ccmlm/VSDB)
![GitHub pull requests](https://img.shields.io/github/issues-pr-raw/ccmlm/VSDB)

# VSDB

VSDB is a 'Git' in the form of KV-database.

Based on the powerful version control function of VSDB, you can easily give your data structure the ability to version management.

**Make everything versioned !!**

## Highlights

- Support Git-like verison operations, such as:
    - Create countless branches and merge them to their parents
    - Rolling back a 'branch' to a specified historical 'version'
    - Querying the historical value of a key on the specified 'branch'
- Most APIs is similar as the coresponding data structures in the standard library
    - Use `Vecx` just like `Vec`
    - Use `Mapx` just like `HashMap`
    - Use `MapxOrd` just like `BTreeMap`
- ...

## Examples

Suppose you have a great algorithm like this:

```rust
struct GreatAlgo {
    a: Vec<...>,
    b: BTreeMap<...>,
    c: u128,
    d: HashMap<...>,
    e: ...
}
```

Simply replace the original structure with the corresponding VSDB data structure,
and your algorithm get the powerful version control ability at once!

```rust
#[dervive(Vs, Default)]
struct GreatAlgo {
    a: VecxVs<...>,
    b: MapxOrdVs<...>,
    c: OrphanVs<u128>,
    d: MapxVs<...>,
    e: ...
}

let algo = GreatAlgo.default();

algo.get_by_branch_version(...);
algo.branch_create(...);
algo.branch_create_by_base_branch(...);
algo.branch_create_by_base_branch_version(...);
algo.branch_remove(...);
algo.version_pop(...);
algo.prune();
```

**NOTE !!**

the `#[derive(Vs)]` macro can be applied to structures
whose internal fields are all types defined in VSDB
(primitive types and their collections are also supported),
but can not be applied to nesting wrapper among VSDB-types,
you should implement the `VsMgmt` trait(or a part of it) manually.

This data structure can be handled correctly by `#[derive(Vs)]`:

```rust
#[derive(Vs)]
struct GoodCase<K, T> {
    a: VecxVs<i64>,
    b: SubItem0,
    c: SubItem1,
    d: SubItem2,
    e: u8,
    f: Vec<i16>,
    g: VecDeque<i64>,
    h: BTreeSet<u16>,
    i: HashMap<K, AtomicU64>,
    j: HashSet<i32>,
    k: LinkedList<()>,
    l: Box<dyn AsRef<bool>,
    m: Box<dyn AsRef<[Vec<u128>]>>,
    n: PhantomData<T>,
}

#[derive(Vs)]
struct SubItem0(MapxVs<u8, u8>, VecxVs<u8>);

#[derive(Vs)]
struct SubItem1 {
    a: OrphanVs<i16>,
    b: MapxOrdVs<String, u8>
}

#[derive(Vs)]
struct SubItem2 {
    a: i8,
    b: u128
}

// // A nope implementation of `VsMgmt` for custom stateless types.
// // the `#[derive(Vs)]` on 'SubItem2' is same as this implementation.
// impl VsMgmt for SubItem2 {
//     impl_vs_methods_nope!();
// }
```

**But** this one can NOT be handled correctly by `#[derive(Vs)]`:

```rust
// It can be compiled, but the result is wrong !
// The versioned methods of the inner 'MapxVs<u8, u8>' will missing,
// you should implement the 'VsMgmt' trait(or a part of it) manually.
#[derive(Vs)]
struct BadCase {
    a: VecxVs<MapxVs<u8, u8>>,
}
```

Some complete examples:

- Versioned examples:
    - [**Simple blockchain**](examples/blockchain_state.rs)
    - [**Operations one by one**](src/versioned/mapx_raw/test.rs)
- Unversioned examples:
    - [**Simple web server**](examples/web_server.rs)
    - [**Operations one by one**](src/basic/mapx/test.rs)

## Compilation features

- [**default**] `sled_engine`, use sled as the backend database
    - Faster compilation speed
    - Support for compiling into a statically linked binary
- `rocks_engine`, use rocksdb as the backend database
    - Faster running speed
    - Can not be compiled into a statically linked binary
- [**default**] `cbor_codec`, use cbor as the codec
    - Faster running speed
- `bcs_codec`, use bcs as the codec
    - Created by the 'Libre' project of Facebook
    - Security reinforcement for blockchain scenarios

## Low-level design

Based on the underlying one-dimensional linear storage structure (native kv-database, such as sled/rocksdb, etc.), multiple different namespaces are divided, and then abstract each dimension in the multi-dimensional logical structure based on these divided namespaces.

In the category of kv-database, namespaces can be expressed as different key ranges, or different key prefix.

This is the same as expressing complex data structures in computer memory(the memory itself is just a one-dimensional linear structure).

User data will be divided into two dimensions: 'branch' and 'version', the functions of the 'basic' category are stateless, and the functions of the 'versioned' category are stateful. In the internal implementation, each stateful function is implemented based on its corresponding stateless function,
all stateful data has two additional identification dimensions ('branch' and 'version'), somewhat like the logic in Git. Stateless functions do not have the feature of 'version' management, but they have higher performance.
