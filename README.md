# hyper-req-log

General-purpose request logger for applications using Hyper.

Logs entries with the following format:
`request: [action:status] user remote host method uri version agent referer elapsed`

as an example:
`request: [Forwarded:200] none 11.22.33.44:44894/55.66.77.88 my-domain.com HEAD /uptime-check HTTP/1.1 "Mozilla/5.0+(compatible; UptimeRobot/2.0; http://www.uptimerobot.com/)" https://my-domain.com/uptime-check 82.556µs`

The fields `action` and `user` are arbitrary and set per-request by the calling code. If `action` is not set, the first field will simply be the HTTP response status code and the colon is omitted.

The `remote` field is the remote address and port, and if an `X-Forwared-For` header is present, a slash and the contents of that header value as well.

The fields that come from HTTP headers, namely, `host`, `agent`, and `referer`, are printed as bare strings if they contain no spaces or unprintable characters, otherwise a double-quoted string where quotes and backslashes are backslash-escaped, and any non-UTF-8 data is given by `\xDD` escapes.

The `elapsed` field is the time between when the `LogRequest` instance was created and the time when it is written out to the log.