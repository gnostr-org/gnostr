# FAQs

## what protocol is used to interact with the git server(s) listed in the announcement event `30617`?

Most Git servers support a variety of protocols, and it is the user's responsibility to specify which one to use in the Git remote URL.

For a Nostr repository, the Git remote URL(s) are specified by the maintainer in the announcement event. These URLs may include a protocol that is not suitable for the user. For instance, if SSH is specified, the user may not have SSH keys configured for the Git server.

Privacy-conscious users often prefer to use unauthenticated HTTP for read operations and SSH for write operations.

The git-remote-nostr plugin is designed to increase the likelihood of success by disregarding the protocol specified by the maintainer and using the following approach. It attempts to use the following protocols for read operations:

- Unauthenticated HTTP
- SSH
- Authenticated HTTP

For write operations, it tries:

- SSH
- Unauthenticated HTTP
- Authenticated HTTP

If the first protocol attempted fails but a subsequent one succeeds, all future attempts will default to the successful protocol.

Some users may prefer a different configuration, such as using SSH for both read and write operations. They can achieve this by specifying ssh in the Nostr Git URL: `nostr://ssh/dan@gitworkshop.dev/ngit`.

Additionally, users may want to use non-default SSH keys. The user for non-default SSH keys can be specified in the Nostr Git URL: `nostr://nym1@ssh/npub123/identifier`. In this case, NIP05 addresses cannot be used.
