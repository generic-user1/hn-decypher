//! Items related to the profile-specific XML save files from the game
use roxmltree::{Document, Node};
use thiserror::Error;

#[derive(Debug, Error)]
/// Reasons [Computer::try_new] may fail
pub enum ComputerError {
    #[error("computer node could not be found")]
    NotFound,

    #[error("node was found, but was not a computer element")]
    NotAComputer
}

/// Strategies for locating a [Computer] within an XML save file
#[derive(Debug, Clone)]
pub enum ComputerFindStrategy<'a, 'input, 'b> {
    /// Assert that a specific [Node] is both a computer element and that
    /// it's the specific computer element desired.
    ///
    /// [Computer::try_new] will check whether the Node is a computer element,
    /// but can't check whether it's the specific computer element desired.
    SpecificNode(Node<'a, 'input>),
    /// Locate the first encountered computer element with a matching IP address
    ByIp(&'b str),
    /// Locate the first encountered computer element with a matching name
    ByName(&'b str),
    /// Locate the first encountered computer element with a atching ID
    ById(&'b str)
}
impl<'a, 'input, 'b> ComputerFindStrategy<'a, 'input, 'b> {
    ///Determine whether the given `node` matches what we are looking for based on this strategy
    pub fn matches(&self, node: &Node<'a, 'input>) -> bool {
        if !is_computer(node) {
            false
        } else {
            match self {
                Self::SpecificNode(target) => node == target,
                Self::ByIp(ip) => node.attribute("ip").is_some_and(|c| c == *ip),
                Self::ByName(name) => node.attribute("name").is_some_and(|c| c == *name),
                Self::ById(id) => node.attribute("id").is_some_and(|c| c == *id)
            }
        }
    }
}

fn is_computer<'a, 'input>(node: &Node<'a, 'input>) -> bool {
    node.is_element() && node.tag_name().name() == "computer"
}

/// A `computer` node in an XML save file. Represents one in-game computer.
///
/// This is just a [Node] that is known to be an element with the
/// tag name "computer".
pub struct Computer<'a, 'input> {
    node: Node<'a, 'input>
}
impl<'a, 'input, 'b> Computer<'a, 'input> {
    /// Try to find a new Computer from the specified `save_file` using a particular `strategy`
    pub fn try_new(
        save_file: &'a Document<'input>,
        strategy: ComputerFindStrategy<'a, 'input, 'b>
    ) -> Result<Self, ComputerError> {
        match strategy {
            /*
            In theory, we could use the same process for SpecificNode
            as we do the other strategies: iterate through every node until we find one that matches.
            However, iterating through the document is only necessary for the other strategies
            because we don't know what Node we're looking for ahead of time - in this case, we do
            know what Node we're looking for, and can skip iteration
            */
            ComputerFindStrategy::SpecificNode(node) => {
                if is_computer(&node) {
                    Ok(Computer { node })
                } else {
                    Err(ComputerError::NotAComputer)
                }
            }
            strategy => save_file
                .descendants()
                .find_map(|n| {
                    if strategy.matches(&n) {
                        Some(Computer { node: n })
                    } else {
                        None
                    }
                })
                .ok_or(ComputerError::NotFound)
        }
    }

    /// Get a reference to the underlying [Node] backing this Computer
    pub fn inner(&self) -> &Node<'a, 'input> {
        &self.node
    }

    /// Take the underlying [Node] backing this Computer
    pub fn into_inner(self) -> Node<'a, 'input> {
        self.node
    }
}
