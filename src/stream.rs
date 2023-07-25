#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(generators)]
#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]

extern crate core;
use bytes::{BufMut, Bytes};
use std::future::Future;
use std::iter::once;
use std::ops::Deref;
use std::sync::Arc;
use anyhow::anyhow;
use itertools::Itertools;

use futures::{pin_mut, Stream, StreamExt};
use futures_async_stream::try_stream;
use mysql_async::prelude::*;


pub type StreamResult<T> = std::result::Result<T, anyhow::Error>;

#[derive(Debug)]
pub struct Chunk {
    pub chunk_str: String,
}

#[derive(Debug, Clone)]
pub struct SchemaTableName {
    pub schema_name: String,
    pub table_name: String,
}

pub trait ExternalTableReader {
    type SnapshotStream<'a>: Stream<Item=StreamResult<Option<Chunk>>> + Send + 'a
        where Self: 'a;

    fn get_normalized_table_name(table_name: &SchemaTableName) -> String;

    fn current_binlog_offset(&self) -> Option<String>;

    fn snapshot_read(&self, table_name: &SchemaTableName) -> Self::SnapshotStream<'_>;
}

// implement a mysql external table reader
pub struct MysqlExternalTableReader {
    pool: mysql_async::Pool,
}

impl MysqlExternalTableReader {
    pub fn new() -> Self {
        let pool = mysql_async::Pool::new("mysql://root:123456@localhost:8306/mydb");
        Self { pool }
    }
}

impl ExternalTableReader for MysqlExternalTableReader {
    type SnapshotStream<'a> = impl Stream<Item=StreamResult<Option<Chunk>>> + Send + 'a;

    fn get_normalized_table_name(table_name: &SchemaTableName) -> String {
        todo!()
    }

    fn current_binlog_offset(&self) -> Option<String> {
        println!("current binlog offset");
        None
    }

    fn snapshot_read(&self, table_name: &SchemaTableName) -> Self::SnapshotStream<'_> {
        #[try_stream]
        async move {
            let mut conn = self.pool.get_conn().await.map_err(|e| anyhow!("fail to get mysql conn: {}", e))?;
            let mut result_set = conn.query_iter("SELECT * FROM `t1`").await?;
            let s = result_set.stream::<mysql_async::Row>().await
                .map_err(|e| anyhow!("fail to read mysql: {}", e))?.unwrap();
            pin_mut!(s);
            #[for_await]
            for row in s {
                let row = row?;
                // println!("got {:?}", row);
                let chunk = Chunk {
                    chunk_str: format!("{:?}", row),
                };
                yield Some(chunk);
            }
        }
    }
}


#[tokio::test]
async fn test_mysql_table_reader() {
    let reader = MysqlExternalTableReader::new();
    let table_name = SchemaTableName {
        schema_name: "mydb".to_string(),
        table_name: "t1".to_string(),
    };

    let stream = reader.snapshot_read(&table_name);
    pin_mut!(stream);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        println!("chunk {:?}", chunk);
    }


    let _ = reader.current_binlog_offset();
}


#[tokio::test]
async fn test_mysql_client() {
    let pool = mysql_async::Pool::new("mysql://root:123456@localhost:8306/mydb");

    let mut conn = pool.get_conn().await.unwrap();
    let mut result_set = conn.query_iter("SELECT * FROM `t1`").await.unwrap();
    let s = result_set.stream::<mysql_async::Row>().await.unwrap().unwrap();
    pin_mut!(s);
    while let Some(row) = s.next().await {
        let row = row.unwrap();
        println!("got {:?}", row);
        let v1 = row.get::<u32, usize>(0);
        let v2 = row.get::<u32, usize>(1);
        println!("v1 {:?}, v2 {:?}", v1, v2);
    }
}

fn main() {


}