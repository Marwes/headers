use super::{Error, Header, HeaderValue};
use http;

/// An extension trait adding "typed" methods to `http::HeaderMap`.
pub trait HeaderMapExt: self::sealed::Sealed {
    /// Inserts the typed `Header` into this `HeaderMap`.
    fn typed_insert<'value, H>(&'value mut self, header: H)
    where
        H: Header<'value>;

    /// Tries to find the header by name, and then decode it into `H`.
    fn typed_get<'value, H>(&'value self) -> Option<H>
    where
        H: Header<'value>;

    /// Tries to find the header by name, and then decode it into `H`.
    fn typed_try_get<'value, H>(&'value self) -> Result<Option<H>, Error>
    where
        H: Header<'value>;
}

impl HeaderMapExt for http::HeaderMap {
    fn typed_insert<'value, H>(&'value mut self, header: H)
    where
        H: Header<'value>,
    {
        let entry = self.entry(H::name()).expect("HeaderName is always valid");
        let mut values = ToValues {
            state: State::First(entry),
        };
        header.encode(&mut values);
    }

    fn typed_get<'value, H>(&'value self) -> Option<H>
    where
        H: Header<'value>,
    {
        HeaderMapExt::typed_try_get(self).unwrap_or(None)
    }

    fn typed_try_get<'value, H>(&'value self) -> Result<Option<H>, Error>
    where
        H: Header<'value>,
    {
        let mut values = self.get_all(H::name()).iter();
        if values.size_hint() == (0, Some(0)) {
            Ok(None)
        } else {
            H::decode(&mut values).map(Some)
        }
    }
}

struct ToValues<'a> {
    state: State<'a>,
}

#[derive(Debug)]
enum State<'a> {
    First(http::header::Entry<'a, HeaderValue>),
    Latter(http::header::OccupiedEntry<'a, HeaderValue>),
    Tmp,
}

impl<'a> Extend<HeaderValue> for ToValues<'a> {
    fn extend<T: IntoIterator<Item = HeaderValue>>(&mut self, iter: T) {
        for value in iter {
            let entry = match ::std::mem::replace(&mut self.state, State::Tmp) {
                State::First(http::header::Entry::Occupied(mut e)) => {
                    e.insert(value);
                    e
                }
                State::First(http::header::Entry::Vacant(e)) => e.insert_entry(value),
                State::Latter(mut e) => {
                    e.append(value);
                    e
                }
                State::Tmp => unreachable!("ToValues State::Tmp"),
            };
            self.state = State::Latter(entry);
        }
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for ::http::HeaderMap {}
}
