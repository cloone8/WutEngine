use std::sync::Arc;

use wutengine_egui::egui;

use crate::assets::path::AssetPath;

mod directory_ui;
mod tree_ui;

#[derive(derive_more::Debug, Clone, derive_more::IsVariant)]
pub(super) enum AssetTreeNode {
    Dir {
        name: String,
        path: AssetPath,
        children: Vec<AssetTreeNode>,
    },
    Asset {
        asset_id: uuid::NonNilUuid,
        icon: &'static str,
        icon_color: egui::Color32,

        #[debug(skip)]
        on_open: Arc<dyn Fn(&uuid::NonNilUuid) + Send + Sync>,

        path: AssetPath,
        name: String,
    },
}

impl AssetTreeNode {
    pub(super) fn new_empty_dir(path: AssetPath) -> Self {
        let name = path.absolute().file_name().map_or_else(
            || "<UNKNOWN NAME>".to_string(),
            |file| file.to_string_lossy().to_string(),
        );

        Self::Dir {
            name,
            path,
            children: Vec::new(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Dir { name, .. } | Self::Asset { name, .. } => name.as_str(),
        }
    }

    fn path(&self) -> &AssetPath {
        match self {
            Self::Dir { path, .. } | Self::Asset { path, .. } => path,
        }
    }

    fn icon(&self) -> egui::RichText {
        match self {
            Self::Asset {
                icon, icon_color, ..
            } => egui::RichText::new(*icon).color(*icon_color),
            Self::Dir { .. } => egui::RichText::new("📁").color(egui::Color32::YELLOW),
        }
    }

    pub(super) fn clear(&mut self) {
        let AssetTreeNode::Dir { children, .. } = self else {
            return;
        };

        children.clear();
    }

    pub(super) fn insert_at(&mut self, node_path: &AssetPath, node: AssetTreeNode) {
        let AssetTreeNode::Dir { path, children, .. } = self else {
            panic!("Cannot insert at leaf node");
        };

        let should_insert_here = match node_path.absolute().parent() {
            Some(parent) => *path == AssetPath::new(parent),
            None => *path == AssetPath::root(),
        };

        if should_insert_here {
            // No more children to travel down, insert the new node here
            children.push(node);
            children.sort_by(|a, b| a.name().cmp(b.name()));
            return;
        }

        let mut to_insert = None;

        for ancestor_path in node_path.absolute().ancestors() {
            if ancestor_path == path.absolute() {
                break;
            }

            to_insert = Some(ancestor_path);
        }

        let to_insert = to_insert.expect("Should have at least one subdirectory here");

        let mut new_branch = AssetTreeNode::Dir {
            name: to_insert.file_name().unwrap().to_string_lossy().to_string(),
            path: AssetPath::new(to_insert),
            children: vec![],
        };

        new_branch.insert_at(node_path, node);

        children.push(new_branch);
        children.sort_by(|a, b| a.name().cmp(b.name()));
    }

    pub(super) fn get_node_at(&self, path: &AssetPath) -> Option<&AssetTreeNode> {
        if self.path() == path {
            // We're the referenced node
            return Some(self);
        }

        let Self::Dir {
            children,
            path: our_path,
            ..
        } = self
        else {
            // We have no children because we're not a directory, so the path doesn't exist
            return None;
        };

        if !our_path.is_ancestor_of(path) {
            // We're not an ancestor, so by definition our children do not contain the requested path
            return None;
        }

        children.iter().find_map(|child| child.get_node_at(path))
    }

    #[inline]
    pub(super) fn has_entry_at(&self, path: &AssetPath) -> bool {
        self.get_node_at(path).is_some()
    }
}
