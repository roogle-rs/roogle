use std::{
    env::{self, temp_dir},
    process::Command,
};

use crates_io_api::SyncClient;

fn main() {
    let workdir = env::current_dir().unwrap();
    let assets = workdir.join("assets");

    let krate = env::args().nth(1).expect("specify a krate to index");

    let client = SyncClient::new(
        "roogle (git@hkmatsumoto.com)",
        std::time::Duration::from_secs(2),
    )
    .expect("failed to instantiate client");

    let krate = client
        .get_crate(&krate)
        .expect(&format!("failed to get crate `{}`", krate));
    let name = krate.crate_data.name;
    let version = krate.crate_data.max_version;

    let tmp = temp_dir();
    let path = tmp.join(format!("{}.tar.gz", name));
    let mut tar = std::fs::File::create(path).unwrap();

    let url = format!(
        "https://static.crates.io/crates/{name}/{name}-{version}.crate",
        name = name,
        version = version
    );
    let mut resp = reqwest::blocking::get(url).expect("request failed");

    std::io::copy(&mut resp, &mut tar).unwrap();

    std::env::set_current_dir(&tmp).unwrap();
    Command::new("tar")
        .args(&["-xf", &format!("{}.tar.gz", name)])
        .output()
        .expect("extracting tar file failed");

    std::env::set_current_dir(&tmp.join(format!("{}-{}", name, version))).unwrap();
    Command::new("cargo")
        .args(&[
            "+stage1",
            "rustdoc",
            "--",
            "--output-format",
            "json",
            "-Z",
            "unstable-options",
        ])
        .output()
        .expect("generating index failed");
    Command::new("mv")
        .arg(format!("target/doc/{}.json", name))
        .arg(assets.to_str().unwrap())
        .output()
        .expect("moving index file to `assets/` failed");
}
