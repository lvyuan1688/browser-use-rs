//! compress: reduce a full DOM tree to the minimal set of nodes an agent
//! needs to make a decision. The skeleton drops text nodes, empty attrs,
//! and prunes children beyond a small depth.

use browser_core::DomNode;

/// Maximum retained depth. Deeper nodes are folded into their parent.
pub const MAX_DEPTH: usize = 6;

/// Compress a DOM tree in place.
pub fn compress(node: &mut DomNode, depth: usize) {
    if depth >= MAX_DEPTH {
        node.children.clear();
        return;
    }
    node.attrs.retain(|_, v| !v.is_empty());
    node.children.retain(|c| {
        !(c.text.is_none() && c.attrs.is_empty() && c.children.is_empty())
    });
    for c in node.children.iter_mut() {
        compress(c, depth + 1);
    }
}

/// Compress a list of root nodes.
pub fn compress_all(roots: &mut Vec<DomNode>) {
    for r in roots.iter_mut() {
        compress(r, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn prunes_empty_children() {
        let mut node = DomNode {
            tag: "div".into(),
            text: None,
            attrs: BTreeMap::new(),
            children: vec![DomNode {
                tag: "span".into(),
                text: None,
                attrs: BTreeMap::new(),
                children: vec![],
            }],
        };
        compress(&mut node, 0);
        assert!(node.children.is_empty());
    }
}
