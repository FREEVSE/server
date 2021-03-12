use std::{fs::File, iter::Filter};
use serde::{Serialize, Deserialize};
use semver::{Version, VersionReq};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Binary{
    version: Version,
    hardware: VersionReq,
    requires: VersionReq,
    file: String
}

#[derive(Clone)]
pub struct Manifest{
    path: String,
    manifest_changed: bool,

    binaries: Vec<Binary>
}

impl Manifest{
    pub fn new(path: String) -> Manifest{
        let file = File::open(path.clone()).unwrap();
        let bins = serde_json::from_reader(file).unwrap();

        Manifest { path: path, manifest_changed: true, binaries: bins}
    }

    pub async fn get_available_binaries(&self, hwv: Version, fwv: Version) -> Vec<Binary> {
        self.binaries.to_vec()
            .into_iter()
            .filter(|bin: &Binary| (
                bin.hardware.matches(&hwv) &&
                bin.requires.matches(&fwv)))
            .collect()
    }
}