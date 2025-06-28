use bytes::Bytes;
use http_body::{Body, Frame, SizeHint};
use http_body_util::{Empty, Full};
use std::pin::Pin;
use std::task::{Context, Poll};

pub enum HttpBody {
    Full(Full<Bytes>),
    Empty(Empty<Bytes>),
}

impl Body for HttpBody {
    type Data = Bytes;
    type Error = std::convert::Infallible; // Because Full and Empty never fail

    #[inline]
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            HttpBody::Full(b) => Pin::new(b).poll_frame(cx),
            HttpBody::Empty(b) => Pin::new(b).poll_frame(cx),
        }
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self {
            HttpBody::Full(b) => b.size_hint(),
            HttpBody::Empty(b) => b.size_hint(),
        }
    }
}
