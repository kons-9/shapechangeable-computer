use crate::utils::type_alias::Id;

pub trait Protocol {
    /// check whether this node is in route
    fn is_in_route(&self, this: Id, global_source: Id, global_destination: Id) -> bool;
    /// get next node
    fn get_next_node(&self, this: Id, global_destination: Id) -> Id;
}
