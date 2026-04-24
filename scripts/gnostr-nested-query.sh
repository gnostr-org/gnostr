cargo run --bin gnostr -- query \
          --ids \
            $(\
            gnostr \
            bech32-to-any \
            npub15d9enu3v0yxyud4jk0pvxk3kmvrzymjpc6f0eq4ck44vr32qck7smrxq6k --raw \
           ) \
          && echo "✅" ## || echo "❌"
