use fs_tree::TrieMap;

use crate::refactor::Refactor;

fn get_ign_num(children: &TrieMap) -> usize {
    0
}

impl Refactor {
    pub fn containment(self) -> Self {
        let tree = &self.state.tree;
        // iterate over nodes (parent nodes)
        for parent in tree.root.nodes() {
            if let Some(children) = parent.children() {
                let ign_num = get_ign_num(children);
                if ign_num > (children.len() + 1) / 2 {
                    // ignore parent
                    // re-include child(ren) not ignored
                }
            }
        }
        self
    }
}
