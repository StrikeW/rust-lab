#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

extern crate core;

use std::collections::{BTreeMap, HashMap};
use bytes::{BufMut, Bytes};
use std::future::Future;
use std::io::Write;
use std::iter::once;
use std::ops::Deref;
use std::sync::Arc;
use itertools::Itertools;

const MAX_LEN: u32 = 4;
const MY_MASK: u32 = 0xff000000;

fn crc_hash() {
    let user_path = "100.data";
    let sst_id: u64 = 123456;
    let crc_hash: u32 = crc32fast::hash(&sst_id.to_be_bytes());
    println!("crc_hash: {}", crc_hash);
    let bytes = crc_hash.to_be_bytes();
    let mut res_bytes = &bytes[0..1];

    let mut vec = res_bytes.to_vec();
    if vec.len() < 4 {
        let zeros: Vec<u8> = vec![0, 0, 0, 0];
        vec.put_slice(&zeros[0..4 - res_bytes.len()]);
        res_bytes = vec.as_slice();
    }

    let hash_slot = u32::from_le_bytes(res_bytes.try_into().expect("incorrect length"));
    let obj_prefix = hash_slot.to_string();

    let mut s3_path = "shared_prefix/".to_string();
    s3_path.push_str(obj_prefix.as_str());
    s3_path.push('/');
    s3_path.push_str(user_path);

    println!("path {}", s3_path);
}

/// `async` block:
///
/// Multiple different `async` blocks can access the same local variable
/// so long as they're executed within the variable's scope
async fn blocks() {
    let my_string = "foo".to_string();

    let future_one = async {
        // ...
        println!("{my_string}");
    };

    let future_two = async {
        // ...
        println!("{my_string}");
    };

    // Run both futures to completion, printing "foo" twice:
    let ((), ()) = futures::join!(future_one, future_two);
}

/// GAT use case: impl an async trait by hand
/// https://www.skyzh.dev/posts/articles/2022-01-31-gat-async-trait/
pub trait KvIterator {
    type NextFuture<'a>: Future<Output = Option<(&'a [u8], &'a [u8])>>
    where
        Self: 'a;

    /// Get the next item from the iterator.
    fn next(&mut self) -> Self::NextFuture<'_>;
}

pub struct TestIterator {
    idx: usize,
    to_idx: usize,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl TestIterator {
    pub fn new(from_idx: usize, to_idx: usize) -> Self {
        Self {
            idx: from_idx,
            to_idx,
            key: Vec::new(),
            value: Vec::new(),
        }
    }
}

impl KvIterator for TestIterator {
    type NextFuture<'a> = impl Future<Output=Option<(&'a [u8], &'a [u8])>>;

    fn next(&mut self) -> Self::NextFuture<'_> {
        async move {
            if self.idx >= self.to_idx {
                return None;
            }

            self.key.clear();
            write!(&mut self.key, "key_{:05}", self.idx).unwrap();

            self.value.clear();
            write!(&mut self.value, "value_{:05}", self.idx).unwrap();

            self.idx += 1;
            Some((self.key.as_slice(), self.value.as_slice()))
        }
    }
}

pub struct ConcatIterator<Iter: KvIterator> {
    iters: Vec<Iter>,
    current_idx: usize,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl<Iter: KvIterator> ConcatIterator<Iter> {
    pub fn new(iters: Vec<Iter>) -> Self {
        Self {
            iters,
            current_idx: 0,
            key: vec![],
            value: vec![],
        }
    }
}

impl<Iter: KvIterator> KvIterator for ConcatIterator<Iter> {
    type NextFuture<'a> = impl Future<Output = Option<(&'a [u8], &'a [u8])>> where Iter: 'a;

    fn next(&mut self) -> Self::NextFuture<'_> {
        async move {
            loop {
                if self.current_idx >= self.iters.len() {
                    return None;
                }

                let iter = &mut self.iters[self.current_idx];
                match iter.next().await {
                    Some(item) => {
                        self.key.clear();
                        self.key.extend_from_slice(item.0);
                        self.value.clear();
                        self.value.extend_from_slice(item.1);
                        break Some((self.key.as_slice(), self.value.as_slice()));
                        // return Some(item);
                    }
                    None => {
                        self.current_idx += 1;
                    }
                }
            }
        }
    }
}

pub type MergedImmIter<'a> =
    impl Iterator<Item = (&'a Bytes, &'a Bytes)> + ExactSizeIterator;


struct MyIterator<'a> {
    iter: MergedImmIter<'a>,
}

struct MergedImmInner {
    payload: BTreeMap<Bytes, Bytes>,
    imm_ids: Vec<usize>
}

impl MergedImmInner {
    pub fn new(imm_ids: Vec<usize>) -> Self {
        Self { payload: BTreeMap::new(), imm_ids }
    }

    pub fn imm_ids(&self) -> &Vec<usize> {
        &self.imm_ids
    }
}


impl Deref for MergedImmInner {
    type Target = BTreeMap<Bytes, Bytes>;
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl<'a> MyIterator<'a> {
    fn new(imm: &'a MergedImmInner) -> Self {
        let iter = imm.iter();
        Self { iter }
    }

    fn next(&mut self) -> Option<(Bytes, Bytes)> {
        self.iter.next().map(|(k, v)| (k.clone(), v.clone()))
    }
}

struct Imm {
    batch_id: usize,
}

enum ImmImpl {
    Batch(Imm),
    Merged(MergedImmInner),
}


#[derive(Clone, Copy, Debug, Default, Hash, PartialOrd, PartialEq, Eq, Ord)]
pub struct TableId {
    pub table_id: u32,
}

#[tokio::test]
async fn test_range() {
    let mut map: HashMap<(TableId, u64), String> = HashMap::new();
    let mut v1 = vec![7, 6, 5, 4, 3, 2, 1];
    let mut v2 = vec![1, 2, 3, 4, 5, 6, 7];

    let idx1 = v1.partition_point(|x| x > &4);
    let idx2 = v2.partition_point(|x| x <= &4);
    println!("idx1 {}", idx1);
    println!("idx2 {}", idx2);

    let removed_v1 = v1.drain(idx1..).collect_vec();
    println!("v1 removed {:?}", removed_v1);
    println!("v1 remained {:?}", v1);

    let removed_v2 = v2.drain(0..idx2).collect_vec();
    println!("v2 removed {:?}", removed_v2);
    println!("v2 remained {:?}", v2);


    let table_1 = TableId { table_id: 1 };
    let table_2 = TableId { table_id: 2 };

    map.insert((table_1, 1), "shard1 of table 1".to_string());
    map.insert((table_1, 2), "shard2 of table 1".to_string());

    map.insert((table_2, 3), "shard3 of table 2".to_string());
    map.insert((table_2, 4), "shard4 of table 2".to_string());

    println!("{}", map[&(table_1, 1)]);
    println!("{}", map[&(table_1, 2)]);
    // println!("{}", map[&(table_1, 3)]);

    println!("{}", map[&(table_2, 3)]);
    println!("{}", map[&(table_2, 4)]);
}

struct Apple {
    id: u32,
}

impl Drop for Apple {
    fn drop(&mut self) {
        println!("drop apple {}", self.id);
    }
}


#[tokio::test]
async fn test_my_iterator() {
    let merged_imm = ImmImpl::Merged(MergedImmInner { payload: BTreeMap::new(), imm_ids: vec![10,11,12] });
    let imm1 =  ImmImpl::Batch(Imm { batch_id: 1 });
    let imm2 = ImmImpl::Batch(Imm { batch_id: 2 });

    let imms = vec![imm1, imm2, merged_imm];

    let imm_ids: Vec<usize> = imms.iter().flat_map(|imm| {
        match imm {
            ImmImpl::Batch(b) => {vec![b.batch_id].iter()}
            ImmImpl::Merged(m) => {m.imm_ids().iter()}
        }
    }).collect();

    println!("imm ids: {:?}", imm_ids);
}


#[tokio::test]
async fn test_concat_iterator() {
    let it1 = TestIterator::new(0, 7);
    let it2 = TestIterator::new(7, 15);

    let mut concat_ier = ConcatIterator::new(vec![it1, it2]);
    while let Some((key, value)) = concat_ier.next().await {
        println!(
            "{:?} {:?}",
            Bytes::copy_from_slice(key),
            Bytes::copy_from_slice(key)
        );
    }
}

#[tokio::test]
async fn test_iterator() {
    let mut iter = TestIterator::new(0, 10);
    while let Some((key, value)) = iter.next().await {
        println!(
            "{:?} {:?}",
            Bytes::copy_from_slice(key),
            Bytes::copy_from_slice(value)
        );
    }
}

#[tokio::main]
async fn main() {
    blocks().await;
}
