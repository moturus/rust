use crate::ffi::OsString;
use crate::fmt;
use crate::hash::{Hash, Hasher};
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::path::{Path, PathBuf};
use crate::sys::time::SystemTime;
use crate::sys::unsupported;

use super::map_moturus_error;

pub struct File {
    inner: moto_runtime::fs::File
}

pub struct FileAttr {
    inner: moto_runtime::fs::FileAttr
}

pub struct ReadDir {
    inner: moto_runtime::fs::ReadDir
}

pub struct DirEntry {
    inner: moto_runtime::fs::DirEntry
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    inner: moto_runtime::fs::OpenOptions
}

#[derive(Copy, Clone, Debug, Default)]
pub struct FileTimes {
    inner: moto_runtime::fs::FileTimes
}

pub struct FilePermissions {
    inner: moto_runtime::fs::FilePermissions
}

pub struct FileType {
    inner: moto_runtime::fs::FileType
}

impl FileAttr {
    pub fn size(&self) -> u64 {
        self.inner.size()
    }

    pub fn perm(&self) -> FilePermissions {
        FilePermissions{ inner: self.inner.perm() }
    }

    pub fn file_type(&self) -> FileType {
        FileType { inner: self.inner.file_type() }
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        self.inner.modified().map(|ts| ->
            SystemTime { SystemTime::from_unix_ts(ts) }).map_err(map_moturus_error)
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        self.inner.accessed().map(|ts| ->
            SystemTime { SystemTime::from_unix_ts(ts) }).map_err(map_moturus_error)
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        self.inner.created().map(|ts| ->
            SystemTime { SystemTime::from_unix_ts(ts) }).map_err(map_moturus_error)
    }
}

impl Clone for FileAttr {
    fn clone(&self) -> FileAttr {
        FileAttr { inner: self.inner.clone() }
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        self.inner.readonly()
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        self.inner.set_readonly(readonly)
    }
}

impl Clone for FilePermissions {
    fn clone(&self) -> FilePermissions {
        FilePermissions { inner: self.inner.clone() }
    }
}

impl PartialEq for FilePermissions {
    fn eq(&self, other: &FilePermissions) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl Eq for FilePermissions {}

impl fmt::Debug for FilePermissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl FileTimes {
    pub fn set_accessed(&mut self, t: SystemTime) {
        self.inner.set_accessed(t.as_unix_ts())
    }

    pub fn set_modified(&mut self, t: SystemTime) {
        self.inner.set_modified(t.as_unix_ts())
    }
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.inner.is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.inner.is_file()
    }

    pub fn is_symlink(&self) -> bool {
        self.inner.is_symlink()
    }
}

impl Clone for FileType {
    fn clone(&self) -> FileType {
        FileType { inner: self.inner.clone() }
    }
}

impl Copy for FileType {}

impl PartialEq for FileType {
    fn eq(&self, other: &FileType) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl Eq for FileType {}

impl Hash for FileType {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.inner.hash(h)
    }
}

impl fmt::Debug for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        let res = self.inner.next();
        if res.is_none() {
            return None;
        }

        let res = unsafe { res.unwrap_unchecked() };
        Some(res.map(|inner| -> DirEntry { DirEntry { inner: inner } }).
            map_err(map_moturus_error))
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.inner.path().to_owned().into()
    }

    pub fn file_name(&self) -> OsString {
        self.inner.file_name().to_owned().into()
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        self.inner.metadata().map(|inner| ->
            FileAttr { FileAttr{inner: inner} }).map_err(map_moturus_error)
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        self.inner.file_type().map(|inner| ->
            FileType { FileType{inner: inner} }).map_err(map_moturus_error)
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {inner: moto_runtime::fs::OpenOptions::new() }
    }

    pub fn read(&mut self, read: bool) {
        self.inner.read(read)
    }

    pub fn write(&mut self, write: bool) {
        self.inner.write(write)
    }

    pub fn append(&mut self, append: bool) {
        self.inner.append(append)
    }

    pub fn truncate(&mut self, truncate: bool) {
        self.inner.truncate(truncate)
    }

    pub fn create(&mut self, create: bool) {
        self.inner.create(create)
    }

    pub fn create_new(&mut self, create_new: bool) {
        self.inner.create_new(create_new)
    }
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        let path_str = path.to_str();
        if path_str.is_none() {
            return Err(io::Error::from(io::ErrorKind::InvalidFilename));
        }
        moto_runtime::fs::File::open(path_str.unwrap(), &opts.inner).
            map(|inner| { Self{inner} }).map_err(map_moturus_error)
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        self.inner.file_attr().map(|inner| ->
            FileAttr { FileAttr{inner: inner} }).map_err(map_moturus_error)
    }

    pub fn fsync(&self) -> io::Result<()> {
        self.inner.fsync().map_err(map_moturus_error)
    }

    pub fn datasync(&self) -> io::Result<()> {
        self.inner.datasync().map_err(map_moturus_error)
    }

    pub fn truncate(&self, size: u64) -> io::Result<()> {
        self.inner.truncate(size).map_err(map_moturus_error)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map_err(map_moturus_error)
    }

    pub fn read_vectored(&self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        unsupported()
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        crate::io::default_read_buf(|buf| self.read(buf), cursor)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf).map_err(map_moturus_error)
    }

    pub fn write_vectored(&self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        unsupported()
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let p = match pos {
            SeekFrom::Start(n) => moto_runtime::fs::SeekFrom::Start(n),
            SeekFrom::End(n) => moto_runtime::fs::SeekFrom::End(n),
            SeekFrom::Current(n) => moto_runtime::fs::SeekFrom::Current(n),
        };
        self.inner.seek(p).map_err(map_moturus_error)
    }

    pub fn duplicate(&self) -> io::Result<File> {
        unsupported()
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        unsupported()
    }

    pub fn set_times(&self, _times: FileTimes) -> io::Result<()> {
        unsupported()
    }
}

#[derive(Debug)]
pub struct DirBuilder {
    inner: moto_runtime::fs::DirBuilder
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {
            inner: moto_runtime::fs::DirBuilder::new()
        }
    }

    pub fn mkdir(&self, p: &Path) -> io::Result<()> {
        if let Some(pathname) = p.to_str() {
            self.inner.mkdir(pathname).map_err(map_moturus_error)
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
        }
    }
}

impl fmt::Debug for File {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(_f)
    }
}

pub fn readdir(path: &Path) -> io::Result<ReadDir> {
    if let Some(pathname) = path.to_str() {
        moto_runtime::fs::readdir(pathname).map(|inner| ->
            ReadDir { ReadDir{inner: inner} }).map_err(map_moturus_error)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
    }
}

pub fn unlink(p: &Path) -> io::Result<()> {
    if let Some(pathname) = p.to_str() {
        moto_runtime::fs::unlink(pathname).map_err(map_moturus_error)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
    }
}

pub fn rename(old: &Path, new: &Path) -> io::Result<()> {
    if let Some(old_path) = old.to_str() {
        if let Some(new_path) = new.to_str() {
            return moto_runtime::fs::rename(old_path, new_path).
                map_err(map_moturus_error);
        }
    }

    Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
}

pub fn set_perm(p: &Path, perm: FilePermissions) -> io::Result<()> {
    if let Some(pathname) = p.to_str() {
        moto_runtime::fs::set_perm(pathname, perm.inner).map_err(map_moturus_error)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
    }
}

pub fn rmdir(p: &Path) -> io::Result<()> {
    if let Some(pathname) = p.to_str() {
        moto_runtime::fs::rmdir(pathname).map_err(map_moturus_error)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
    }
}

pub fn remove_dir_all(p: &Path) -> io::Result<()> {
    if let Some(pathname) = p.to_str() {
        moto_runtime::fs::rmdir_all(pathname).map_err(map_moturus_error)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
    }
}

pub fn try_exists(_path: &Path) -> io::Result<bool> {
    unsupported()
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported()
}

pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    unsupported()
}

pub fn stat(p: &Path) -> io::Result<FileAttr> {
    if let Some(path) = p.to_str() {
        moto_runtime::fs::stat(path).map(|inner| ->
            FileAttr { FileAttr{inner: inner} }).map_err(map_moturus_error)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
    }
}

pub fn lstat(p: &Path) -> io::Result<FileAttr> {
    if let Some(path) = p.to_str() {
        moto_runtime::fs::lstat(path).map(|inner| ->
            FileAttr { FileAttr{inner: inner} }).map_err(map_moturus_error)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
    }
}

pub fn canonicalize(p: &Path) -> io::Result<PathBuf> {
    if let Some(path) = p.to_str() {
        moto_runtime::fs::canonicalize(path).map(|s| ->
            PathBuf { s.into() }).map_err(map_moturus_error)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidFilename, ""))
    }
}

pub fn copy(_from: &Path, _to: &Path) -> io::Result<u64> {
    unsupported()
}
