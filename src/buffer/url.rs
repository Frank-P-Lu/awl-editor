//! Conservative URL enrollment shared by Insert Link prefill and smart paste.

/// Whether `s` is one plausible bare web/mail destination. This is deliberately
/// narrower than a general URL parser: only HTTP(S) with a real host and mailto
/// with a local part plus domain are enrolled. Everything else pastes literally.
pub fn is_url(s: &str) -> bool {
    if s.is_empty() || s.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((scheme, rest)) = s.split_once(':') else {
        return false;
    };
    if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") {
        let Some(rest) = rest.strip_prefix("//") else {
            return false;
        };
        let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
        return plausible_authority(authority);
    }
    if scheme.eq_ignore_ascii_case("mailto") {
        return plausible_mailbox(rest.split(['?', '#']).next().unwrap_or_default());
    }
    false
}

fn plausible_authority(authority: &str) -> bool {
    let host_port = match authority.split_once('@') {
        Some((userinfo, host)) if !userinfo.is_empty() && !host.contains('@') => host,
        Some(_) => return false,
        None => authority,
    };
    let host = if let Some(bracketed) = host_port.strip_prefix('[') {
        let Some(close) = bracketed.find(']') else {
            return false;
        };
        let tail = &bracketed[close + 1..];
        if !valid_port_tail(tail) {
            return false;
        }
        &bracketed[..close]
    } else {
        let (host, tail) = host_port
            .split_once(':')
            .map_or((host_port, ""), |(host, _port)| {
                (host, &host_port[host.len()..])
            });
        if !valid_port_tail(tail) {
            return false;
        }
        host
    };
    !host.is_empty() && host.chars().any(char::is_alphanumeric)
}

fn valid_port_tail(tail: &str) -> bool {
    tail.is_empty()
        || tail
            .strip_prefix(':')
            .is_some_and(|port| !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit()))
}

fn plausible_mailbox(address: &str) -> bool {
    let Some((local, domain)) = address.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && domain.chars().any(char::is_alphanumeric)
}
