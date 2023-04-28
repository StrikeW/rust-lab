// rust call java demo

use std::fs;
use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;
use j4rs::errors::J4RsError;
use j4rs::{ClasspathEntry, InvocationArg, Jvm, JvmBuilder};
// use risingwave_pb::connector_service::table_schema::Column;
// use risingwave_pb::connector_service::TableSchema;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub enum BearType {
    Polar,
    Brown,
    Panda,
}

fn create_jvm() -> Result<Jvm, J4RsError> {
    // let jars = fs::read_dir("/home/willykid/Desktop/j4rs-test/libs").unwrap();
    // let mut jar_paths = jars
    //     .into_iter()
    //     .map(|jar| jar.unwrap().path().display().to_string())
    //     .collect_vec();
    // jar_paths
    //     .push("/home/willykid/Desktop/java-code/target/java-code-1.0-SNAPSHOT.jar".to_string());

    let jar_paths = vec!["/Users/siyuan/workspace/acting/rust/java-code/target/java-code-1.0-SNAPSHOT.jar".to_string()];

    let classpath_entries = jar_paths
        .iter()
        .map(|p| {
            println!("adding {} to class path", p.as_str());
            ClasspathEntry::new(p.as_str())
        })
        .collect_vec();

    JvmBuilder::new()
        .classpath_entries(classpath_entries)
        .build()
}

pub fn vec_to_args(v: Vec<&str>) -> Vec<InvocationArg> {
    v.into_iter()
        .map(|s| InvocationArg::try_from(s).unwrap())
        .collect_vec()
}

fn my_validate_test() {
    let jvm = create_jvm().unwrap();
    // test simple invocation
    jvm.invoke_static(
        "com.risingwave.connector.SinkUtils",
        "getSinkFactory",
        vec![InvocationArg::try_from("jdbc").unwrap()].as_slice(),
    )
        .unwrap();

    // test enum
    let bear_type = jvm
        .static_class_field("org.dummy.BearType", "Polar")
        .unwrap();
    let bt_rust: BearType = jvm
        .to_rust(
            jvm.static_class_field("org.dummy.BearType", "Polar")
                .unwrap(),
        )
        .unwrap();
    println!("the converted enum is: {:#?}", bt_rust);

    let bear = jvm
        .create_instance(
            "org.dummy.Bear",
            vec![
                InvocationArg::try_from("Bear").unwrap(),
                InvocationArg::try_from(bear_type).unwrap(),
            ]
                .as_slice(),
        )
        .unwrap();
    jvm.invoke(&bear, "shout", &Vec::new()).unwrap();

    // test channel
    let receiver = jvm
        .invoke_to_channel(&bear, "callBack", &Vec::new())
        .unwrap();
    for _ in 0..10 {
        let s: String = jvm.to_rust(receiver.rx().recv().unwrap()).unwrap();
        println!("{}", s);
    }

    let _bear_health = jvm
        .static_class_field("org.dummy.Bear$BearHealth", "Vaccinated")
        .unwrap();

    // test validate
    let source_type = jvm
        .static_class_field("com.risingwave.sourcenode.types.SourceType", "Postgres")
        .unwrap();
    let properties_str = vec![
        "hostname",
        "localhost",
        "port",
        "5432",
        "username",
        "postgres",
        "password",
        "test",
        "database.name",
        "test",
        "schema.name",
        "public",
        "table.name",
        "orders",
    ];
    let properties_args = vec_to_args(properties_str);
    let properties = jvm
        .invoke_static("java.util.Map", "of", properties_args.as_slice())
        .unwrap();

    // let col = Column {
    //     name: "o_key".to_string(),
    //     data_type: 1,
    // };
    //
    // let schema = TableSchema {
    //     columns: vec![col],
    //     pk_indices: vec![0],
    // };
    //
    // jvm.invoke_static(
    //     "com.risingwave.connector.SourceHandlerIpc",
    //     "handleValidate",
    //     vec![
    //         InvocationArg::try_from(source_type).unwrap(),
    //         InvocationArg::try_from(properties).unwrap(),
    //         InvocationArg::new(&schema, "com.risingwave.sourcenode.types.TableSchema"),
    //     ]
    //         .as_slice(),
    // )
    //     .unwrap();
}


fn test_call_java(jvm: &Jvm) {

    // test enum
    let bear_type = jvm
        .static_class_field("org.dummy.BearType", "Polar")
        .unwrap();
    let bt_rust: BearType = jvm
        .to_rust(
            jvm.static_class_field("org.dummy.BearType", "Polar")
                .unwrap(),
        )
        .unwrap();
    println!("the converted enum is: {:#?}", bt_rust);

    let bear = jvm
        .create_instance(
            "org.dummy.Bear",
            vec![
                InvocationArg::try_from("Bear").unwrap(),
                InvocationArg::try_from(bear_type).unwrap(),
            ]
                .as_slice(),
        )
        .unwrap();
    jvm.invoke(&bear, "shout", &Vec::new()).unwrap();

    // test channel
    let receiver = jvm
        .invoke_to_channel(&bear, "callBack", &Vec::new())
        .unwrap();
    for _ in 0..10 {
        let s: String = jvm.to_rust(receiver.rx().recv().unwrap()).unwrap();
        println!("{}", s);
    }

}

fn test_call_java_simple(jvm: Arc<Jvm>) -> String {
    let bear_type = jvm
        .static_class_field("org.dummy.BearType", "Polar")
        .unwrap();
    let bear = jvm
        .create_instance(
            "org.dummy.Bear",
            vec![
                InvocationArg::try_from("Siyuan").unwrap(),
                InvocationArg::try_from(bear_type).unwrap(),
            ]
                .as_slice(),
        )
        .unwrap();
    let ret = jvm.invoke(&bear, "shout", &Vec::new()).unwrap();
    let str = jvm.to_rust::<String>(ret).unwrap();
    str
}



#[tokio::main]
async fn main() {
    let jvm = Arc::new(create_jvm().unwrap());

    let fut = async {
        test_call_java_simple(jvm.clone())
    };


    println!("sleeping for 1 sec");
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("await java result");
    let s = fut.await;
    println!("s: {}", s);
}