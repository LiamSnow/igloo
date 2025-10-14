use crate::glacier::{
    query::{Query, QueryFilter, QueryKind, QueryTarget},
    tree::GroupID,
};
use igloo_interface::{Component, ComponentType};
use smallvec::smallvec;

#[test]
fn test() {
    // let query = Query {
    //     filter: QueryFilter::With(ComponentType::Light),
    //     target: QueryTarget::Group(GroupID::from_parts(1, 2)),
    //     kind: QueryKind::Set(smallvec![Component::Int(60)]),
    // };

    // let ser = borsh::to_vec(&query);
}
