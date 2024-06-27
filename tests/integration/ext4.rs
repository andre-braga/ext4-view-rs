// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ext4_view::{Ext4, Ext4Error, PathBuf};

fn load_test_disk1() -> Ext4 {
    const DATA: &[u8] = include_bytes!("../../test_data/test_disk1.bin");
    Ext4::load(Box::new(DATA.to_vec())).unwrap()
}

#[test]
fn test_read() {
    let fs = load_test_disk1();

    // Empty file.
    assert_eq!(fs.read("/empty_file").unwrap(), []);

    // Small file.
    assert_eq!(fs.read("/small_file").unwrap(), b"hello, world!");

    // File with holes.
    let mut expected = vec![];
    for i in 0..5 {
        expected.extend(vec![0xa5; 4096]);
        if i != 4 {
            expected.extend(vec![0; 8192]);
        }
    }
    assert_eq!(fs.read("/holes").unwrap(), expected);

    // Errors.
    assert!(fs.read("not_absolute").is_err());
    assert!(fs.read("/does_not_exist").is_err());
}

#[test]
fn test_read_to_string() {
    let fs = load_test_disk1();

    // Empty file.
    assert_eq!(fs.read_to_string("/empty_file").unwrap(), "");

    // Small file.
    assert_eq!(fs.read_to_string("/small_file").unwrap(), "hello, world!");

    // Errors:
    assert!(matches!(
        fs.read_to_string("/holes").unwrap_err(),
        Ext4Error::NotUtf8
    ));
    assert!(matches!(
        fs.read_to_string("/empty_dir").unwrap_err(),
        Ext4Error::IsADirectory
    ));
    assert!(matches!(
        fs.read_to_string("not_absolute").unwrap_err(),
        Ext4Error::NotAbsolute
    ));
    assert!(matches!(
        fs.read_to_string("/does_not_exist").unwrap_err(),
        Ext4Error::NotFound
    ));
    assert!(matches!(
        fs.read_to_string("\0").unwrap_err(),
        Ext4Error::MalformedPath
    ));
}

#[test]
fn test_read_dir() {
    let fs = load_test_disk1();

    // Get contents of directory `/big_dir`.
    let dir = fs
        .read_dir("/big_dir")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // Get the sorted list of entry names.
    let mut entry_names: Vec<String> = dir
        .iter()
        .map(|e| e.file_name().as_str().unwrap().to_owned())
        .collect();
    entry_names.sort_unstable();

    // Get the sorted list of entry paths.
    let mut entry_paths: Vec<PathBuf> = dir.iter().map(|e| e.path()).collect();
    entry_paths.sort_unstable();

    // Get expected entry names, 0-9999.
    let mut expected_names = vec![".".to_owned(), "..".to_owned()];
    expected_names.extend((0u32..10_000u32).map(|n| n.to_string()));
    expected_names.sort_unstable();

    // Get expected entry paths.
    let expected_paths = expected_names
        .iter()
        .map(|n| PathBuf::try_from(format!("/big_dir/{n}").as_bytes()).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(entry_names, expected_names);
    assert_eq!(entry_paths, expected_paths);

    // Errors:
    assert!(matches!(
        fs.read_dir("not_absolute").unwrap_err(),
        Ext4Error::NotAbsolute
    ));
    assert!(matches!(
        fs.read_dir("/empty_file").unwrap_err(),
        Ext4Error::NotADirectory
    ));
    assert!(matches!(
        fs.read_dir("\0").unwrap_err(),
        Ext4Error::MalformedPath
    ));
}
