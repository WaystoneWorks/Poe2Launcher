//! The launcher against a real `cargo xtask release-mirror` folder (the packages the workflows
//! make, built locally). Ignored by default: set `POE2_TEST_MIRROR=<dir>` (and, for the http
//! variant, `POE2_TEST_MIRROR_URL=http://127.0.0.1:<port>` of a server serving that folder).
//!
//!     cargo xtask release-mirror runtime/mirror-test
//!     POE2_TEST_MIRROR=$PWD/runtime/mirror-test cargo test --test mirror_flow -- --ignored

use std::sync::Arc;

use poe2_launcher_lib::{
    core::{Collector, Core},
    keystore::MemoryStore,
    paths::Dirs,
    source::{ReleaseSource, SourceKind},
    version::VersionFile,
};

fn mirror_source() -> Option<ReleaseSource> {
    let dir = std::env::var("POE2_TEST_MIRROR").ok()?;
    let base = std::env::var("POE2_TEST_MIRROR_URL").unwrap_or_else(|_| {
        reqwest::Url::from_file_path(&dir)
            .unwrap()
            .to_string()
            .trim_end_matches('/')
            .to_owned()
    });
    Some(ReleaseSource {
        kind: SourceKind::Static,
        base,
        ..Default::default()
    })
}

#[tokio::test]
#[ignore = "needs POE2_TEST_MIRROR=<release-mirror folder>"]
async fn a_real_mirror_installs_hook_and_server_and_feeds_the_lists() {
    let Some(src) = mirror_source() else {
        panic!("set POE2_TEST_MIRROR");
    };
    let tmp = tempfile::tempdir().unwrap();
    let core = Core::new(
        Dirs::under(tmp.path()),
        Arc::new(MemoryStore::new()),
        Arc::new(Collector::default()),
    )
    .unwrap();
    core.set_game_version(VersionFile::known_versions()[0])
        .unwrap();
    let mut s = core.settings().unwrap();
    s.release_source = src;
    core.set_settings(s).unwrap();

    let hook = core.hook_install_latest().await.unwrap();
    println!("hook {} at {}", hook.hook.version, hook.hook.dir.display());
    assert!(hook.hook.dir.join("poe_hook.dll").is_file());

    let id = core.host_create("Mirror test").unwrap();
    let out = core.host_install_latest(&id).await.unwrap();
    println!(
        "server {} at {}",
        out.package.server_version,
        out.package.dir.display()
    );
    assert!(out.checksum_verified);
    assert!(out.package.dir.join("release.json").is_file());

    let news = core.news_get(true).await;
    println!("news items: {}", news.items.len());
    let list = core.server_list_get(true).await;
    println!(
        "server list: {} servers, error {:?}",
        list.servers.len(),
        list.error
    );
    assert!(list.error.is_empty(), "{}", list.error);
    let idx = core.mod_index_get(true).await;
    println!("mod index: {} mods, error {:?}", idx.mods.len(), idx.error);
    assert!(idx.error.is_empty(), "{}", idx.error);
}
