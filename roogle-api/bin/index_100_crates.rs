use std::{
    env::{self, temp_dir},
    process::Command,
};

use crates_io_api::{ListOptions, Sort, SyncClient};
use indicatif::ProgressBar;

fn main() {
    let workdir = env::current_dir().unwrap();
    let assets = workdir.join("assets");

    let client = SyncClient::new(
        "roogle (git@hkmatsumoto.com)",
        std::time::Duration::from_secs(1),
    )
    .expect("failed to instantiate client");

    let krates = client
        .crates(ListOptions {
            sort: Sort::Downloads,
            per_page: 100,
            page: 1,
            query: None,
        })
        .expect("failed to get crates");
    let pb = ProgressBar::new(krates.crates.len() as u64);
    for krate in krates.crates {
        pb.inc(1);

        let name = krate.name;
        let version = krate.max_version;

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
                "+nightly",
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
}
