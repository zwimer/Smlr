//! Define a mock filesystem for more fine-grained control over unit tests

use std::{fs, io, time};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

mod real_fs;
pub use self::real_fs::RealFileSystem;

mod test_fs;
pub use self::test_fs::{TestFile, TestFileSystem, TestMD};

use hash::FileHash;

//definition of traits
//RUST NOTE: the "trait foo: baz" denotes that foo reuires that
// any object it is implemented on also implements baz.
// this allows the default implementation of methods to
// employ the methods of baz

/// The VFS [virtual file system] trait is the interface we require
/// for the injection into the directectory walker.
pub trait VFS: Clone + Debug {
    type FileIter: File;
    /// Recursively enumerate all files beneath a given path
    fn list_dir<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<Box<Iterator<Item = io::Result<Self::FileIter>>>>;

    /// Get the metadata of a file (follows symlinks)
    fn get_metadata<P: AsRef<Path>>(&self, p: P) -> io::Result<<Self::FileIter as File>::MD>;

    /// Get the metadata of a file (doesn't follow symlinks)
    fn get_symlink_metadata<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<<Self::FileIter as File>::MD>;

    /// Resolve a link path to the path of its target
    fn read_link<P: AsRef<Path>>(&self, p: P) -> io::Result<PathBuf>;

    // must be of type "File" (not a dir/link/other)
    /// Get a File handle from a path
    fn get_file(&self, p: &Path) -> io::Result<Self::FileIter>;

    // must be of type "File" (not a dir/link/other)
    /// Delete a file
    fn rm_file<P: AsRef<Path>>(&mut self, p: &P) -> io::Result<()>;

    // create
    /// Create a haard link at `src` pointing to what's at `dst`
    fn make_link(&mut self, src: &Path, dst: &Path) -> io::Result<()>;
}

/// Define common interface for a real or mock file
pub trait File: Debug {
    type MD: MetaData;
    fn get_inode(&self) -> io::Result<Inode>;
    fn get_path(&self) -> PathBuf;
    fn get_type(&self) -> io::Result<FileType>;
    fn get_metadata(&self) -> io::Result<Self::MD>;
    /// Read and hash first K bytes of the file
    fn get_first_bytes<H: FileHash>(&self) -> io::Result<<H as FileHash>::Output>;
    /// Hash the contents of the file
    fn get_hash<H: FileHash>(&self) -> io::Result<<H as FileHash>::Output>;
}

// it is the subset of the interface of fs::MetaData that we use
/// Define common interface for real of mock metadata object
pub trait MetaData: Debug {
    fn get_len(&self) -> u64;
    fn get_mod_time(&self) -> io::Result<time::SystemTime>;
    fn get_type(&self) -> FileType;
    fn get_inode(&self) -> Inode;
    fn get_device(&self) -> io::Result<DeviceId>;
}


// helper types
//RUST NOTE: rust enums can be defined over types such that
//a variable of the the enum type can be of any of the included types.

/// `Filetype` is an enum of all types used for filesystem objects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    File,
    Dir,
    Symlink,
    Other,
}

/// Implementation of creation method for the `FileType` enum.
/// maps creation (from) method over the constitute types of `FileType`
impl From<fs::FileType> for FileType {
    fn from(ft: fs::FileType) -> FileType {
        if ft.is_file() {
            FileType::File
        } else if ft.is_dir() {
            FileType::Dir
        } else if ft.is_symlink() {
            FileType::Symlink
        } else {
            // for other filesystem objets. might be block/char device, fifo,
            // socket, etc depending on os
            FileType::Other
        }
    }
}
//RUST NOTE: the #[derive(...)] automatically adds the traits indicated in derive
// one should also note that Clone, Copy, Hash, PartialEQ, and EQ are part of the rust
// std and do pretty much what it they say.
/// Inode is wraper around a 'long' with several added traits (interface)
/// which represents the inode of a file
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Inode(pub u64);

/// `DeviceId` is a wraper around a 'long' with several traits
/// represents a device id.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceId(pub u64);
