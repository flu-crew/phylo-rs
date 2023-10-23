pub mod simple_rtree;

use std::collections::{HashMap, HashSet};
use itertools::Itertools;

use crate::node::*;
use crate::tree::simple_rtree::*;
use crate::iter::{node_iter::*, edge_iter::*};

pub struct UnrootedPhyloTree{
    _nodes: HashMap<NodeID, NodeType>,
    _neighbours: HashMap<NodeID, HashSet<(Option<EdgeWeight>, NodeID)>>,
    _leaves: HashMap<NodeID, String>,
}

#[derive(Debug)]
pub struct RootedPhyloTree{
    root: NodeID,
    nodes: HashMap<NodeID, NodeType>,
    children: HashMap<NodeID, Vec<(NodeID, Option<EdgeWeight>)>>,
    parents: HashMap<NodeID, Option<NodeID>>,
}

impl RootedPhyloTree{
    pub fn new()->Self{
        RootedPhyloTree { 
            root: 0,
            nodes: HashMap::from([(0, NodeType::Internal(None))]),
            children: HashMap::from([(0, Vec::new())]),
            parents: HashMap::from([(0, None)]),
        }
    }

    pub fn from_newick(newick_string: String)->Self{
        let mut tree = RootedPhyloTree::new();
        let mut stack : Vec<NodeID> = Vec::new();
        let mut context : NodeID = tree.get_root().clone();
        let mut taxa_str = String::new();
        let mut decimal_str: String = String::new();
        let mut str_ptr: usize = 0;
        let newick_string = newick_string.chars().filter(|c| !c.is_whitespace()).collect::<Vec<char>>();
        while str_ptr<newick_string.len(){
            match newick_string[str_ptr]{
                '(' => {
                    stack.push(context);
                    context = tree.add_node();
                    str_ptr +=1;
                },
                ')'|',' => {
                    // last context id
                    let last_context = stack.last().expect("Newick string ended abruptly!");
                    // add current context as a child to last context
                    tree.set_child(
                        &context,
                        last_context,
                        decimal_str.parse::<EdgeWeight>().ok(),
                        match taxa_str.is_empty(){
                            true => None,
                            false => Some(taxa_str.to_string())
                        }
                    );
                    // we clear the strings
                    taxa_str.clear();
                    decimal_str.clear();

                    match newick_string[str_ptr] {
                        ',' => {
                            context = tree.add_node();
                            str_ptr += 1;
                        }
                        _ => {
                            context = stack.pop().expect("Newick string ended abruptly!");
                            str_ptr += 1;
                        }
                    }
                },
                ';'=>{
                    if !taxa_str.is_empty(){
                        tree.assign_taxa(&context, &taxa_str);
                    }
                    break;
                }
                ':' => {
                    // if the current context had a weight
                    if newick_string[str_ptr]==':'{
                        str_ptr+=1;
                        while newick_string[str_ptr].is_ascii_digit() || newick_string[str_ptr]=='.'{
                            decimal_str.push(newick_string[str_ptr]); 
                            str_ptr+=1;
                        }
                    }
                }
                _ => {
                    // push taxa characters into taxa string
                    while newick_string[str_ptr]!=':'&&newick_string[str_ptr]!=')'&&newick_string[str_ptr]!=','&&newick_string[str_ptr]!='('&&newick_string[str_ptr]!=';'{
                        taxa_str.push(newick_string[str_ptr]); 
                        str_ptr+=1;
                    }
                },
            }
        }
        let mut leaf_ids = Vec::new();
        tree.leaves_of_node(tree.get_root(), &mut leaf_ids);
        for leaf_id in leaf_ids{
            tree.set_leaf(&leaf_id);
        }
        return tree;
    }

    fn leaves_of_node(&self, node_id:&NodeID, leaves:&mut Vec<NodeID>){
        if self.get_node_children(node_id).is_empty(){
            leaves.push(*node_id);
        }

        for (child_node_id, _edge_weight) in self.get_node_children(node_id).iter(){
            self.leaves_of_node(child_node_id, leaves);
        }
    }
}

impl SimpleRTree for RootedPhyloTree{
    fn add_node(&mut self)->NodeID{
        // New node id
        let node_id = self.nodes.len();
        // add entry of node in parents and children fields
        self.nodes.insert(node_id.clone(), NodeType::Internal(None));
        self.parents.insert(node_id.clone(), None);
        self.children.insert(node_id.clone(), Vec::new());
        node_id
    }

    fn set_child(&mut self, node_id:&NodeID, parent_id:&NodeID, distance:Option<EdgeWeight>, taxa:Option<String>){
        self.parents.insert(node_id.clone(), Some(parent_id.clone()));
        self.children.entry(parent_id.clone()).or_default().push((node_id.clone(), distance));
        self.nodes.insert(node_id.clone(), NodeType::Internal(taxa));
    }

    fn set_leaf(&mut self, node_id: &NodeID) {
        self.nodes.entry(node_id.clone()).and_modify(|node| node.flip());
    }

    fn assign_taxa(&mut self,node:&NodeID, taxa:&str) {
        self.nodes.insert(node.clone(), NodeType::Internal(Some(taxa.to_string())));
    }

    fn set_edge_weight(&mut self, parent:&NodeID, child:&NodeID, edge_weight:Option<EdgeWeight>){
        self.children.entry(parent.clone())
            .and_modify(|children| *children = children.clone().iter()
                    .map(|(id, w)| {
                        match id==child{
                            true => {(id.clone(), edge_weight)},
                            false => {(id.clone(), w.clone())},
                        }
                    })
                    .collect()
        );
    }

    fn get_root(&self)->&NodeID{
        &self.root
    }

    fn get_node(&self, node_id: &NodeID)->&NodeType{
        self.nodes.get(node_id).expect("Invalid NodeID")
    }

    fn get_nodes(&self)->&HashMap<NodeID, NodeType>{
        &self.nodes
    }

    fn get_children(&self)->&HashMap<NodeID, Vec<(NodeID, Option<EdgeWeight>)>>{
        &self.children
    }

    fn get_parents(&self)->&HashMap<NodeID, Option<NodeID>>{
        &self.parents
    }


    fn get_node_children(&self, node_id: &NodeID)->&Vec<(NodeID, Option<EdgeWeight>)>{
        self.children.get(node_id).expect("Invalid NodeID!")
    }

    fn get_node_parent(&self, node_id:&NodeID)->Option<&NodeID>{
        self.parents.get(node_id).expect("Invalid NodeID!").as_ref()
    }

    fn get_leaves(&self, node_id: &NodeID)->Vec<(NodeID, NodeType)>{
        let mut leaf_vec: Vec<NodeID> = Vec::new();
        self.leaves_of_node(node_id, &mut leaf_vec);
        leaf_vec.into_iter().map(|leaf_id| (leaf_id, self.nodes.get(&leaf_id).cloned().expect("Invalid NodeID!"))).collect::<Vec<(NodeID, NodeType)>>()
    }

    fn get_subtree(&self, node_id: &NodeID)->Box<dyn SimpleRTree>{
        if self.is_leaf(node_id){
            panic!("NodeID is a leaf");
        }
        let root= node_id.clone();
        let mut nodes: HashMap<NodeID, NodeType>= HashMap::new();
        let mut children: HashMap<NodeID, Vec<(NodeID, Option<EdgeWeight>)>> = HashMap::new();
        let mut parents: HashMap<NodeID, Option<NodeID>> = HashMap::new();
        for decsendant_node_id in self.iter_node_pre(node_id){
            nodes.insert(decsendant_node_id.clone(), self.nodes.get(&decsendant_node_id).expect("Invalid NodeID!").clone());
            children.insert(decsendant_node_id.clone(), self.children.get(&decsendant_node_id).expect("Invalid NodeID!").clone());
            parents.insert(decsendant_node_id.clone(), self.parents.get(&decsendant_node_id).expect("Invalid NodeID!").clone());
        }
        Box::new(
            RootedPhyloTree{
                root: root,
                nodes: nodes,
                children: children,
                parents: parents,
            }
        )
    }

    fn get_mrca(&self, node_id_list: Vec<&NodeID>)->NodeID{
        let ancestor_iter_vec: Vec<std::vec::IntoIter<NodeID>> = node_id_list.iter().map(|x| self.get_ancestors_pre(x).into_iter()).collect();
        let mut mrca: NodeID = 0;
        for mut iterator in ancestor_iter_vec{
            let temp: HashSet<NodeID> = HashSet::new();
            match iterator.next(){
                Some(x) => {
                    match temp.contains(&x){
                        true => {mrca = x.clone()},
                        false => {
                            match temp.len()==0{
                                true => {},
                                false => {return mrca}
                            }
                        }
                    }
                },
                None => {}
            }
        }
        mrca
    }

    fn is_leaf(&self, node_id: &NodeID)->bool{
        self.nodes.get(node_id).expect("Invalid NodeID").is_leaf()
    }

    fn graft_subtree(&mut self, _tree: Box<dyn SimpleRTree>, _edge: (&NodeID, &NodeID)){
        todo!()
    }

    fn prune(&mut self, node_id: &NodeID)-> Box<dyn SimpleRTree>{
        let root= node_id.clone();
        let mut nodes: HashMap<NodeID, NodeType>= HashMap::new();
        let mut children: HashMap<NodeID, Vec<(NodeID, Option<EdgeWeight>)>> = HashMap::new();
        let mut parents: HashMap<NodeID, Option<NodeID>> = HashMap::new();
        for decsendant_node_id in self.iter_node_pre(node_id){
            nodes.insert(decsendant_node_id.clone(), self.nodes.remove(&decsendant_node_id).expect("Invalid NodeID!").clone());
            children.insert(decsendant_node_id.clone(), self.children.remove(&decsendant_node_id).expect("Invalid NodeID!").clone());
            parents.insert(decsendant_node_id.clone(), self.parents.remove(&decsendant_node_id).expect("Invalid NodeID!").clone());
            }
        Box::new(
            RootedPhyloTree{
                root: root,
                nodes: nodes,
                children: children,
                parents: parents,
            }
        )
    }

    fn iter_node_pre(&self, start_node_id: &NodeID)->PreOrdNodes{
        PreOrdNodes::new(start_node_id, &self.children)
    }

    fn iter_node_post(&self, start_node_id: &NodeID)->PostOrdNodes{
        PostOrdNodes::new(start_node_id, &self.children)
    }

    fn iter_edges_pre(&self, start_node_id: &NodeID)->PreOrdEdges{
        PreOrdEdges::new(&self, start_node_id)
    }

    fn iter_edges_post(&self, start_node_id: &NodeID)->PostOrdEdges{
        PostOrdEdges::new(&self, start_node_id)
    }

    fn get_ancestors_pre(&self, node_id: &NodeID)->Vec<NodeID>{
        let mut node_iter: Vec<NodeID> = Vec::new();
        let mut curr_node = node_id;
        while self.parents.get(curr_node) != None {
            match self.parents.get(curr_node).expect("Invalid NodeID!") {
                Some(node) => {
                    node_iter.push(node.clone());
                    curr_node = node;
                },
                None => {
                    node_iter.push(self.get_root().clone());
                    break;
                },
            }
        }
        node_iter
    }

    fn reroot_at_node(&mut self, _node_id: &NodeID){
        todo!()
    }

    fn reroot_at_edge(&mut self, _edge: (&NodeID, &NodeID)) {
        todo!()
    }

    fn split_edge(&mut self, edge: (NodeID, NodeID), edge_weights:(Option<EdgeWeight>, Option<EdgeWeight>)){
        let new_node_id = self.add_node();
        self.parents.insert(new_node_id, Some(edge.0));
        self.children.entry(new_node_id).or_default().push((edge.1, edge_weights.1));
        self.parents.insert(edge.1, Some(new_node_id));
    }

    fn distance_from_ancestor(&self, node: &NodeID, ancestor: &NodeID, weighted: bool)->f64{
        let binding = self.get_ancestors_pre(node);
        let start_idx = binding.iter().position(|&x| x==*ancestor).expect("Provided ancestor is not an ancestor of node!");
        let mut node_ancestor_pre = binding[start_idx..].iter();
        let mut curr_parent = node_ancestor_pre.next().unwrap();
        let mut distance  = 0 as f64;
        loop{
            match node_ancestor_pre.next(){
                Some(node_id) => {
                    let curr_parent_children = self.get_node_children(curr_parent);
                    for (child_id, w) in curr_parent_children{
                        if child_id==node_id{
                            match weighted {
                                true => {distance += w.unwrap_or(0 as f64);}
                                false => {distance += 1 as f64;}
                            }
                            curr_parent = node_id;
                            continue;
                        }
                        panic!("Ancestor chain is broken! Clean tree before moving forward...")
                    } 
                },
                None => {break;}
            } 
        };
        distance
    }

    fn get_bipartition(&self, edge: (&NodeID, &NodeID))->(Vec<(NodeID, NodeType)>, Vec<(NodeID, NodeType)>){
        let c2 = self.get_cluster(edge.1);
        (self.nodes.clone().into_iter().filter(|x| !c2.contains(x)).collect_vec(), c2)
    }

    fn get_cluster(&self, node_id: &NodeID)-> Vec<(NodeID, NodeType)>{
        let mut leaves: Vec<NodeID> = Vec::new();
        self.leaves_of_node(node_id, &mut leaves);
        leaves.into_iter().map(|leaf_id| (leaf_id, self.get_node(&leaf_id).clone())).collect_vec()
    }

    fn clean(&mut self) {
        let mut remove_list: Vec<&NodeID> = Vec::new();
        for (node_id, node) in self.nodes.clone().iter(){
            // remove root with only one child
            if node_id==self.get_root() && self.get_node_degree(node_id)<2{
                let new_root = self.get_node_children(self.get_root())[0].0;
                self.root = new_root;
                self.parents.entry(new_root).and_modify(|x| *x = None);
                remove_list.push(node_id);
            }
            // remove nodes with only one child
            else if !node.is_leaf() &&  self.get_node_degree(node_id)<3{
                let parent = self.get_node_parent(node_id).cloned();
                let children = self.get_node_children(node_id).clone();
                for (child_id, _edge_weight) in children.clone().into_iter(){
                    self.parents.entry(child_id.clone()).and_modify(|x| *x = parent);
                }
                self.set_children(parent.as_ref().unwrap(), &children);
            }
        }
    }

    fn get_taxa(&self, node_id:&NodeID)->String {
        self.get_node(node_id).taxa()
    }

    fn incerement_ids(&mut self, value: &usize){
        self.nodes = self.nodes.clone().into_iter().map(|(node_id, node_type)| (node_id+value, node_type)).collect();
        self.parents = self.parents.clone().into_iter().map(|(node_id, parent_id)| {
            (
                node_id+value, 
                match parent_id{
                    Some(id) => Some(id+value),
                    None => None,
                }
            )
        }).collect();
        self.children = self.children.clone().into_iter().map(|(node_id, children_vec)| {
            (
                node_id+value,
                children_vec.into_iter().map(|(child_id, w)| {
                    (
                        child_id+value,
                        w
                    )
                })
                .collect()
            )
        }).collect();
    }

}