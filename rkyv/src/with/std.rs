use crate::{
    string::{ArchivedString, StringResolver},
    with::{
        ArchiveWith, AsString, AsStringError, DeserializeWith, Immutable, Lock, LockError,
        SerializeWith,
    },
    Archive, Deserialize, Fallible, Serialize, SerializeUnsized,
};
use core::str::FromStr;
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{Mutex, RwLock},
};

impl ArchiveWith<OsString> for AsString {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    #[inline]
    unsafe fn resolve_with(
        field: &OsString,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        // It's safe to unwrap here because if the OsString wasn't valid UTF-8 it would have failed
        // to serialize
        ArchivedString::resolve_from_str(field.to_str().unwrap(), pos, resolver, out);
    }
}

impl<S: Fallible + ?Sized> SerializeWith<OsString, S> for AsString
where
    S::Error: From<AsStringError>,
    str: SerializeUnsized<S>,
{
    #[inline]
    fn serialize_with(field: &OsString, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        ArchivedString::serialize_from_str(
            field.to_str().ok_or(AsStringError::InvalidUTF8)?,
            serializer,
        )
    }
}

impl<D: Fallible + ?Sized> DeserializeWith<ArchivedString, OsString, D> for AsString {
    #[inline]
    fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<OsString, D::Error> {
        Ok(OsString::from_str(field.as_str()).unwrap())
    }
}

impl ArchiveWith<PathBuf> for AsString {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    #[inline]
    unsafe fn resolve_with(
        field: &PathBuf,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        // It's safe to unwrap here because if the OsString wasn't valid UTF-8 it would have failed
        // to serialize
        ArchivedString::resolve_from_str(field.to_str().unwrap(), pos, resolver, out);
    }
}

impl<S: Fallible + ?Sized> SerializeWith<PathBuf, S> for AsString
where
    S::Error: From<AsStringError>,
    str: SerializeUnsized<S>,
{
    #[inline]
    fn serialize_with(field: &PathBuf, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        ArchivedString::serialize_from_str(
            field.to_str().ok_or(AsStringError::InvalidUTF8)?,
            serializer,
        )
    }
}

impl<D: Fallible + ?Sized> DeserializeWith<ArchivedString, PathBuf, D> for AsString {
    #[inline]
    fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<PathBuf, D::Error> {
        Ok(PathBuf::from_str(field.as_str()).unwrap())
    }
}

impl<F: Archive> ArchiveWith<Mutex<F>> for Lock {
    type Archived = Immutable<F::Archived>;
    type Resolver = F::Resolver;

    #[inline]
    unsafe fn resolve_with(
        field: &Mutex<F>,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        // Unfortunately, we have to unwrap here because resolve must be infallible
        //
        // An alternative would be to only implement ArchiveWith for Arc<Mutex<F>>, copy an Arc into
        // the resolver, and hold the guard in there as well (as a reference to the internal mutex).
        // This unfortunately will cause a deadlock if two Arcs to the same Mutex are serialized
        // before the first is resolved. The compromise is, unfortunately, to just unwrap poison
        // errors here and document it.
        field.lock().unwrap().resolve(pos, resolver, out.cast());
    }
}

impl<F: Serialize<S>, S: Fallible + ?Sized> SerializeWith<Mutex<F>, S> for Lock
where
    S::Error: From<LockError>,
{
    #[inline]
    fn serialize_with(field: &Mutex<F>, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        field
            .lock()
            .map_err(|_| LockError::Poisoned)?
            .serialize(serializer)
    }
}

impl<F, T, D> DeserializeWith<Immutable<F>, Mutex<T>, D> for Lock
where
    F: Deserialize<T, D>,
    D: Fallible + ?Sized,
{
    #[inline]
    fn deserialize_with(field: &Immutable<F>, deserializer: &mut D) -> Result<Mutex<T>, D::Error> {
        Ok(Mutex::new(field.value().deserialize(deserializer)?))
    }
}

impl<F: Archive> ArchiveWith<RwLock<F>> for Lock {
    type Archived = Immutable<F::Archived>;
    type Resolver = F::Resolver;

    #[inline]
    unsafe fn resolve_with(
        field: &RwLock<F>,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        // Unfortunately, we have to unwrap here because resolve must be infallible
        //
        // An alternative would be to only implement ArchiveWith for Arc<Mutex<F>>, copy a an Arc
        // into the resolver, and hold the guard in there as well (as a reference to the internal
        // mutex). This unfortunately will cause a deadlock if two Arcs to the same Mutex are
        // serialized before the first is resolved. The compromise is, unfortunately, to just
        // unwrap poison errors here and document it.
        field.read().unwrap().resolve(pos, resolver, out.cast());
    }
}

impl<F: Serialize<S>, S: Fallible + ?Sized> SerializeWith<RwLock<F>, S> for Lock
where
    S::Error: From<LockError>,
{
    #[inline]
    fn serialize_with(field: &RwLock<F>, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        field
            .read()
            .map_err(|_| LockError::Poisoned)?
            .serialize(serializer)
    }
}

impl<F, T, D> DeserializeWith<Immutable<F>, RwLock<T>, D> for Lock
where
    F: Deserialize<T, D>,
    D: Fallible + ?Sized,
{
    #[inline]
    fn deserialize_with(field: &Immutable<F>, deserializer: &mut D) -> Result<RwLock<T>, D::Error> {
        Ok(RwLock::new(field.value().deserialize(deserializer)?))
    }
}
