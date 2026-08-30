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
    pub fn matches(&self, node: &Node) -> bool {
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

//some convinience functions for identifying noteworthy types of element
fn is_element_of_type(node: &Node, tagname: &str) -> bool {
    //not sure if we need to check if it's an element since I beleive it can't have
    //a tag name if it's not an element, but I'd rather be certain it is an element than save
    //whatever (presumably miniscule) amount of time it takes to double check
    node.is_element() && node.has_tag_name(tagname)
}
fn is_computer(node: &Node) -> bool {
    is_element_of_type(node, "computer")
}
fn is_filesystem(node: &Node) -> bool {
    is_element_of_type(node, "filesystem")
}
fn is_folder(node: &Node) -> bool {
    is_element_of_type(node, "folder")
}
fn is_file(node: &Node) -> bool {
    is_element_of_type(node, "file")
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

    /// Get a reference to the root "folder" element of this Computer's filesystem element
    fn rootdir(&self) -> Option<Node<'a, 'input>> {
        //step 1: get the filesystem element
        let filesystem = self.node.children().find(is_filesystem)?;

        //step 2: get the root folder element; an element with tag name "folder" and "name" attribute "/"
        filesystem
            .children()
            .find(|n| is_folder(n) && n.attribute("name").is_some_and(|a| a == "/"))
    }

    /// Get a particular file on this Computer
    ///
    /// Returns the file element [Node] if a file matching the given `path` can be found
    /// on this Computer, or None if no file matching the given `path` can be found
    ///
    /// This function's treatment of paths may not exactly match treatment of paths
    /// by the actual game, or your operating system's treatment of paths:
    ///
    /// - The path is always treates as absolute - this can be thought of as
    ///   though a "/" is prepended to the path before looking for the file.
    /// - Forward slashes "/" and backslashes "\\" are treated identically
    /// - References to the current directory (".") and parent directory ("..")
    ///   are ignored entirely
    /// - Contiguous seperators (e.g. "//") are treated the same as a single seperator ("/")
    /// - Items are case-sensitive ("This_File_Name" is different from "this_file_name")
    // TODO: perhaps eliminate case-sensitivity? Autocompletion in-game isn't case sensitive; file paths themselves (when fed to commands like `cat`)
    // *are* case sensitive though
    pub fn file(&self, path: &str) -> Option<Node<'a, 'input>> {
        let mut split = path.split(&['/', '\\']);

        // getting the filename first means that our searching through the filesystem structure
        // only needs to pay attention to folders, meaning our strategy for looping through the folders
        // still works if one of the folders in our path has the same name as a file (which I believe can legitimately happen)
        let filename = split.next_back()?;

        // iterate through the path parts, finding a folder with a matching name for each part.
        let mut current_folder = self.rootdir()?;
        for pathpart in split {
            if pathpart.is_empty() || pathpart == "." || pathpart == ".." {
                continue;
            }

            current_folder = current_folder
                .children()
                .find(|f| is_folder(f) && f.attribute("name").is_some_and(|n| n == pathpart))?;
        }
        // now, current_folder is the folder that should contain our file
        // simply look for a file by the specified name in current_folder and return whatever we find
        current_folder
            .children()
            .find(|f| is_file(f) && f.attribute("name").is_some_and(|n| n == filename))
    }

    /// Get the content of a particular file on this Computer
    ///
    /// Convinience method for `Computer.file(path).and_then(|f| f.text())`; see [Computer::file] for
    /// more details.
    pub fn file_content(&self, path: &str) -> Option<&str> {
        self.file(path).and_then(|f| f.text())
    }
}

#[derive(Error, Debug)]
pub enum ReadFileFromSaveError {
    #[error("failed to parse the save file")]
    ParseError(#[from] roxmltree::Error),

    #[error("failed to find the target computer")]
    ComputerError(#[from] ComputerError),

    #[error("file with specified path not found on target computer")]
    FileNotFound
}

/// Read the content of a file that belongs to some computer within the specified save file and return it as a string
///
/// `save_file` is the string content of one of the game's profile-specific XML save files
/// `computer_strategy` is a [ComputerFindStrategy] defining how to locate the computer which the file is located on
/// `target_path` is the path to the file on the target computer
pub fn read_file_from_save(
    save_file: &str,
    computer_strategy: ComputerFindStrategy,
    target_path: &str
) -> Result<String, ReadFileFromSaveError> {
    let parsed = Document::parse(save_file)?;
    let computer = Computer::try_new(&parsed, computer_strategy)?;
    computer
        .file_content(target_path)
        .ok_or(ReadFileFromSaveError::FileNotFound)
        .map(|c| c.to_owned())
}
