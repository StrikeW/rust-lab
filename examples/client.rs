use std::sync::{Arc, atomic};
use std::sync::atomic::{AtomicU64, Ordering};

use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::oneshot::{Receiver, Sender};

use crate::Store::{AStore, UStore};

static G_IDGEN: AtomicU64 = atomic::AtomicU64::new(1);

#[derive(Clone, Copy)]
struct Instance {
    id: u64,
}

enum Store {
    UStore(Instance),
    AStore(Instance),
}

impl Clone for Store {
    fn clone(&self) -> Self {
        match self {
            Store::UStore(o) => UStore(Instance {
                id: G_IDGEN.fetch_add(1, Ordering::SeqCst),
            }),
            Store::AStore(o) => AStore(Instance {
                id: G_IDGEN.fetch_add(1, Ordering::SeqCst),
            }),
        }
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        match self {
            UStore(o) => {
                println!("drop ustore id: {}", o.id)
            }
            AStore(o) => {
                println!("drop astore id: {}", o.id)
            }
        }
    }
}

#[macro_export]
macro_rules! vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

#[tokio::main]
async fn main() {

    let v = vec![1,2,3,4,5];

    let mut viter = v.into_iter();
    let mut opt = Some(viter);
    if let Some(res) = opt.as_mut() {
        for i in res.take(3) {
            println!("{}", i);
        }

        for i in res.take(3) {
            println!("{}", i);
        }

    }


    // if let Store::UStore(mut o) = store_impl.clone() {
    //     println!("start process ustore id: {}", o.id);
    //     o.id = 999;
    //     println!("finish process ustore id: {}", o.id);
    // }
    //
    // println!("done");
}