use std::fmt::{self, Display, Formatter, Write};
use std::io;
use std::net::SocketAddr;
use std::time::Instant;

use hyper::header::{HOST, REFERER, USER_AGENT};
use hyper::http::{HeaderValue, Method, Request, Uri, Version};
use hyper::Response;

use crate::escaped::Escaped;

/// [LogRequest] is a container for information about a HTTP request which
/// writes a log entry when dropped.
///
/// The `A` type parameter is the type of the `action` field, whose `Display`
/// representation is used when logging.
pub struct LogRequest<A: Display> {
    start_time: Instant,
    logged: bool,
    user: Option<String>,
    remote: Option<SocketAddr>,
    fwd: Option<HeaderValue>,
    host: Option<HeaderValue>,
    method: Method,
    uri: Uri,
    version: Version,
    user_agent: Option<HeaderValue>,
    referer: Option<HeaderValue>,
    action: Option<A>,
    status: Option<u16>,
}

impl<A: Display> LogRequest<A> {
    /// Create a new [LogRequest] instance from the given Hyper [Request].
    /// The request will be logged to stderr when the instance is dropped
    /// unless [write](Self::write) or [discard](Self::discard) are called
    /// first.
    pub fn from_request<B>(req: &Request<B>) -> Self {
        Self {
            start_time: Instant::now(),
            logged: false,
            user: None,
            remote: None,
            fwd: req.headers().get("x-forwarded-for").cloned(),
            host: req.headers().get(HOST).cloned(),
            method: req.method().to_owned(),
            uri: req.uri().to_owned(),
            version: req.version(),
            user_agent: req.headers().get(USER_AGENT).cloned(),
            referer: req.headers().get(REFERER).cloned(),
            action: None,
            status: None,
        }
    }

    /// Set the address of the remote endpoint.
    ///
    /// If a `X-Forwarded-For` header is present in the response, it will be
    /// appended to this value, following a colon.
    pub fn set_remote(&mut self, remote: SocketAddr) -> &mut Self {
        self.remote = Some(remote);
        self
    }

    /// Set a user identifier for the request. This can be any arbitrary
    /// string, and will be escaped if necessary.
    pub fn set_user(&mut self, user: String) -> &mut Self {
        self.user = Some(user);
        self
    }

    /// Set an action value for the request. This is intended to identify the
    /// part of the application which handled the request, and its Display
    /// representation is printed in the log.
    pub fn set_action(&mut self, action: A) -> &mut Self {
        self.action = Some(action);
        self
    }

    /// Take information from the response to the request.
    ///
    /// Currently only the HTTP status is extracted.
    pub fn set_response<B>(&mut self, response: &Response<B>) -> &mut Self {
        self.status = Some(response.status().as_u16());
        // TODO: response content length?
        self
    }

    /// Write the log entry to the given stream.
    pub fn write<W: io::Write>(mut self, write: W) -> io::Result<()> {
        self.logged = true;
        self.internal_write(write)
    }

    fn internal_write<W: io::Write>(&self, mut write: W) -> io::Result<()> {
        write!(write, "{self}")
    }

    /// Discard the instance without logging anything.
    pub fn discard(mut self) {
        self.logged = true;
    }
}

impl<A: Display> Display for LogRequest<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("request: [")?;
        if let Some(act) = &self.action {
            write!(f, "{act}:")?;
        }
        if let Some(status) = self.status {
            write!(f, "{status}")?;
        } else {
            f.write_str("???")?;
        }
        f.write_str("] ")?;
        if let Some(user) = &self.user {
            write!(f, "{} ", Escaped::from(user))?;
        }

        match self.remote {
            Some(SocketAddr::V4(v4)) => write!(f, "{v4}")?,
            Some(SocketAddr::V6(v6)) => {
                // TODO: use to_ipv4_mapped() once it's stable
                match v6.ip().octets() {
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, a, b, c, d] => {
                        write!(f, "{a}.{b}.{c}.{d}")?;
                    }
                    _ => write!(f, "{}", v6.ip())?,
                };
                write!(f, ":{}", v6.port())?;
            }
            None => f.write_str("<unknown-remote>")?,
        }
        if let Some(fwd) = &self.fwd {
            f.write_char('/')?;
            let mut fwd = fwd.as_bytes();
            fwd = fwd.strip_prefix(b"::ffff:").unwrap_or(fwd);
            write!(f, "{}", Escaped::from(fwd))?;
        }

        writeln!(
            f,
            " {host} {method} {uri} {version:?} {agent} {referer} {duration:?}",
            host = Escaped::from(self.host.as_ref()),
            method = self.method,
            uri = self.uri,
            version = self.version,
            agent = Escaped::from(self.user_agent.as_ref()),
            referer = Escaped::from(self.referer.as_ref()),
            duration = self.start_time.elapsed(),
        )?;

        Ok(())
    }
}

impl<A: Display> Drop for LogRequest<A> {
    fn drop(&mut self) {
        if !self.logged {
            let _ = self.internal_write(std::io::stderr().lock());
        }
    }
}
