//! RepresentationModel trait for hypermedia link management
//! Equivalent to Spring HATEOAS RepresentationModel

use crate::link::Link;
use crate::link::LinkRelation;

pub trait RepresentationModel {
    fn get_links(&self) -> &[Link];
    fn add_link(&mut self, link: Link);
    fn add_links(&mut self, links: impl IntoIterator<Item = Link>) {
        for link in links {
            self.add_link(link);
        }
    }

    fn has_link(&self, rel: &LinkRelation) -> bool {
        self.get_links().iter().any(|l| l.rel() == rel)
    }

    fn get_link(&self, rel: &LinkRelation) -> Option<&Link> {
        self.get_links().iter().find(|l| l.rel() == rel)
    }

    fn get_required_link(&self, rel: &LinkRelation) -> Option<&str> {
        self.get_link(rel).map(|l| l.href())
    }

    fn remove_links(&mut self, _rel: &LinkRelation) {
        // Default no-op; structs with mutable access to their link vec can override
    }
}
