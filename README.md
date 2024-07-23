# Phylo

Phylo a fast, extensible, general-purpose, and WebAssembly-capable library for phylogenetic analysis and inference written in Rust. Phylo-rs leverages a combination of memory safety, speed, and native WebAssembly support offered by Rust to provide a robust set of memory-efficient data structures with basic algorithms ranging from tree manipulation with SPR to computing tree statistics such as phylogenetic diversity.

# A note on implementation

Implementation of tree-like structures in rust can be difficult and time-intensive. Additionally implementing 
tree traversals and operations on tree structures (recursive or otherwise) can be an even bigger task. 

This crate aims to implement a majority of such methods as easily-derivable traits, so you don't have to implement them from scratch where they are not needed.

**We also provide a struct so you don't have to implement one...**  

# Using `phylo`
Most of the functionality is implemented in [`crate::tree::simple_rtree`]. The
[`crate::tree::ops`] module is used to dealt with phylolgenetic analysis that require tree mutations such as SPR, NNI, etc.
[`crate::tree::simulation`] module is used to simulate random trees
[`crate::tree::io`] module is used to read trees from various encodings
[`crate::tree::distances`] module is used to compute various types of distance between nodes in a tree and between trees
[`crate::iter`] is a helper module to provide tree traversals and iterations.

## Building trees
The simplest way to build a tree is to create an empty tree, add a root node and
then add children to the various added nodes:

```rust
use phylo::prelude::*;

let mut tree = SimpleRootedTree::new(1);

let new_node = Node::new(2);
tree.add_child(tree.get_root_id(), new_node);
let new_node = Node::new(3);
tree.add_child(tree.get_root_id(), new_node);
let new_node: Node = Node::new(4);
tree.add_child(2, new_node);
let new_node: Node = Node::new(5);
tree.add_child(2, new_node);
```

## Reading and writing trees
This library can build trees strings (or files) encoded in the
[newick](https://en.wikipedia.org/wiki/Newick_format) format:
```
use phylo::prelude::*;

let input_str = String::from("((A:0.1,B:0.2),C:0.6);");
let tree = SimpleRootedTree::from_newick(input_str.as_bytes())?;
```

## Traversing trees
Several traversals are implemented to visit nodes in a particular order. pre-order,
post-order. A traversals returns an [`Iterator`] of either nodes or NodeID's
in the order they are to be visited in.
```rust
use phylo::prelude::*;

let input_str = String::from("((A:0.1,B:0.2),C:0.6);");
let tree = SimpleRootedTree::from_newick(input_str.as_bytes())?;

let dfs_traversal = tree.dfs(tree.get_root_id()).into_iter();
let bfs_traversal = tree.bfs_ids(tree.get_root_id());
let postfix_traversal = tree.postord_ids(tree.get_root_id());
```


## Comparing trees
A number of metrics taking into account topology and branch lenghts are implemented
in order to compare trees with each other:
```rust
use phylo::prelude::*;

fn depth(tree: &SimpleRootedTree, node_id: usize) -> f32 {
    tree.depth(node_id) as f32
}

let newick_1 = "((A:0.1,B:0.2):0.6,(C:0.3,D:0.4):0.5);";
let newick_2 = "((D:0.3,C:0.4):0.5,(B:0.2,A:0.1):0.6);";

let tree_1 = SimpleRootedTree::from_newick(newick_1.as_bytes())?;
let tree_2 = SimpleRootedTree::from_newick(newick_2.as_bytes())?;

tree_1.precompute_constant_time_lca();
tree_2.precompute_constant_time_lca();

tree_1.set_zeta(depth);
tree_2.set_zeta(depth);


let rfs = tree_1.rfs(&tree_2);
let wrfs = tree_1.wrfs(&tree_2);
let ca = tree_1.ca(&tree_2);
let cophen = tree_1.cophen_dist_naive(&tree_2, 2);
```
