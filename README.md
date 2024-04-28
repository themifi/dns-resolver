# DNS resolver
[![Check and Lint](https://github.com/themifi/dns-resolver/actions/workflows/check-and-lint.yaml/badge.svg)](https://github.com/themifi/dns-resolver/actions/workflows/check-and-lint.yaml)
[![Test with Code Coverage](https://github.com/themifi/dns-resolver/actions/workflows/test.yaml/badge.svg)](https://github.com/themifi/dns-resolver/actions/workflows/test.yaml)
[![codecov](https://codecov.io/gh/themifi/dns-resolver/graph/badge.svg?token=2KTR62Z4CD)](https://codecov.io/gh/themifi/dns-resolver)

A toy DNS resolver. It translates domain names to IP addresses.

# Usage

Just pass a domain name as an argument.

```
cargo run <DOMAIN_NAME>
# or
./dns-resolver <DOMAIN_NAME>
```

Example:

```
./dns-resolver example.com
IP address: 93.184.215.14
```

# Features

- Recoursive resolve domain names starting from the root nameserver.
```
cargo run twitter.com
Querying 198.41.0.4 for twitter.com
Querying 192.41.162.30 for twitter.com
Querying 198.41.0.4 for a.r06.twtrdns.net
Querying 192.55.83.30 for a.r06.twtrdns.net
Querying 205.251.195.207 for a.r06.twtrdns.net
Querying 205.251.192.179 for twitter.com
IP address: 104.244.42.129
```

# Rationale

I wanted to learn DNS and prefer to learn theory with practice. The [How DNS works zine](https://wizardzines.com/zines/dns) and the [Implement DNS in a weekend guide](https://implement-dns.wizardzines.com/) by Julia Evans allowed me to understand and play with the technology.

# Materials

Literature I used to learn about DNS:

- [Implement DNS in a weekend](https://implement-dns.wizardzines.com/) by Julia Evans
- [How DNS works zine](https://wizardzines.com/zines/dns) by Julia Evans
- [What is DNS?](https://www.cloudflare.com/learning/dns/what-is-dns/) by Cloudflare
- [RFC 1035: DOMAIN NAMES - IMPLEMENTATION AND SPECIFICATION](https://datatracker.ietf.org/doc/html/rfc1035)
