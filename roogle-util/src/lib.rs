use std::collections::HashMap;

use rustdoc_types::{Crate, Id, Item, ItemSummary};

/// Perform a tree shaking to reduce the size of given `krate`.
pub fn shake(krate: Crate) -> Crate {
    let Crate {
        root,
        crate_version,
        includes_private,
        index,
        paths,
        format_version,
        ..
    } = krate;

    let index = shake_index(index);
    let paths = shake_paths(paths);
    let external_crates = HashMap::default();

    Crate {
        root,
        crate_version,
        includes_private,
        index,
        paths,
        external_crates,
        format_version,
    }
}

fn shake_index(index: HashMap<Id, Item>) -> HashMap<Id, Item> {
    use rustdoc_types::ItemEnum::*;

    index
        .into_iter()
        .filter(|(_, item)| {
            matches!(
                item.inner,
                Function(_) | Method(_) | Trait(_) | Impl(_) | Typedef(_) | AssocConst { .. }
            )
        })
        .collect()
}

fn shake_paths(paths: HashMap<Id, ItemSummary>) -> HashMap<Id, ItemSummary> {
    use rustdoc_types::ItemKind::*;

    paths
        .into_iter()
        .filter(|(_, item)| matches!(item.kind, Struct | Union | Enum | Function | Trait | Method))
        .collect()
}
