# Security Policy

## Reporting a vulnerability

Please do not report security vulnerabilities in a public issue. Use
[GitHub's private vulnerability reporting](https://github.com/tramuri/tramuri/security/advisories/new)
to contact the maintainers confidentially. Include the affected version or commit, impact, and
reproduction steps when possible. Avoid sending real credentials or unrelated personal data.

If private vulnerability reporting is unavailable, contact the repository maintainers privately
through GitHub and ask for a secure reporting channel. Do not disclose details publicly until the
maintainers have had a reasonable opportunity to investigate and coordinate a fix.

## Scope and response

The latest released version is the primary target for security fixes. The maintainers will assess
reports affecting supported versions and the current development branch. Tramuri is maintained by
a small team, so response and remediation times are not guaranteed; reporters will receive updates
as the investigation progresses.

## Security design

By default, Project Discovery does not execute files it scans and excludes sensitive paths. Evidence
references are stored without fetching remote resources or storing scanned file contents. Please
report any behavior that violates these expectations.
