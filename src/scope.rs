use std::any::TypeId;
use std::collections::HashMap;

pub trait Scope: crate::private::Sealed {
    const NAME: &'static str;
}

#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct Nil;

impl crate::private::Sealed for Nil {}

#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct Cons<H, T>(H, T);

impl<H, T> crate::private::Sealed for Cons<H, T> {}

#[doc(hidden)]
pub trait Length {
    const LEN: usize;
}

impl<H, T> Length for Cons<H, T>
where
    T: Length,
{
    const LEN: usize = 1 + T::LEN;
}

impl Length for Nil {
    const LEN: usize = 0;
}

pub trait Prepend<X>: crate::private::Sealed {
    type Result;
}

impl<X> Prepend<X> for Nil {
    type Result = Cons<X, Self>;
}

impl<X, H, T> Prepend<X> for Cons<H, T> {
    type Result = Cons<X, Self>;
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! right_brackets {
    ($f:ident; $($acc:tt)*) => {
        $($acc)* >
    };
    ($f:ident, $($m:ident),*; $($acc:tt)*) => {
        right_brackets!($($m),*; $($acc)* >)
    };
}

#[macro_export]
macro_rules! scopes {
    ($($x:ident),*) => {
        $crate::scope::ScopeList::<$crate::right_brackets!($($x),*; $($crate::scope::Cons<$x,)* $crate::scope::Nil)>
    };
}

pub trait Scoped<T>: crate::private::Sealed {}

#[derive(Debug, Copy, Clone)]
pub struct ScopeList<T> {
    list: std::marker::PhantomData<T>,
}

impl<T> crate::private::Sealed for ScopeList<T> where T: crate::private::Sealed {}

#[doc(hidden)]
pub trait AccumulateScopeName: crate::private::Sealed {
    const LEN: usize;
    fn accumulate_into(into: String) -> String;
}

impl<H, T> AccumulateScopeName for Cons<H, T>
where
    T: AccumulateScopeName,
    H: Scope,
{
    const LEN: usize = H::NAME.len() + T::LEN + 1;

    fn accumulate_into(mut into: String) -> String {
        into.push_str(H::NAME);
        into.push(' ');
        T::accumulate_into(into)
    }
}

impl AccumulateScopeName for Nil {
    const LEN: usize = 0;

    fn accumulate_into(into: String) -> String {
        into
    }
}

#[doc(hidden)]
pub trait AccumulateTypeMap<T> {
    fn accumulate_into(into: HashMap<TypeId, T>) -> HashMap<TypeId, T>;
}

impl<H, T, U> AccumulateTypeMap<U> for Cons<H, T>
where
    U: Default,
    T: AccumulateTypeMap<U>,
    H: 'static,
{
    fn accumulate_into(mut into: HashMap<TypeId, U>) -> HashMap<TypeId, U> {
        into.insert(TypeId::of::<H>(), U::default());
        T::accumulate_into(into)
    }
}

impl<T> AccumulateTypeMap<T> for Nil {
    fn accumulate_into(into: HashMap<TypeId, T>) -> HashMap<TypeId, T> {
        into
    }
}

pub trait ScopeListBehaviour: crate::private::Sealed {
    type TypeList;

    fn joined_names() -> Option<String>
    where
        Self::TypeList: AccumulateScopeName;

    fn type_map<T>() -> HashMap<TypeId, T>
    where
        Self::TypeList: AccumulateTypeMap<T> + Length;
}

impl<H, T> ScopeListBehaviour for ScopeList<Cons<H, T>> {
    type TypeList = Cons<H, T>;

    fn joined_names() -> Option<String>
    where
        Self::TypeList: AccumulateScopeName,
    {
        let mut names = Self::TypeList::accumulate_into(String::with_capacity(Self::TypeList::LEN));
        names.truncate(names.len() - 1);
        Some(names)
    }

    fn type_map<U>() -> HashMap<TypeId, U>
    where
        Self::TypeList: AccumulateTypeMap<U> + Length,
    {
        Self::TypeList::accumulate_into(HashMap::with_capacity(Self::TypeList::LEN))
    }
}

impl ScopeListBehaviour for ScopeList<Nil> {
    type TypeList = Nil;

    fn joined_names() -> Option<String>
    where
        Self::TypeList: AccumulateScopeName,
    {
        None
    }

    fn type_map<U>() -> HashMap<TypeId, U>
    where
        Self::TypeList: AccumulateTypeMap<U> + Length,
    {
        HashMap::new()
    }
}

impl ScopeList<Nil> {
    pub const fn empty() -> ScopeList<Nil> {
        Self {
            list: std::marker::PhantomData,
        }
    }
}

impl<C> ScopeList<C> {
    pub const fn create() -> ScopeList<C> {
        ScopeList {
            list: std::marker::PhantomData,
        }
    }

    #[allow(unused_variables)]
    pub fn add<S>(self, scope: S) -> ScopeList<C::Result>
    where
        C: Prepend<S>,
        S: Scope,
    {
        ScopeList {
            list: std::marker::PhantomData,
        }
    }
}

impl<S, T> Scoped<S> for ScopeList<T> where T: Scoped<S> {}

macro_rules! def_scopes {
    ($($t:ident),*) => {
        def_scopes![$($t,)*];
    };
    ($($t:ident,)*) => {
        $(
            #[derive(Debug, Copy, Clone)]
            pub struct $t;
        )*

        $(
            impl crate::private::Sealed for $t {}
        )*

        macro_rules! scoped {
            ($inner_t:ty) => {
                // Head impl
                impl<T> Scoped<$inner_t> for Cons<$inner_t, T> {}

                // Tail impls
                $(
                    impl<T> Scoped<$inner_t> for Cons<$t, T> where T: Scoped<$inner_t> {}
                )*
            };
        }

        $(
            scoped!($t);
        )*

        pub fn scope_type_id(name: &str) -> Option<TypeId> {
            match name {
                $(
                    $t::NAME => Some(TypeId::of::<$t>()),
                )*
                _ => None,
            }
        }

        fn _is_scope(_: impl Scope) {}

        fn _test() {
            $(
                _is_scope($t);
            )*
        }
    };
}

/*
 *  Images
 *      ugc-image-upload
 *  Library
 *      user-library-modify
 *      user-library-read
 *  Playback
 *      app-remote-control
 *      streaming
 *  Playlists
 *      playlist-read-private
 *      playlist-read-collaborative
 *      playlist-modify-public
 *      playlist-modify-private
 *  Follow
 *      user-follow-modify
 *      user-follow-read
 *  Listening History
 *      user-read-recently-played
 *      user-top-read
 *  Users
 *      user-read-private
 *      user-read-email
 *  Spotify Connect
 *      user-read-currently-playing
 *      user-read-playback-state
 *      user-modify-playback-state
 */

def_scopes![
    UgcImageUpload,
    UserLibraryModify,
    UserLibraryRead,
    AppRemoteControl,
    Streaming,
    PlaylistReadPrivate,
    PlaylistReadCollaborative,
    PlaylistModifyPublic,
    PlaylistModifyPrivate,
    UserFollowModify,
    UserFollowRead,
    UserReadRecentlyPlayed,
    UserTopRead,
    UserReadPrivate,
    UserReadEmail,
    UserReadCurrentlyPlaying,
    UserReadPlaybackState,
    UserModifyPlaybackState,
];

impl Scope for UgcImageUpload {
    const NAME: &'static str = "ugc-image-upload";
}

impl Scope for UserLibraryModify {
    const NAME: &'static str = "user-library-modify";
}

impl Scope for UserLibraryRead {
    const NAME: &'static str = "user-library-read";
}

impl Scope for AppRemoteControl {
    const NAME: &'static str = "app-remote-control";
}

impl Scope for Streaming {
    const NAME: &'static str = "streaming";
}

impl Scope for PlaylistReadPrivate {
    const NAME: &'static str = "playlist-read-private";
}

impl Scope for PlaylistReadCollaborative {
    const NAME: &'static str = "playlist-read-collaborative";
}

impl Scope for PlaylistModifyPublic {
    const NAME: &'static str = "playlist-modify-public";
}

impl Scope for PlaylistModifyPrivate {
    const NAME: &'static str = "playlist-modify-private";
}

impl Scope for UserFollowModify {
    const NAME: &'static str = "user-follow-modify";
}

impl Scope for UserFollowRead {
    const NAME: &'static str = "user-follow-read";
}

impl Scope for UserReadRecentlyPlayed {
    const NAME: &'static str = "user-read-recently-played";
}

impl Scope for UserTopRead {
    const NAME: &'static str = "user-top-read";
}

impl Scope for UserReadPrivate {
    const NAME: &'static str = "user-read-private";
}

impl Scope for UserReadEmail {
    const NAME: &'static str = "user-read-email";
}

impl Scope for UserReadCurrentlyPlaying {
    const NAME: &'static str = "user-read-currently-playing";
}

impl Scope for UserReadPlaybackState {
    const NAME: &'static str = "user-read-playback-state";
}

/// Write access to a user’s playback state
///
/// `user-msg` Control playback on your Spotify clients and Spotify Connect devices.
impl Scope for UserModifyPlaybackState {
    const NAME: &'static str = "user-modify-playback-state";
}

#[cfg(test)]
mod test {
    use super::*;

    const SCOPE: scopes![
        UserReadPrivate,
        UserReadPlaybackState,
        UserReadPlaybackState
    ] = ScopeList::create();

    #[test]
    fn test() {
        fn test(_: impl Scoped<UserReadPrivate>) {}
        fn scopes<S: ScopeListBehaviour>(_: S) -> Option<String>
        where
            S::TypeList: AccumulateScopeName,
        {
            S::joined_names()
        }

        test(<scopes![
            UserReadPrivate,
            UserReadPlaybackState,
            UserReadPlaybackState
        ]>::create());

        test(SCOPE);

        test(ScopeList::empty().add(UserReadPrivate).add(UserReadEmail));

        assert_eq!(
            Some(String::from("user-read-email user-read-private")),
            scopes(ScopeList::empty().add(UserReadPrivate).add(UserReadEmail))
        )
    }
}
