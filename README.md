# DNS resolver

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

# Rationale

I wanted to learn DNS and prefer to learn theory with practice. The [How DNS works zine](https://wizardzines.com/zines/dns) and the [Implement DNS in a weekend guide](https://implement-dns.wizardzines.com/) by Julia Evans allowed me to understand and play with the technology.

# Materials

Literature I used to learn about DNS:

- [Implement DNS in a weekend](https://implement-dns.wizardzines.com/) by Julia Evans
- [How DNS works zine](https://wizardzines.com/zines/dns) by Julia Evans
- [What is DNS?](https://www.cloudflare.com/learning/dns/what-is-dns/) by Cloudflare
- [RFC 1035: DOMAIN NAMES - IMPLEMENTATION AND SPECIFICATION](https://datatracker.ietf.org/doc/html/rfc1035)
