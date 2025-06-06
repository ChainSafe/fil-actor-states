// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::v13::{Multimap, parse_uint_key, u64_key};
use fvm_ipld_amt::Amt;
use fvm_ipld_blockstore::MemoryBlockstore;
use fvm_shared4::HAMT_BIT_WIDTH;
use fvm_shared4::address::Address;

#[test]
fn basic_add() {
    let store = MemoryBlockstore::new();
    let mut mm = Multimap::new(&store, HAMT_BIT_WIDTH, 3);

    let addr = Address::new_id(100);
    assert_eq!(mm.get::<u64>(&addr.to_bytes()).unwrap(), None);

    mm.add(addr.to_bytes().into(), 8).unwrap();
    let arr: Amt<u64, _> = mm.get(&addr.to_bytes()).unwrap().unwrap();
    assert_eq!(arr.get(0).unwrap(), Some(&8));

    mm.add(addr.to_bytes().into(), 2).unwrap();
    mm.add(addr.to_bytes().into(), 78).unwrap();
}

#[test]
fn for_each() {
    let store = MemoryBlockstore::new();
    let mut mm = Multimap::new(&store, HAMT_BIT_WIDTH, 3);

    let addr = Address::new_id(100);
    assert_eq!(mm.get::<u64>(&addr.to_bytes()).unwrap(), None);

    mm.add(addr.to_bytes().into(), 8).unwrap();
    mm.add(addr.to_bytes().into(), 2).unwrap();
    mm.add(addr.to_bytes().into(), 3).unwrap();
    mm.add("Some other string".into(), 7).unwrap();

    let mut vals: Vec<(u64, u64)> = Vec::new();
    mm.for_each(&addr.to_bytes(), |i, v| {
        vals.push((i, *v));
        Ok(())
    })
    .unwrap();

    assert_eq!(&vals, &[(0, 8), (1, 2), (2, 3)])
}

#[test]
fn remove_all() {
    let store = MemoryBlockstore::new();
    let mut mm = Multimap::new(&store, HAMT_BIT_WIDTH, 3);

    let addr1 = Address::new_id(100);
    let addr2 = Address::new_id(101);

    mm.add(addr1.to_bytes().into(), 8).unwrap();
    mm.add(addr1.to_bytes().into(), 88).unwrap();
    mm.add(addr2.to_bytes().into(), 1).unwrap();

    let arr: Amt<u64, _> = mm.get(&addr1.to_bytes()).unwrap().unwrap();
    assert_eq!(arr.get(1).unwrap(), Some(&88));

    mm.remove_all(&addr1.to_bytes()).unwrap();
    assert_eq!(mm.get::<u64>(&addr1.to_bytes()).unwrap(), None);

    assert!(mm.get::<u64>(&addr2.to_bytes()).unwrap().is_some());
    mm.remove_all(&addr2.to_bytes()).unwrap();
    assert_eq!(mm.get::<u64>(&addr2.to_bytes()).unwrap(), None);
}

#[test]
fn varint_key() {
    let k = u64_key(1);
    let out = parse_uint_key(&k).unwrap();
    assert_eq!(1, out);
}
