use bytes::BufMut;


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
        let zeros : Vec<u8> = vec![0,0,0,0];
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



fn main() {
}