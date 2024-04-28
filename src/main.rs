use dns_resolver::resolve_domain;

fn main() {
    let mut args = std::env::args();
    if args.len() < 2 {
        eprintln!("Usage: {} <DOMAIN_NAME>", args.next().unwrap());
        std::process::exit(1);
    }

    let domain = args.nth(1).unwrap();
    let ip = resolve_domain(domain);
    println!("IP address: {}", ip);
}
