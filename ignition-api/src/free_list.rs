use alloc::vec::Vec;
use core::convert::TryInto;
use core::fmt::Debug;
use core::mem::replace;

pub struct FreeList<I, T> {
    nodes: Vec<Node<I, T>>,
    first_free_index: Option<I>,
}

enum Node<I, T> {
    Free { next_free_index: Option<I> },
    Allocated(T),
}

impl<I, T> FreeList<I, T>
where
    I: Copy + Into<usize>,
    usize: TryInto<I>,
    <usize as TryInto<I>>::Error: Debug,
{
    pub fn insert(&mut self, value: T) -> I {
        if let Some(index) = self.first_free_index {
            // At least one free node exists.

            // Remove the free node at the head of the free list by replacing the free list with its
            // tail.
            let index_usize: usize = index.into();
            self.first_free_index = match self.nodes[index_usize] {
                Node::Free { next_free_index } => next_free_index,
                _ => panic!("bad free list"),
            };

            // Overwrite the allocated node.
            self.nodes[index_usize] = Node::Allocated(value);

            index
        } else {
            // There are no free nodes.

            // Add a new one.
            self.nodes.push(Node::Allocated(value));

            (self.nodes.len() - 1).try_into().unwrap()
        }
    }

    pub fn get(&self, index: I) -> &T {
        let index_usize: usize = index.into();
        match &self.nodes[index_usize] {
            Node::Allocated(value) => value,
            _ => panic!("get() called on a free node"),
        }
    }

    pub fn get_mut(&mut self, index: I) -> &mut T {
        let index_usize: usize = index.into();
        match &mut self.nodes[index_usize] {
            Node::Allocated(value) => value,
            _ => panic!("get_mut() called on a free node"),
        }
    }

    pub fn remove(&mut self, index: I) -> T {
        // Overwrite this node and push it onto the free list.
        let index_usize: usize = index.into();
        let old_node = replace(
            &mut self.nodes[index_usize],
            Node::Free {
                next_free_index: self.first_free_index,
            },
        );
        self.first_free_index = Some(index);

        // Return the old node.
        match old_node {
            Node::Allocated(value) => value,
            _ => panic!("remove() called on a free node"),
        }
    }
}

impl<I, T> Default for FreeList<I, T> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            first_free_index: Default::default(),
        }
    }
}
