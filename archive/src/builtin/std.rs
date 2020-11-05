use core::{
    borrow::Borrow,
    ops::Deref,
};
use crate::{
    Archive,
    ArchiveRef,
    builtin::core::ArchivedSliceRef,
    default,
    Resolve,
    Write,
};

#[derive(Hash, Eq, PartialEq)]
#[repr(transparent)]
pub struct ArchivedString(<str as ArchiveRef>::Reference);

impl ArchivedString {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for ArchivedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Borrow<str> for ArchivedString {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl PartialEq<String> for ArchivedString {
    fn eq(&self, other: &String) -> bool {
        self.deref().eq(other.deref())
    }
}

impl PartialEq<ArchivedString> for String {
    fn eq(&self, other: &ArchivedString) -> bool {
        other.eq(self)
    }
}

pub struct StringResolver(<str as ArchiveRef>::Resolver);

impl Resolve<String> for StringResolver {
    type Archived = ArchivedString;

    fn resolve(self, pos: usize, value: &String) -> Self::Archived {
        ArchivedString(self.0.resolve(pos, value.as_str()))
    }
}

impl Archive for String {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    fn archive<W: Write + ?Sized>(&self, writer: &mut W) -> Result<Self::Resolver, W::Error> {
        Ok(StringResolver(self.as_str().archive_ref(writer)?))
    }
}

#[derive(Hash, Eq, PartialEq)]
#[repr(transparent)]
pub struct ArchivedBox<T>(T);

impl<T: Deref> Deref for ArchivedBox<T> {
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: Deref<Target = U>, U: PartialEq<V> + ?Sized, V: ?Sized> PartialEq<Box<V>> for ArchivedBox<T> {
    fn eq(&self, other: &Box<V>) -> bool {
        self.deref().eq(other.deref())
    }
}

pub struct BoxResolver<T>(T);

impl<T: ArchiveRef + ?Sized> Resolve<Box<T>> for BoxResolver<T::Resolver> {
    type Archived = ArchivedBox<T::Reference>;

    fn resolve(self, pos: usize, value: &Box<T>) -> Self::Archived {
        ArchivedBox(self.0.resolve(pos, value.as_ref()))
    }
}

impl<T: ArchiveRef + ?Sized> Archive for Box<T> {
    type Archived = ArchivedBox<T::Reference>;
    type Resolver = BoxResolver<T::Resolver>;

    fn archive<W: Write + ?Sized>(&self, writer: &mut W) -> Result<Self::Resolver, W::Error> {
        Ok(BoxResolver(self.as_ref().archive_ref(writer)?))
    }
}

fn slice_archive_ref<T: Archive, W: Write + ?Sized>(slice: &[T], writer: &mut W) -> Result<<[T] as ArchiveRef>::Resolver, W::Error> {
    let mut resolvers = Vec::with_capacity(slice.len());
    for i in 0..slice.len() {
        resolvers.push(slice[i].archive(writer)?);
    }
    let result = writer.align_for::<T::Archived>()?;
    unsafe {
        for (i, resolver) in resolvers.drain(..).enumerate() {
            writer.resolve_aligned(&slice[i], resolver)?;
        }
    }
    Ok(result)
}

impl<T: Archive> ArchiveRef for [T] {
    type Archived = [T::Archived];
    type Reference = ArchivedSliceRef<T::Archived>;
    type Resolver = usize;

    default! {
        fn archive_ref<W: Write + ?Sized>(&self, writer: &mut W) -> Result<Self::Resolver, W::Error> {
            slice_archive_ref(self, writer)
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct ArchivedVec<T>(T);

impl<T: Deref> Deref for ArchivedVec<T> {
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub struct VecResolver<T>(T);

impl<T: Resolve<[U]>, U> Resolve<Vec<U>> for VecResolver<T> {
    type Archived = ArchivedVec<T::Archived>;

    fn resolve(self, pos: usize, value: &Vec<U>) -> Self::Archived {
        ArchivedVec(self.0.resolve(pos, value.deref()))
    }
}

impl<T: Archive> Archive for Vec<T> {
    type Archived = ArchivedVec<<[T] as ArchiveRef>::Reference>;
    type Resolver = VecResolver<<[T] as ArchiveRef>::Resolver>;

    fn archive<W: Write + ?Sized>(&self, writer: &mut W) -> Result<Self::Resolver, W::Error> {
        Ok(VecResolver(self.as_slice().archive_ref(writer)?))
    }
}

impl<T: Deref<Target = [U]>, U: PartialEq<V>, V> PartialEq<Vec<V>> for ArchivedVec<T> {
    fn eq(&self, other: &Vec<V>) -> bool {
        self.deref().eq(other.deref())
    }
}

// TODO: impl Archive for HashMap/Set, etc