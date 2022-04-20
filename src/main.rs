use clap::Parser;

#[cfg(test)]
use std::collections::HashMap;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The key to partition on.
    pub key: String,
    /// The number of partitions of the topic.
    pub num_partitions: i32,
}

fn murmur2(data: &[u8]) -> i32 {
    let length = data.len() as i32;
    let seed = -1756908916 as i32; // 0x9747b28c;

    // 'm' and 'r' are mixing constants generated offline.
    // They're not really 'magic', they just happen to work well.
    let m = 1540483477; // 0x5bd1e995;
    let r = 24;

    // Initialize the hash to a random value
    let mut hash = seed ^ length;
    let length4 = length / 4;

    for i in 0..length4 {
        let i4 = i * 4;

        let p1 = (data[(i4 + 0) as usize] & 0xff) as i32;
        let p2 = ((data[(i4 + 1) as usize] & 0xff) as i32) << 8;
        let p3 = ((data[(i4 + 2) as usize] & 0xff) as i32) << 16;
        let p4 = ((data[(i4 + 3) as usize] & 0xff) as i32) << 24;

        let mut k = p1 + p2 + p3 + p4;

        k = k.wrapping_mul(m);
        k = ((k as u32) ^ ((k as u32) >> r)) as i32;
        k = k.wrapping_mul(m);

        hash = hash.wrapping_mul(m);
        hash = hash ^ k;
    }

    // Handle the last few bytes of the input array
    match length % 4 {
        3 => {
            let p = (data[((length & !3) + 2) as usize] & 0xff) as i32;
            hash ^= p << 16 as i32;
            let p = (data[((length & !3) + 1) as usize] & 0xff) as i32;
            hash ^= p << 8 as i32;
            hash ^= (data[(length & !3) as usize] & 0xff) as i32;
            hash = hash.wrapping_mul(m);
        }
        2 => {
            let p = (data[((length & !3) + 1) as usize] & 0xff) as i32;
            hash ^= p << 8 as i32;
            hash ^= (data[(length & !3) as usize] & 0xff) as i32;
            hash = hash.wrapping_mul(m);
        }
        1 => {
            hash ^= (data[(length & !3) as usize] & 0xff) as i32;
            hash = hash.wrapping_mul(m);
        }
        _ => {}
    }

    let hash = ((hash as u32) ^ ((hash as u32) >> 13)) as i32;
    let hash = hash.wrapping_mul(m);
    let hash = ((hash as u32) ^ ((hash as u32) >> 15)) as i32;

    println!("Murmur2 Hash: {}", hash);
    hash
}

fn to_positive(num: i32) -> i32 {
    num & 0x7fffffff
}

/// An implementation of Kafka's default partitioning strategy.
fn get_partition(key: &str, num_partitions: i32) -> i32 {
    let hash = murmur2(key.as_bytes());
    let pos = to_positive(hash);
    pos % num_partitions
}

fn main() {
    let args = Cli::parse();
    let p = get_partition(&args.key, args.num_partitions);
    println!("Partition: {}", p)
}

#[test]
fn murmur2_returns_correct_hash() {
    let mut cases = HashMap::new();
    cases.insert("21".as_bytes(), -973932308);
    cases.insert("foobar".as_bytes(), -790332482);
    cases.insert("a-little-bit-long-string".as_bytes(), -985981536);
    cases.insert("a-little-bit-longer-string".as_bytes(), -1486304829);
    cases.insert(
        "lkjh234lh9fiuh90y23oiuhsafujhadof229phr9h19h89h8".as_bytes(),
        -58897971,
    );

    for (case, expected) in cases.iter() {
        let actual = murmur2(*case);
        assert_eq!(actual, *expected);
    }
}

#[test]
fn get_partition_returns_correct_partition_number() {
    let mut cases = HashMap::new();
    cases.insert(vec!["21", "32"], 12);
    cases.insert(vec!["foobar", "32"], 30);
    cases.insert(vec!["testing", "16"], 3);
    cases.insert(
        vec!["a-very-big-long-key-to-test-1234567890987654321", "24"],
        18,
    );

    for (case, expected) in cases.iter() {
        let actual = get_partition(case[0], case[1].parse().unwrap());
        assert_eq!(actual, *expected);
    }
}
