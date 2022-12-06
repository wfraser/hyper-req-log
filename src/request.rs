use std::fmt::{self, Display, Formatter, Write};
use std::io;
use std::net::SocketAddr;
use std::time::Instant;

use hyper::header::{HOST, REFERER, USER_AGENT};
use hyper::http::{HeaderValue, Method, Request, Uri, Version};
use hyper::Response;

use crate::escaped::Escaped;

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

    pub fn set_user(&mut self, user: String) -> &mut Self {
        self.user = Some(user);
        self
    }

    pub fn set_action(&mut self, action: A) -> &mut Self {
        self.action = Some(action);
        self
    }

    pub fn set_response<B>(&mut self, response: &Response<B>) -> &mut Self {
        self.status = Some(response.status().as_u16());
        // TODO: response content length?
        self
    }

    pub fn write<W: io::Write>(mut self, write: W) -> io::Result<()> {
        self.logged = true;
        self.internal_write(write)
    }

    fn internal_write<W: io::Write>(&self, mut write: W) -> io::Result<()> {
        write!(write, "{self}")
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
            f.write_str(user)?;
            f.write_char(' ')?;
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

        write!(
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
