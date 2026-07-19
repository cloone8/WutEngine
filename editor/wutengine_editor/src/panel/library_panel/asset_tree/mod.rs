use wutengine_egui::egui;

use crate::assets::path::AssetPath;

mod directory_ui;
mod tree_ui;

#[derive(Debug, Clone)]
pub(super) enum AssetTreeNode {
    Branch {
        name: String,
        path: AssetPath,
        children: Vec<AssetTreeNode>,
    },
    Leaf {
        asset_id: uuid::NonNilUuid,
        icon: &'static str,
        icon_color: egui::Color32,
        path: AssetPath,
        name: String,
    },
}

impl AssetTreeNode {
    pub(super) fn new_empty_dir(path: AssetPath) -> Self {
        let name = path
            .absolute()
            .file_name()
            .map(|file| file.to_string_lossy().to_string())
            .unwrap_or_else(|| "<UNKNOWN NAME>".to_string());

        Self::Branch {
            name,
            path,
            children: Vec::new(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Branch { name, .. } => name.as_str(),
            Self::Leaf { name, .. } => name.as_str(),
        }
    }

    fn path(&self) -> &AssetPath {
        match self {
            Self::Branch { path, .. } => path,
            Self::Leaf { path, .. } => path,
        }
    }

    fn icon(&self) -> egui::RichText {
        match self {
            Self::Leaf {
                icon, icon_color, ..
            } => egui::RichText::new(*icon).color(*icon_color),
            Self::Branch { .. } => egui::RichText::new("📁").color(egui::Color32::YELLOW),
        }
    }

    pub(super) fn clear(&mut self) {
        let AssetTreeNode::Branch { children, .. } = self else {
            return;
        };

        children.clear();
    }

    pub(super) fn insert_at(&mut self, node_path: &AssetPath, node: AssetTreeNode) {
        let AssetTreeNode::Branch { path, children, .. } = self else {
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
        };

        let mut to_insert = None;

        for ancestor_path in node_path.absolute().ancestors() {
            if ancestor_path == path.absolute() {
                break;
            }

            to_insert = Some(ancestor_path);
        }

        let to_insert = to_insert.expect("Should have at least one subdirectory here");

        let mut new_branch = AssetTreeNode::Branch {
            name: to_insert.file_name().unwrap().to_string_lossy().to_string(),
            path: AssetPath::new(to_insert),
            children: vec![],
        };

        new_branch.insert_at(node_path, node);

        children.push(new_branch);
        children.sort_by(|a, b| a.name().cmp(b.name()));
    }
}
