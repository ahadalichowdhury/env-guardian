use std::fs;
use std::path::Path;

use tempfile::TempDir;

use env_guardian::commands::check;
use env_guardian::config::{EnvGuardianConfig, is_forbidden_env_commit};
use env_guardian::drift::compare::{compare_env_maps, DriftKind};
use env_guardian::env::parser::parse_env_contents;
use env_guardian::scanner::codebase::scan_content;
use env_guardian::share::crypto::{create_share, generate_keypair, load_private_key, load_public_key, open_share};
use env_guardian::vault::crypto::{decrypt_bytes, encrypt_bytes};

fn setup_project(dir: &Path) {
    let config = EnvGuardianConfig::default();
    config.save(dir).unwrap();

    fs::write(
        dir.join(".env.example"),
        "DATABASE_URL=\nAPI_KEY=\nPORT=3000\n",
    )
    .unwrap();
}

#[test]
fn env_parser_handles_comments_and_export() {
    let env = parse_env_contents("# hi\nexport FOO=bar\nEMPTY=");
    assert_eq!(env.vars.get("FOO"), Some(&"bar".to_string()));
    assert_eq!(env.vars.get("EMPTY"), Some(&String::new()));
}

#[test]
fn check_reports_missing_keys() {
    let tmp = TempDir::new().unwrap();
    setup_project(tmp.path());

    fs::write(tmp.path().join(".env"), "PORT=3000\n").unwrap();

    let passed = check::run(tmp.path(), None, false, true).unwrap();
    assert!(!passed);
}

#[test]
fn check_passes_when_keys_match() {
    let tmp = TempDir::new().unwrap();
    setup_project(tmp.path());

    fs::write(
        tmp.path().join(".env"),
        "DATABASE_URL=postgres://localhost\nAPI_KEY=secret\nPORT=3000\n",
    )
    .unwrap();

    let passed = check::run(tmp.path(), None, false, true).unwrap();
    assert!(passed);
}

#[test]
fn check_profile_development() {
    let tmp = TempDir::new().unwrap();
    let config = EnvGuardianConfig::default();
    config.save(tmp.path()).unwrap();

    fs::write(
        tmp.path().join(".env.example.development"),
        "DATABASE_URL=\nAPI_KEY=\nPORT=3000\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".env.development"),
        "DATABASE_URL=dev\nAPI_KEY=k\nPORT=3000\n",
    )
    .unwrap();

    let passed = check::run(tmp.path(), Some("development"), false, true).unwrap();
    assert!(passed);
}

#[test]
fn forbidden_env_commit_rules() {
    assert!(is_forbidden_env_commit(".env.development"));
    assert!(!is_forbidden_env_commit(".env.example.development"));
    assert!(!is_forbidden_env_commit(".env.development.enc"));
}

#[test]
fn scanner_detects_js_env_usage() {
    let src = "const u = process.env.DATABASE_URL;";
    let usages = scan_content(src, Path::new("app.ts")).unwrap();
    assert!(usages.iter().any(|u| u.key == "DATABASE_URL"));
}

#[test]
fn vault_roundtrip() {
    let plain = b"KEY=value\nOTHER=abc";
    let enc = encrypt_bytes(plain, "test-password").unwrap();
    let dec = decrypt_bytes(&enc, "test-password").unwrap();
    assert_eq!(dec, plain);
}

#[test]
fn vault_wrong_password() {
    let enc = encrypt_bytes(b"KEY=v", "right").unwrap();
    let err = decrypt_bytes(&enc, "wrong").unwrap_err();
    assert!(err.to_string().contains("decryption failed"));
}

#[test]
fn check_detects_codebase_undefined_keys() {
    let tmp = TempDir::new().unwrap();
    setup_project(tmp.path());

    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src/main.ts"),
        "const x = process.env.UNKNOWN_VAR;",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".env"),
        "DATABASE_URL=u\nAPI_KEY=k\nPORT=3000\n",
    )
    .unwrap();

    let passed = check::run(tmp.path(), None, false, false).unwrap();
    assert!(!passed);
}

#[test]
fn drift_compare_detects_mismatch() {
    let local = std::collections::BTreeMap::from([
        ("API_KEY".to_string(), "local".to_string()),
        ("PORT".to_string(), "3000".to_string()),
    ]);
    let remote = std::collections::BTreeMap::from([
        ("API_KEY".to_string(), "remote".to_string()),
        ("PORT".to_string(), "3000".to_string()),
    ]);
    let report = compare_env_maps(&local, &remote);
    assert!(report.has_drift());
    assert!(report
        .items
        .iter()
        .any(|i| i.key == "API_KEY" && i.kind == DriftKind::ValueMismatch));
}

#[test]
fn share_e2e_roundtrip() {
    let pair = generate_keypair().unwrap();
    let public = load_public_key(&pair.public).unwrap();
    let private = load_private_key(&pair.private).unwrap();
    let plain = b"SECRET=abc\nPORT=8080";
    let share = create_share(plain, &public, "test").unwrap();
    let opened = open_share(&share, &private).unwrap();
    assert_eq!(opened, plain);
}
