// rust type exercise
// https://www.skyzh.dev/posts/articles/2022-01-22-rust-type-exercise-in-database-executors/

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::result;
use std::task::{ready, Context, Poll};


use serde::{Deserialize, Serialize};
use serde_json::Result;
use tokio::task::JoinHandle;


trait MyTrait {
    fn method(&self) -> String;
}

struct A {
    a: String,
}

struct B {
    b: u32,
}

impl MyTrait for A {
    fn method(&self) -> String {
        "A".to_string()
    }
}

impl MyTrait for B {
    fn method(&self) -> String {
        "B".to_string()
    }
}

enum MyEnum {
    Imm,
    MergedImm
}

impl MyEnum {
    fn myenum_method(self) -> Box<dyn MyTrait> {
        match self {
            MyEnum::Imm => Box::new(A { a: "aaa".to_string() }),
            MyEnum::MergedImm => Box::new(B { b: 111 }),
        }
    }
}


#[tokio::main]
async fn main() {
    let myenum1 = MyEnum::MergedImm;
    let myenum2 = MyEnum::Imm;

    println!("Enum1: {}", myenum1.myenum_method().method());
    println!("Enum2: {}", myenum2.myenum_method().method());

}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateLinkInfoItem {
    pub service_name: String,
    pub availability_zones: Vec<String>,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename(deserialize = "private_links"))]
pub struct PrivateLinkInfo {
    pub provider: String,
    pub infos: Vec<PrivateLinkInfoItem>,
}


struct TestTask {
    join_handle: JoinHandle<String>
}

impl TestTask {
    pub fn new() -> Self {
        let join_handle = tokio::spawn(async move {
            let str = "Siyuan";
            str.to_string()
        });
        Self {
            join_handle,
        }
    }

    fn poll_result(&mut self, cx: &mut Context<'_>) -> Poll<result::Result<String, String>> {
        Poll::Ready(match ready!(self.join_handle.poll_unpin(cx)) {
            Ok(task_result) => Ok(task_result),
            Err(_) => Err(format!("join error")),
        })
    }

}

impl Future for TestTask {
    type Output = result::Result<String, String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_result(cx)
    }
}




#[tokio::test]
async fn test_json() {
    // let mut map = HashMap::new();
    // let data = r#"
    //     {
    //         "provider": "aws",
    //         "infos": [{"service_name": "xxx", "availability_zones": ["az1"], "port": 9001}]
    //     } "#;
    // map.insert("a".to_string(), "b".to_string());
    // map.insert("aa".to_string(), "bb".to_string());

    let v1 = vec![1, 2, 3];

    v1.iter().for_each(|x| {
        println!("{}", x);
    });


    // let task = TestTask::new();
    //
    // match task.await {
    //     Ok(res) => {
    //         println!("result: {:?}", res);
    //     }
    //     Err(err) => {
    //         println!("error occurred: {}", err);
    //     }
    // }
    //
    //
    // task.join_handle.abort();
    // println!("abort success");

    // map.insert("private_links", data);
    //
    // map.get("private_links").map_or((), |str| {
    //     let link_info = serde_json::from_str::<PrivateLinkInfo>(str).unwrap();
    //     println!("{:?}", link_info);
    // });

    // let j = serde_json::to_string(&map).unwrap();
    // println!("{}", j);



}
