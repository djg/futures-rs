use futures_core::{Poll, Async, Stream};
use futures_core::task;
use futures_sink::{Sink};

/// A combinator used to flatten a stream-of-streams into one long stream of
/// elements.
///
/// This combinator is created by the `Stream::flatten` method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Flatten<S>
    where S: Stream,
{
    stream: S,
    next: Option<S::Item>,
}

pub fn new<S>(s: S) -> Flatten<S>
    where S: Stream,
          S::Item: Stream<Error = S::Error>,
{
    Flatten {
        stream: s,
        next: None,
    }
}

impl<S: Stream> Flatten<S> {
    /// Acquires a reference to the underlying stream that this combinator is
    /// pulling from.
    pub fn get_ref(&self) -> &S {
        &self.stream
    }

    /// Acquires a mutable reference to the underlying stream that this
    /// combinator is pulling from.
    ///
    /// Note that care must be taken to avoid tampering with the state of the
    /// stream which may otherwise confuse this combinator.
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    /// Consumes this combinator, returning the underlying stream.
    ///
    /// Note that this may discard intermediate state of this combinator, so
    /// care should be taken to avoid losing resources when this is called.
    pub fn into_inner(self) -> S {
        self.stream
    }
}

// Forwarding impl of Sink from the underlying stream
impl<S> Sink for Flatten<S>
    where S: Sink + Stream
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;

    delegate_sink!(stream);
}

impl<S> Stream for Flatten<S>
    where S: Stream,
          S::Item: Stream,
          <S::Item as Stream>::Error: From<S::Error>,
{
    type Item = <S::Item as Stream>::Item;
    type Error = <S::Item as Stream>::Error;

    fn poll_next(&mut self, cx: &mut task::Context) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if self.next.is_none() {
                match try_ready!(self.stream.poll_next(cx)) {
                    Some(e) => self.next = Some(e),
                    None => return Ok(Async::Ready(None)),
                }
            }
            assert!(self.next.is_some());
            match self.next.as_mut().unwrap().poll_next(cx) {
                Ok(Async::Ready(None)) => self.next = None,
                other => return other,
            }
        }
    }
}
