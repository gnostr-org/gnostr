
#[cfg(test)]
mod opt_tests {
    use super::super::opt::{CliArgument, Opt};
    use clap::Parser;
    use std::path::PathBuf;
    use libp2p::Multiaddr;

    #[test]
    fn test_parse_minimal_provide() {
        let args = vec!["gnostr", "provide", "--path", "/tmp/file.txt", "--name", "my_file"];
        let opt = Opt::parse_from(args);

        match opt.argument {
            CliArgument::Provide { path, name } => {
                assert_eq!(path, PathBuf::from("/tmp/file.txt"));
                assert_eq!(name, "my_file");
            }
            _ => panic!("Expected Provide subcommand"),
        }
        assert!(opt.secret_key_seed.is_none());
        assert!(opt.peer.is_none());
        assert!(opt.listen_address.is_none());
    }

    #[test]
    fn test_parse_provide_with_options() {
        let args = vec![
            "gnostr",
            "--secret-key-seed", "10",
            "--peer", "/ip4/127.0.0.1/tcp/5000/p2p/12D3KooWH1URV3uTNQW6SZ1UFDnHN8NXwznAA8JeETTBm8oimjh9",
            "--listen-address", "/ip4/0.0.0.0/tcp/6000",
            "provide",
            "--path", "/tmp/another_file.dat",
            "--name", "another_file",
        ];
        let opt = Opt::parse_from(args);

        assert_eq!(opt.secret_key_seed, Some(10));
        let expected_peer: Multiaddr = "/ip4/127.0.0.1/tcp/5000/p2p/12D3KooWH1URV3uTNQW6SZ1UFDnHN8NXwznAA8JeETTBm8oimjh9".parse().unwrap();
        assert_eq!(opt.peer, Some(expected_peer));
        let expected_listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/6000".parse().unwrap();
        assert_eq!(opt.listen_address, Some(expected_listen_addr));

        match opt.argument {
            CliArgument::Provide { path, name } => {
                assert_eq!(path, PathBuf::from("/tmp/another_file.dat"));
                assert_eq!(name, "another_file");
            }
            _ => panic!("Expected Provide subcommand"),
        }
    }

    #[test]
    fn test_parse_minimal_get() {
        let args = vec!["gnostr", "get", "--name", "file_to_get"];
        let opt = Opt::parse_from(args);

        match opt.argument {
            CliArgument::Get { name } => {
                assert_eq!(name, "file_to_get");
            }
            _ => panic!("Expected Get subcommand"),
        }
        assert!(opt.secret_key_seed.is_none());
        assert!(opt.peer.is_none());
        assert!(opt.listen_address.is_none());
    }

    #[test]
    fn test_parse_get_with_options() {
        let args = vec![
            "gnostr",
            "--secret-key-seed", "20",
            "--listen-address", "/ip4/1.1.1.1/tcp/7000",
            "get",
            "--name", "important_data",
        ];
        let opt = Opt::parse_from(args);

        assert_eq!(opt.secret_key_seed, Some(20));
        assert!(opt.peer.is_none());
        let expected_listen_addr: Multiaddr = "/ip4/1.1.1.1/tcp/7000".parse().unwrap();
        assert_eq!(opt.listen_address, Some(expected_listen_addr));

        match opt.argument {
            CliArgument::Get { name } => {
                assert_eq!(name, "important_data");
            }
            _ => panic!("Expected Get subcommand"),
        }
    }

    #[test]
    fn test_parse_minimal_kv_get() {
        let args = vec!["gnostr", "kv", "--get", "my_key"];
        let opt = Opt::parse_from(args);

        match opt.argument {
            CliArgument::Kv { get } => {
                assert_eq!(get, Some("my_key".to_string()));
            }
            _ => panic!("Expected Kv subcommand"),
        }
        assert!(opt.secret_key_seed.is_none());
        assert!(opt.peer.is_none());
        assert!(opt.listen_address.is_none());
    }

    #[test]
    fn test_parse_kv_no_get() {
        let args = vec!["gnostr", "kv"];
        let opt = Opt::parse_from(args);

        match opt.argument {
            CliArgument::Kv { get } => {
                assert!(get.is_none());
            }
            _ => panic!("Expected Kv subcommand"),
        }
        assert!(opt.secret_key_seed.is_none());
        assert!(opt.peer.is_none());
        assert!(opt.listen_address.is_none());
    }

    #[test]
    fn test_parse_kv_with_options() {
        let args = vec![
            "gnostr",
            "--secret-key-seed", "30",
            "--peer", "/dnsaddr/bootstrap.libp2p.io",
            "kv",
            "--get", "config_key",
        ];
        let opt = Opt::parse_from(args);

        assert_eq!(opt.secret_key_seed, Some(30));
        let expected_peer: Multiaddr = "/dnsaddr/bootstrap.libp2p.io".parse().unwrap();
        assert_eq!(opt.peer, Some(expected_peer));
        assert!(opt.listen_address.is_none());

        match opt.argument {
            CliArgument::Kv { get } => {
                assert_eq!(get, Some("config_key".to_string()));
            }
            _ => panic!("Expected Kv subcommand"),
        }
    }
}
