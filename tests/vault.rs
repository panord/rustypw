use rlib::vault::UnlockedVault;
use std::fs;
use std::path::PathBuf;

#[test]
fn lock_unlock() {
    let uv = UnlockedVault::new("test");
    let pw = "password";
    if uv.lock(pw).unwrap().unlock(pw).is_err() {
        panic!("Failed to unlock vault.");
    }
}

#[test]
fn import_export() {
    let mut export = UnlockedVault::new("export");
    let mut import = UnlockedVault::new("import");
    let path = PathBuf::from("/tmp/rpwtest/");
    let stored = String::from("password");
    let key = String::from("pw");
    let fpath = PathBuf::from(format!("/tmp/rpwtest/{}", "export.vlt"));

    fs::create_dir_all(&path).expect("Failed creating test directory.");

    export.insert(key.clone(), stored.clone());
    if export.export(&fpath).is_err() {
        panic!("Failed exporting vault");
    }
    if import.import(&fpath).is_err() {
        panic!("Failed importing vault");
    }
    assert_eq!(import.get(key.clone()), Ok(&stored));
}
