#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

extern crate core;

use bytes::{BufMut, Bytes};
use std::future::Future;
use std::io::Write;

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
    type NextFuture<'a> = impl Future<Output = Option<(&'a [u8], &'a [u8])>>;

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
