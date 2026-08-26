# test-fluent-drivers

> DON'T use this repo! This is a template repo used for testing various fluent drivers' compatibility with swift-libp2p

### How to use
1. Clone swift-libp2p if you havent already

2. Run the Generate command from swift-libp2p's root directory
``` bash
swift run Generate new "my-test" --template "test-fluent" --mode listener -t tcp -s noise -m yamux --extra postgres
```

3. Build & Test
``` bash
# In your projects root directory
swift build
swift test
```

> [!NOTE]
> Ensure you have your environment variables for your DB configuration set before running the tests
> DB_HOSTNAME, DB_USERNAME, DB_PASSWORD, DB_DATABASE
