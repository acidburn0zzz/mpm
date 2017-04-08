extern crate rpf;

use std::process::Command;
use std::env;

fn mpm_path() -> String {
    let mut current_dir = env::current_dir().unwrap();
    current_dir.push("target/debug/mpm");
    return current_dir.to_str().unwrap().to_string();
}

#[test]
fn test_mpm_help() {
    let prog = Command::new(mpm_path())
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("Failed to run test for mpm: {}", e));
    println!("{}", String::from_utf8_lossy(&prog.stdout));
    assert_eq!(prog.status.success(), true);
}

#[test]
fn test_mpm_build_clean_tar() {
    let mpm_path = mpm_path();
    println!("{}", mpm_path);
    let build = Command::new(&mpm_path)
        .arg("build")
        .current_dir("example/tar")
        .output()
        .unwrap_or_else(|e| panic!("Failed to run test for mpm: {}", e));
    println!("{}", String::from_utf8_lossy(&build.stdout));

    let clean = Command::new(&mpm_path)
        .arg("build")
        .arg("clean")
        .current_dir("example/tar")
        .output()
        .unwrap_or_else(|e| panic!("Failed to run test for mpm: {}", e));
    println!("{}", String::from_utf8_lossy(&clean.stdout));

    assert_eq!(build.status.success(), true);
    assert_eq!(clean.status.success(), true);
}

#[test]
fn test_mpm_build_clean_git() {
    let mpm_path = mpm_path();
    println!("{}", mpm_path);
    let build = Command::new(&mpm_path)
        .arg("build")
        .current_dir("example/git")
        .output()
        .unwrap_or_else(|e| panic!("Failed to run test for mpm: {}", e));
    println!("{}", String::from_utf8_lossy(&build.stdout));

    let clean = Command::new(&mpm_path)
        .arg("build")
        .arg("clean")
        .current_dir("example/git")
        .output()
        .unwrap_or_else(|e| panic!("Failed to run test for mpm: {}", e));
    println!("{}", String::from_utf8_lossy(&clean.stdout));

    assert_eq!(build.status.success(), true);
    assert_eq!(clean.status.success(), true);
}

#[test]
fn test_mpm_build_clean_self() {
    let mpm_path = mpm_path();
    println!("{}", mpm_path);
    let build = Command::new(&mpm_path)
        .arg("build")
        .current_dir("example/mpm-git")
        .output()
        .unwrap_or_else(|e| panic!("Failed to run test for mpm: {}", e));
    println!("{}", String::from_utf8_lossy(&build.stdout));

    let clean = Command::new(&mpm_path)
        .arg("build")
        .arg("clean")
        .current_dir("example/mpm-git")
        .output()
        .unwrap_or_else(|e| panic!("Failed to run test for mpm: {}", e));
    println!("{}", String::from_utf8_lossy(&clean.stdout));

    assert_eq!(build.status.success(), true);
    assert_eq!(clean.status.success(), true);
}
