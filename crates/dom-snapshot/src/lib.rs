//! dom-snapshot — structured DOM snapshot for browser-use-rs.
//!
//! A `DomSnapshot` captures a moment of the page: tree of `DomNode`s,
//! viewport metadata, and a timestamp. Snapshots diff structurally via
//! `Diff::between`, yielding `DiffOp`s (Added / Removed / Changed).
//! Use for action-replay state comparison and TUI history rewind.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single DOM node in the snapshot tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomNode {
    /// Stable id assigned by the snapshotter (matches across snapshots
    /// when the node is structurally unchanged).
    pub id: u32,
    /// Tag name, e.g. "div", "button".
    pub tag: String,
    /// Element attributes (excludes children/text).
    pub attrs: HashMap<String, String>,
    /// Direct text content (whitespace-trimmed).
    pub text: Option<String>,
    /// Child node ids.
    pub children: Vec<u32>,
}

impl DomNode {
    pub fn new(id: u32, tag: impl Into<String>) -> Self {
        Self {
            id,
            tag: tag.into(),
            attrs: HashMap::new(),
            text: None,
            children: Vec::new(),
        }
    }

    pub fn with_attr(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.attrs.insert(k.into(), v.into());
        self
    }

    pub fn with_text(mut self, t: impl Into<String>) -> Self {
        self.text = Some(t.into());
        self
    }

    pub fn with_child(mut self, child_id: u32) -> Self {
        self.children.push(child_id);
        self
    }
}

/// Viewport + scroll position at snapshot time.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub scroll_x: i64,
    pub scroll_y: i64,
}

/// A complete DOM snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomSnapshot {
    /// Wall-clock timestamp (ms since epoch).
    pub ts_ms: u64,
    pub viewport: Viewport,
    /// Nodes keyed by id — root is conventionally id 0.
    pub nodes: HashMap<u32, DomNode>,
    /// Id of the root node (None for an empty document).
    pub root_id: Option<u32>,
    /// Snapshot URL.
    pub url: Option<String>,
}

impl DomSnapshot {
    pub fn new(viewport: Viewport) -> Self {
        Self {
            ts_ms: 0,
            viewport,
            nodes: HashMap::new(),
            root_id: None,
            url: None,
        }
    }

    pub fn with_ts_ms(mut self, ts: u64) -> Self {
        self.ts_ms = ts;
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Insert a node, returning its id. First insert becomes the root
    /// if no root is set yet.
    pub fn insert(&mut self, node: DomNode) -> u32 {
        let id = node.id;
        if self.root_id.is_none() {
            self.root_id = Some(id);
        }
        self.nodes.insert(id, node);
        id
    }

    /// Root node, if any.
    pub fn root(&self) -> Option<&DomNode> {
        self.root_id.and_then(|id| self.nodes.get(&id))
    }

    /// Count nodes in the snapshot.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// True if the snapshot contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Walk the tree depth-first, yielding node ids root→leaf.
    pub fn dfs(&self) -> Vec<u32> {
        let mut out = Vec::with_capacity(self.nodes.len());
        if let Some(root) = self.root_id {
            self.dfs_visit(root, &mut out);
        }
        out
    }

    fn dfs_visit(&self, id: u32, out: &mut Vec<u32>) {
        out.push(id);
        if let Some(node) = self.nodes.get(&id) {
            for child in &node.children {
                self.dfs_visit(*child, out);
            }
        }
    }
}

/// A single structural diff operation between two snapshots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiffOp {
    /// A node exists in `new` but not `old`.
    Added { id: u32, tag: String },
    /// A node exists in `old` but not `new`.
    Removed { id: u32, tag: String },
    /// A node's attrs/text/children changed.
    Changed { id: u32, tag: String },
}

/// Structural diff between two snapshots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Diff {
    pub ops: Vec<DiffOp>,
}

impl Diff {
    /// Compute structural diff. Nodes match by id.
    pub fn between(old: &DomSnapshot, new: &DomSnapshot) -> Self {
        let mut ops = Vec::new();

        // Removed: in old, not in new.
        for (id, node) in &old.nodes {
            if !new.nodes.contains_key(id) {
                ops.push(DiffOp::Removed {
                    id: *id,
                    tag: node.tag.clone(),
                });
            }
        }

        // Added or Changed: in new.
        for (id, node) in &new.nodes {
            match old.nodes.get(id) {
                None => ops.push(DiffOp::Added {
                    id: *id,
                    tag: node.tag.clone(),
                }),
                Some(old_node) => {
                    if old_node != node {
                        ops.push(DiffOp::Changed {
                            id: *id,
                            tag: node.tag.clone(),
                        });
                    }
                }
            }
        }

        // Sort by id for determinism.
        ops.sort_by_key(|op| match op {
            DiffOp::Added { id, .. } => *id,
            DiffOp::Removed { id, .. } => *id,
            DiffOp::Changed { id, .. } => *id,
        });
        Self { ops }
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vp() -> Viewport {
        Viewport { width: 1280, height: 800, scroll_x: 0, scroll_y: 0 }
    }

    #[test]
    fn empty_snapshot_has_no_root() {
        let s = DomSnapshot::new(vp());
        assert!(s.root().is_none());
        assert!(s.is_empty());
    }

    #[test]
    fn first_insert_becomes_root() {
        let mut s = DomSnapshot::new(vp());
        s.insert(DomNode::new(0, "html"));
        assert_eq!(s.root().unwrap().tag, "html");
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn dfs_walks_root_to_leaf() {
        let mut s = DomSnapshot::new(vp());
        s.insert(DomNode::new(0, "html").with_child(1));
        s.insert(DomNode::new(1, "body").with_child(2));
        s.insert(DomNode::new(2, "p"));
        let order = s.dfs();
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn builder_chains() {
        let n = DomNode::new(5, "a")
            .with_attr("href", "/x")
            .with_text("click")
            .with_child(6);
        assert_eq!(n.id, 5);
        assert_eq!(n.tag, "a");
        assert_eq!(n.attrs.get("href").unwrap(), "/x");
        assert_eq!(n.text.as_deref(), Some("click"));
        assert_eq!(n.children, vec![6]);
    }

    #[test]
    fn diff_detects_added_node() {
        let mut old = DomSnapshot::new(vp());
        old.insert(DomNode::new(0, "html"));
        let mut new = DomSnapshot::new(vp());
        new.insert(DomNode::new(0, "html"));
        new.insert(DomNode::new(1, "body"));
        let d = Diff::between(&old, &new);
        assert_eq!(d.len(), 1);
        assert!(matches!(&d.ops[0], DiffOp::Added { id: 1, tag } if tag == "body"));
    }

    #[test]
    fn diff_detects_removed_node() {
        let mut old = DomSnapshot::new(vp());
        old.insert(DomNode::new(0, "html"));
        old.insert(DomNode::new(1, "body"));
        let mut new = DomSnapshot::new(vp());
        new.insert(DomNode::new(0, "html"));
        let d = Diff::between(&old, &new);
        assert_eq!(d.len(), 1);
        assert!(matches!(&d.ops[0], DiffOp::Removed { id: 1, tag } if tag == "body"));
    }

    #[test]
    fn diff_detects_changed_attrs() {
        let mut old = DomSnapshot::new(vp());
        old.insert(DomNode::new(0, "a").with_attr("href", "/old"));
        let mut new = DomSnapshot::new(vp());
        new.insert(DomNode::new(0, "a").with_attr("href", "/new"));
        let d = Diff::between(&old, &new);
        assert!(matches!(&d.ops[0], DiffOp::Changed { id: 0, tag } if tag == "a"));
    }

    #[test]
    fn diff_empty_when_identical() {
        let mut a = DomSnapshot::new(vp());
        a.insert(DomNode::new(0, "html"));
        let mut b = DomSnapshot::new(vp());
        b.insert(DomNode::new(0, "html"));
        assert!(Diff::between(&a, &b).is_empty());
    }

    #[test]
    fn serde_roundtrip() {
        let mut s = DomSnapshot::new(vp()).with_ts_ms(123).with_url("https://x");
        s.insert(DomNode::new(0, "html"));
        let json = serde_json::to_string(&s).unwrap();
        let back: DomSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
        assert_eq!(back.url.as_deref(), Some("https://x"));
    }

    #[test]
    fn diff_ops_sorted_by_id() {
        let mut old = DomSnapshot::new(vp());
        old.insert(DomNode::new(0, "a"));
        let mut new = DomSnapshot::new(vp());
        new.insert(DomNode::new(0, "a"));
        new.insert(DomNode::new(2, "c"));
        new.insert(DomNode::new(1, "b"));
        let d = Diff::between(&old, &new);
        let ids: Vec<u32> = d.ops.iter().map(|op| match op {
            DiffOp::Added { id, .. } => *id,
            _ => unreachable!(),
        }).collect();
        assert_eq!(ids, vec![1, 2]);
    }
}
