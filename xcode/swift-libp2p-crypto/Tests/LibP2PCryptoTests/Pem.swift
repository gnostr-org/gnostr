//===----------------------------------------------------------------------===//
//
// This source file is part of the swift-libp2p open source project
//
// Copyright (c) 2022-2025 swift-libp2p project authors
// Licensed under MIT
//
// See LICENSE for license information
// See CONTRIBUTORS for the list of swift-libp2p project authors
//
// SPDX-License-Identifier: MIT
//
//===----------------------------------------------------------------------===//

import Foundation

struct TestPEMKeys {

    // MARK: RSA Keys

    /// DER Format
    static let RSA_1024_PUBLIC_DER = """
        -----BEGIN RSA PUBLIC KEY-----
        MIGJAoGBANxn+vSe8nIdRSy0gHkGoJQnUIIJ3WfOV7hsSk9An9LRafuZXY
        UMB6H5RxtWFm72f7nPKlg2N5kpqk+oEuhPx4IrnXIqnN5vwu4Sbc/w8rjE
        3XxcGsgXUams3wgiBJ0r1/lLCd6a61xRGtj4+Vae+Ps3mz/TdGUkDf80dV
        ek9b9VAgMBAAE=
        -----END RSA PUBLIC KEY-----
        """

    /// RSA 1024 Public Key
    ///
    /// openssl asn1parse -i -in rsa_1024_pub.pem
    /// ```
    /// 0:d=0  hl=3 l= 159 cons: SEQUENCE
    /// 3:d=1  hl=2 l=  13 cons:  SEQUENCE
    /// 5:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
    /// 16:d=2  hl=2 l=   0 prim:   NULL
    /// 18:d=1  hl=3 l= 141 prim:  BIT STRING
    /// ```
    static let RSA_1024_PUBLIC = """
        -----BEGIN PUBLIC KEY-----
        MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDUkb0ol7/oLRbfbLqcW43xtJdh
        7xQ+Vf57DWtkEpeflZ5fUTIND+s1AlTAv63NsfUX3GJ/p+5jhnl3zweTKr7e5haM
        ZqOJaARSKpGxOBwz1K3bhvJW+izXwUrwPbcmkiAlvkAsjj1hwRpND8t/NouF+hOw
        HLQCPvkX8YLbUzoZrwIDAQAB
        -----END PUBLIC KEY-----
        """

    /// RSA 2048 Public Key
    ///
    /// openssl asn1parse -i -in rsa_2048_pub.pem
    /// ```
    /// 0:d=0  hl=4 l= 290 cons: SEQUENCE
    /// 4:d=1  hl=2 l=  13 cons:  SEQUENCE
    /// 6:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
    /// 17:d=2  hl=2 l=   0 prim:   NULL
    /// 19:d=1  hl=4 l= 271 prim:  BIT STRING
    /// ```
    static let RSA_2048_PUBLIC = """
        -----BEGIN PUBLIC KEY-----
        MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAxoLE8VNgeYQxGt5x++zW
        5INUrgTGhcrgEjDt8J78xAQ9UVZdya4q1PP79UPKb3xVvFHHAKQ2tEzzgpZvv2h8
        M/DCzvltWpsZwVx5lMiDA71xucRdF5Uiy5IiVdObIEswIN/x9AE21VJEqPihzPZf
        AGKdpd8NkeaHnAnq4Pm5uJt9S+82U5iDQzufqm5S0YTTnmpmn6guCN7H2q9WENIi
        D3mKxYzmDNyzxEhpS9jMKubvGM8p4dRSFFzQlWO1mIuO0Lf2QkgZsFMNNAEZg3ww
        LP1OO288oXz8iAapfoLq3W+I2Jg4bOarHxSIvO2zSCZ1eagUCiHAtYfbWHAugoc5
        cQIDAQAB
        -----END PUBLIC KEY-----
        """

    /// RSA 3072 Public Key
    ///
    /// openssl asn1parse -i -in rsa_3072_pub.pem
    /// ```
    /// 0:d=0  hl=4 l= 418 cons: SEQUENCE
    /// 4:d=1  hl=2 l=  13 cons:  SEQUENCE
    /// 6:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
    /// 17:d=2  hl=2 l=   0 prim:   NULL
    /// 19:d=1  hl=4 l= 399 prim:  BIT STRING
    /// ```
    static let RSA_3072_PUBLIC = """
        -----BEGIN PUBLIC KEY-----
        MIIBojANBgkqhkiG9w0BAQEFAAOCAY8AMIIBigKCAYEA0F50xB4o1O2y9Avm+YW2
        v9t5HHDI7kuWL/VxQE1R6FkpgLYQ0tUlB2QWGigYUhRS1f2ql1s7LXUVceUesHzH
        oXlzmt3Rnow9doASPROlyAcNrzniMyDzH6hz2xaVsaj3Kygc9evQzuf1Rq1R0/bj
        AkIlssumizfv70FZrmKBcgp5X9seC/wo3kWIRBV/Akx5vom0V6TEupy/39TDffnK
        a3rN9yb7+ZGrHMoXofkd+pYATyIbuwsjeCU0F2v6+pbkXDrB0bgsp6/FRx8Tw9CF
        gKf1JESSjeRQIk8nKXiqiNPmmOXvKH39lyEhWetcpn1E+bv7a4TAu1wy/FzLNp9w
        NxB5zLX6gqxUDM0YUpsPMftB2dlixO0yzLcvbUohceIb03L4mBsvWfQ3W7yYkzBo
        XTsPWofPw/jQY2IorGTKH4vgbQfW3fmqj1CXqgLEOe6XbWkcXyUnTp3Z24RiL5Ql
        dqZJs8yEKZqNP9/miIS83Onc1zjWovT7LLFCnucNgoOHAgMBAAE=
        -----END PUBLIC KEY-----
        """

    /// RSA 4096 Public Key
    ///
    /// openssl asn1parse -i -in rsa_4096_pub.pem
    /// ```
    /// 0:d=0  hl=4 l= 546 cons: SEQUENCE
    /// 4:d=1  hl=2 l=  13 cons:  SEQUENCE
    /// 6:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
    /// 17:d=2  hl=2 l=   0 prim:   NULL
    /// 19:d=1  hl=4 l= 527 prim:  BIT STRING
    /// ```
    static let RSA_4096_PUBLIC = """
        -----BEGIN PUBLIC KEY-----
        MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA96lgFtImy0lK6uxopyli
        o5WwN5FIxNdQXE6NL5o6t5Lwd3r/FCq+C1ix1Pb73lU0nJv64rCkgNn5U9rYv4e8
        /Eyu2Egp+2Ff1mBpPnNUU2oqe3de/cf8EyFR8+bQqS+cl5VSCOK2Bp87WlnjBBd7
        vy8UfjrDTIj63tNQADq/OkoUye9q7PPunTIVTvbRlC1vwVDPiCLIPUniRqAv44cG
        qM1zxRMhJTEVWJhjnaMy/NJmJQPJvnsiED3aEi/uxsUaxhpKa6JfFL8doPbeydeb
        2NE+ynG6lCYjoqmSZU+9KSwaDtutV8U8LEP9B5cHS8thdyH7uFKjGt3kgD8bVtbT
        UGL/zWWcPaqJktlM8iOh+arugW5C/fwZa1GkZZc2+Btq2MfEJ/8cSzp4nyQCk3ye
        kCQPW+7wetbqadXMpvWHFQA3HHyPEdPFF4lImSH38c0WzTWaBpXeN5dcP1T5iY+Y
        iQ7ZUSMrz8ImeYjAexHkVtE3uwaW77oBGJ33rwEXC0Agdo03mtQdKcEfB4l7gJ1G
        Tg8KWPPK0bEHew+OZxiZRfrVnGRhPVGQ5w9Kuib6Q5tP8udsEgZIZUHftf7qdscU
        kFc67jbydCj+HCD1Nvmja5u+GdJyQa021y2xbLINPaSTT1Ro8ttWRUM12Y4QEbuI
        IwumfplSdLsojGNWVCpfxAsCAwEAAQ==
        -----END PUBLIC KEY-----
        """

    /// RSA 1024 Private Key
    ///
    /// openssl asn1parse -i -in rsa_1024_priv.pem
    /// ```
    /// 0:d=0  hl=4 l= 630 cons: SEQUENCE
    /// 4:d=1  hl=2 l=   1 prim:  INTEGER           :00
    /// 7:d=1  hl=2 l=  13 cons:  SEQUENCE
    /// 9:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
    /// 20:d=2  hl=2 l=   0 prim:   NULL
    /// 22:d=1  hl=4 l= 608 prim:  OCTET STRING      [HEX DUMP]:3082...AA50
    /// ```
    static let RSA_1024_PRIVATE = """
        -----BEGIN PRIVATE KEY-----
        MIICdgIBADANBgkqhkiG9w0BAQEFAASCAmAwggJcAgEAAoGBANSRvSiXv+gtFt9s
        upxbjfG0l2HvFD5V/nsNa2QSl5+Vnl9RMg0P6zUCVMC/rc2x9RfcYn+n7mOGeXfP
        B5Mqvt7mFoxmo4loBFIqkbE4HDPUrduG8lb6LNfBSvA9tyaSICW+QCyOPWHBGk0P
        y382i4X6E7ActAI++RfxgttTOhmvAgMBAAECgYAyjMnf+l5ft0FGNpQWFMunnBuX
        5YP54vdWifVs4eL+x1TXM/bkFlIH1BsVjz+kt9oiJ32g/+1364W9URVrEPI8nk5i
        Id40q3Qiozvn4ceWtSoCGmuxIbdRqL1JJn5e8Zfzs7E8KZimYu00t2qcidFDUsEC
        biCqT14UNcbpwsOpkQJBAPxMogs3OnkgCWlptZckjg+srwxnXIkzDhSO+WaKl2Dm
        DvSMnB2Ws64EDu2dP+HLiuoyqG/ZqRabdQqIsIrkLpkCQQDXr+wYGoOiL26PrAew
        z49aH/wQZq8fBM2yuwvR+ZDRraQ0otQV1aRwffqlI+IfYowJvyPX+EC/ftcff0qa
        6R+HAkAQZlrSJ9DhNrfl5j0rifDCDBOE1uMo9+yeYXzMsY2NeubV9p3fPoCHezQN
        Nf+FCdoJxykzvA5Fre05thDjtllhAkEAgdsf2M81y1KlTQi0xKXiV8+D8dfwvUsm
        EOJ+Vlfb8fGKOEqER/UNgNDIM96ryFuLll6m1ONZEDHskMERiLysRwJAawkNDlqe
        sIjqrfR2luw+TLHkMI0T6pTY9s+79F9VVV/V13v2qtTpXw1eu7Sw+oDBpJoocz/h
        +YzU+CyyzO+qUA==
        -----END PRIVATE KEY-----
        """

    /// RSA 2048 Private Key
    ///
    /// openssl asn1parse -i -in rsa_2048_priv.pem
    /// ```
    /// 0:d=0  hl=4 l=1214 cons: SEQUENCE
    /// 4:d=1  hl=2 l=   1 prim:  INTEGER           :00
    /// 7:d=1  hl=2 l=  13 cons:  SEQUENCE
    /// 9:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
    /// 20:d=2  hl=2 l=   0 prim:   NULL
    /// 22:d=1  hl=4 l=1192 prim:  OCTET STRING      [HEX DUMP]:3082...59D7
    /// ```
    static let RSA_2048_PRIVATE = """
        -----BEGIN PRIVATE KEY-----
        MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDGgsTxU2B5hDEa
        3nH77Nbkg1SuBMaFyuASMO3wnvzEBD1RVl3JrirU8/v1Q8pvfFW8UccApDa0TPOC
        lm+/aHwz8MLO+W1amxnBXHmUyIMDvXG5xF0XlSLLkiJV05sgSzAg3/H0ATbVUkSo
        +KHM9l8AYp2l3w2R5oecCerg+bm4m31L7zZTmINDO5+qblLRhNOeamafqC4I3sfa
        r1YQ0iIPeYrFjOYM3LPESGlL2Mwq5u8Yzynh1FIUXNCVY7WYi47Qt/ZCSBmwUw00
        ARmDfDAs/U47bzyhfPyIBql+gurdb4jYmDhs5qsfFIi87bNIJnV5qBQKIcC1h9tY
        cC6ChzlxAgMBAAECggEBAJvNVR+HbgfRvey1vEaa+4p8nUC7lMi7kyQT7RxW3FJI
        dYvaOmApZ4qeOBmm7EKWFoBousUBHcJjRxguVGSpcBogE/X4hGCBrTQ7DV2+Bj4w
        OQsxWFNDBP07o+Ey5OTyvkJ/Idp9/XhuSl9ITU2d7LBTtiHSsEbb5YGNsyCCP8bo
        OP/PQZPDgXz9vLg674rPgm32cHPIWDomspJ34EkD/szyQOpW2AM99v3NbqovM8ie
        T91266iPASTngGQG3qA54zJ91Ulu7kx+LNYpSknZOvWxrxDB8+oArdEZFuPpT5L6
        OKj5ICJCFRkiROE2xErn/nUmu4R1AA3WDnFZQMzGFAECgYEA5dDYyHUhtjlM13OL
        hUoiBWk4yzz3wB/jcvvdq8OVU0nWeuTiZh4cPSDp67q4AAffAbJvsqYomyA/Eqt/
        NNZKE7ZP9a+KFXbyd1BtDgFiVEcNGNZ0K7beQEbVlG7sFN+s8YT7eeObBoXyhhPA
        WC6fWcYfrZatCuNAmcjutoWCarECgYEA3SDViFs2Eb5II9DHwljNJF8gFgRJhAKj
        bMEGUVgSK8WWQIO2ZdzlA1kV+U0S5FXx3zyQAWVqCkdAzFB4sZjmT47NZN4cvo30
        E32rXWDoNbKpljBvEhx2HUMA+Zpdpe/vZUHlNpqGT0MZaP6nPTg79EsBdiMvVC8l
        3UC7XW5f6sECgYA1AMzutq0WxPJnAnwcOrPMAa+amC4fvnsLyvEeK1amRfJUl7Nr
        j+g9ZPjuaDsFrssNLiU6072rwW0qlikZe47MKxEX/etf9fYH9KGiSElwXI61ushC
        SMPLmUqrGEYUrl3JujzxqL/Zak08BRQogmA4KUynEYhJaY49qaz8paAlkQKBgDer
        U3avl84hvGGf5xprZsHYXOiODb/5NhFkCuYhqPlyFeCKCDpewRz1qY2ItM/dPzY3
        Nf3T/T03MP3+6FO1rY2r4tOZA12JuT/K7IBmrC8Qmpcf/GZv2eCGBNHR5e+nlvpD
        +6OihVuhBd2j9pB3/sgCtgx60Sh9cifgawsbhXRBAoGBALXBsU6d6TRU0PTOwvos
        zV4dWva41cuElT1hp4rzgSgRtUIbaFVpGi4STb/B3znvlE4VNsePgq2oe6c4Bx7l
        JwZ4bU1ZvxWZ+tLdYnelwhfgq/14tjWPGvlE+bF7s1irJIHsTsoShF+RfavhqVXF
        6FyFEjB0XDmXjJgIxkenwlnX
        -----END PRIVATE KEY-----
        """

    /// RSA 3072 Private Key
    ///
    /// openssl asn1parse -i -in rsa_3072_priv.pem
    /// ```
    /// 0:d=0  hl=4 l=1791 cons: SEQUENCE
    /// 4:d=1  hl=2 l=   1 prim:  INTEGER           :00
    /// 7:d=1  hl=2 l=  13 cons:  SEQUENCE
    /// 9:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
    /// 20:d=2  hl=2 l=   0 prim:   NULL
    /// 22:d=1  hl=4 l=1769 prim:  OCTET STRING      [HEX DUMP]:3082...C813
    /// ```
    static let RSA_3072_PRIVATE = """
        -----BEGIN PRIVATE KEY-----
        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
        9QS5ykVryo8URNisVCornefIEw==
        -----END PRIVATE KEY-----
        """

    /// RSA 4096 Private Key
    ///
    /// openssl asn1parse -i -in rsa_4096_priv.pem
    /// ```
    /// 0:d=0  hl=4 l=2370 cons: SEQUENCE
    /// 4:d=1  hl=2 l=   1 prim:  INTEGER           :00
    /// 7:d=1  hl=2 l=  13 cons:  SEQUENCE
    /// 9:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
    /// 20:d=2  hl=2 l=   0 prim:   NULL
    /// 22:d=1  hl=4 l=2348 prim:  OCTET STRING      [HEX DUMP]:3082...B4EA
    /// ```
    static let RSA_4096_PRIVATE = """
        -----BEGIN PRIVATE KEY-----
        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
        PgBeOgkLlpYxcq87zFpg5qyHdq+06g==
        -----END PRIVATE KEY-----
        """

    /// An encrypted RSA 1024 private key and it's original unencrypted pem file for testing AES CBC PBKDF2 decryption of PEM files...
    ///
    /// To decrypt an encrypted private RSA key...
    /// 1) Strip the headers of the PEM and base64 decode the data
    /// 2) Parse the data via ASN1 looking for the encryption algo, salt, iv and itterations used, and the ciphertext (aka octet string)
    /// 3) Derive the encryption key using PBKDF2 (sha1, salt and itterations)
    /// 4) Use encryption key to instantiate the AES CBC Cipher along with the IV
    /// 5) Decrypt the encrypted octet string
    /// 6) The decrypted octet string can be ASN1 parsed again for the private key octet string
    /// 7) This raw data can be used to instantiate a SecKey
    struct RSA_1024_PRIVATE_ENCRYPTED_PAIR {
        /// An unencrypted RSA 1024 Private Key
        ///
        /// Generated with
        /// ```
        /// openssl genpkey -algorithm RSA
        ///   -pkeyopt rsa_keygen_bits:1024
        ///   -pkeyopt rsa_keygen_pubexp:65537
        ///   -out foo.pem
        /// ```
        static let UNENCRYPTED = """
            -----BEGIN PRIVATE KEY-----
            MIICdQIBADANBgkqhkiG9w0BAQEFAASCAl8wggJbAgEAAoGBAMrTsyVLP/Ureyqm
            zJDcolbO9cafCafeGXHrWJ4ar6QL+tT/apk+6kkgqvRU4QpJFurbzDXhmLpiUk9u
            t2Oy6lw0LrF7Nz/XGHXfNHutLS6+jrHI+9x55l87CQyObsdaTt1jhP3IHp6FIA/S
            rAAiCrFiPG7L97OdqREa2uWYIrupAgMBAAECgYAigSMvy/5kafI5Dkkst6wSUoDz
            Oij9WsY/YAciVm3c3YDdbVooGdDngdwzVqE2C7sPVzcFT4yY4JMaGj6ugkhl+2mm
            8BP6GkOGYbwrMgyjPXLjg4mmeQS75NxxzVFcGxM3405G4p833DxNJyFZyJpfA0b6
            5qhn4J2ZrxTwGu/TxQJBAPUxawoM1E3kRrJ+l30zHLVu0cbS4yXCYD24a9cWy6GI
            aSNnJAzsb5eGkfg7epKNIov/sTh5RLeUu9x6d3Xe0qcCQQDTxEDSmBqdB0GhQF/9
            7JXVdS25WEB4vJ/eVPPMDSoSS6IAR3+noVpQuLXqdIrAPA6alTsX1oxEKl6M6hTQ
            0lkvAkA9t2Kp9PC7amohI5wd92+Se4Jx+UMTjgmLf5AlY6d90UglkSCR4DF2gnjb
            cp03pi677nA9NskFLHrc1DadhKihAkAE443/jqVmpKk+OMc+jHy1Ddx9X+01HF2w
            e1OZjWBAReC6kuv+iboVDP6eKAyf/YL0zKctmLVqSXQfWrQaUhDfAkBNmxhdcL3M
            18PnvSVbfuwhKuNQd3lf8Xpr9eSHOnpYglqAbLObNHqZN24v+MI5M6Rdp6+yXryE
            vcUs0rZnDXkl
            -----END PRIVATE KEY-----
            """

        /// The encrypted version of the RSA 1024 Private Key
        ///
        /// Encrypted with
        /// ```
        /// openssl pkcs8
        ///     -in foo.pem
        ///     -topk8
        ///     -v2 aes-128-cbc
        ///     -passout pass:mypassword
        /// ```
        static let ENCRYPTED = """
            -----BEGIN ENCRYPTED PRIVATE KEY-----
            MIICzzBJBgkqhkiG9w0BBQ0wPDAbBgkqhkiG9w0BBQwwDgQI49PtP+7yJmgCAggA
            MB0GCWCGSAFlAwQBAgQQYz/oWtq4qhWPNrAQiO3i5wSCAoCjWvOSqAMdA4qDF8BB
            aaqGRnZ/Lvewsrs4keppFogFnYpeVkzEmeleQLIYkO2mnNvsjhfh2Vk1LW/qNPIl
            NvwjXyNbP1E6TlLmTNEAgIfyViHOCuk+17tkgAtK98huFTi0U+LbMcaxSnJ7CsNY
            9JODko7fLXMpEaGy5qcuXWsMHG1iKcggYs0J1kmWSVw9ZQP7Uh9hs31zz60kFe+T
            1I8EOjC06EcKY2HmOhzS+p378nWD3Lxi49FWkHslx1OtQwAXqMG5xWSo+kTWgmUx
            fB3Olmv7opDcQ5OtOSxRjM/6SCtrtIlPRjIS7Uu4foW2BpFS+mkkvaJR0lMiEFjA
            qMdLu3MZzT8U9lEDpd+ki+OjIC2bOXkv/OgHFmHjrTrGTVnK+HP5B0XkcaN0kmi5
            ypd8/XB4zDqO/eSSTKnDe5cvw9Ruj5vt9cesUGjckTlVlZ7Sip2nqtngEAh0k7gc
            p8p0LpNRyOM5edxNCsRLWj3Z9oskkbEFbL3INuVr6HZ5C1IpUHaxzdii1FBeLSqY
            RYCC7iOgfqRILkBN2dsnWhdLLvcVpeQqSccnNCYSrXgr40T8BqZKLnuhHT7/iZaw
            OiKp9MyygPf0wO5IFaSglpk02dohJpg/LYxFBZk+qJKPR9883NrtSPSzXxDogu2f
            /tc8OCoH919cB8WAsU1cvKYMxsr9HTfoxS7itrJX9d7tE3J2Ky7fQrPWt247BXSE
            FMUJ8BQpLL/2lNIxW9clLEuzr0RZKu3AhBU0V0o8KDucrsLPdbLvV9/J8+G8VJWB
            DZjkXrHO2Oob0rOBtz0gnIF4TSwMWlI28OFWLwN3ByGeT0KcDN7SghLtDSyEQKNW
            ZHiA
            -----END ENCRYPTED PRIVATE KEY-----
            """
    }

    // MARK: EC Keys

    /// EC P256 Public Key
    ///
    /// openssl asn1parse -i -in ec_256_pub.pem
    /// ```
    /// 0:d=0  hl=2 l=  89 cons: SEQUENCE
    /// 2:d=1  hl=2 l=  19 cons:  SEQUENCE
    /// 4:d=2  hl=2 l=   7 prim:   OBJECT            :id-ecPublicKey
    /// 13:d=2  hl=2 l=   8 prim:   OBJECT            :prime256v1
    /// 23:d=1  hl=2 l=  66 prim:  BIT STRING
    /// ```
    static let EC_256_PUBLIC = """
        -----BEGIN PUBLIC KEY-----
        MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEb4nB0k8CBVnKCHVHkxuXAkSlZuO5
        Nsev1rzcRv5QHiJuWUKomFGadQlMSGwoDOHEDdW3ujcA6t0ADteHw6KrZg==
        -----END PUBLIC KEY-----
        """

    /// EC P384 Public Key
    ///
    /// openssl asn1parse -i -in ec_384_pub.pem
    /// ```
    /// 0:d=0  hl=2 l= 118 cons: SEQUENCE
    /// 2:d=1  hl=2 l=  16 cons:  SEQUENCE
    /// 4:d=2  hl=2 l=   7 prim:   OBJECT            :id-ecPublicKey
    /// 13:d=2  hl=2 l=   5 prim:   OBJECT            :secp384r1
    /// 20:d=1  hl=2 l=  98 prim:  BIT STRING
    /// ```
    static let EC_384_PUBLIC = """
        -----BEGIN PUBLIC KEY-----
        MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEBwY0l7mq7hSBEZRld5ISWfSoFsYN3wwM
        hdD3cMU95DmYXzbqVHB4dCfsy7bexm4h9c0zs4CyTPzy3DV3vfmv1akQJIQv7l08
        lx/YXNeGXTN4Gr9r4rwA5GvRl1p6plPL
        -----END PUBLIC KEY-----
        """

    /// EC P521 Public Key
    ///
    /// openssl asn1parse -i -in ec_521_pub.pem
    /// ```
    /// 0:d=0  hl=3 l= 155 cons: SEQUENCE
    /// 3:d=1  hl=2 l=  16 cons:  SEQUENCE
    /// 5:d=2  hl=2 l=   7 prim:   OBJECT            :id-ecPublicKey
    /// 14:d=2  hl=2 l=   5 prim:   OBJECT            :secp521r1
    /// 21:d=1  hl=3 l= 134 prim:  BIT STRING
    /// ```
    static let EC_521_PUBLIC = """
        -----BEGIN PUBLIC KEY-----
        MIGbMBAGByqGSM49AgEGBSuBBAAjA4GGAAQAp3v1UQWvSyQnkAUEBu+x/7ZrPtNJ
        SCUk9kMvuZMyGP1idwvspALuJjzrSFFlXObjlOjxucSbWhTYF/o3nc0XzpAA3dxA
        BYiMqH9vrVePoJMpv+DMdkUiUJ/WqHSOu9bJEi1h4fdqh5HHx4QZJY/iX/59VAi1
        uSbAhALvbdGFbVpkcOs=
        -----END PUBLIC KEY-----
        """

    /// EC P256 Private Key
    ///
    /// openssl asn1parse -i -in ec_priv_256.pem
    /// ```
    /// 0:d=0  hl=2 l= 119 cons: SEQUENCE
    /// 2:d=1  hl=2 l=   1 prim:  INTEGER           :01
    /// 5:d=1  hl=2 l=  32 prim:  OCTET STRING      [HEX DUMP]:7C12DEBEED7417C33D239A4FFC7A036AAF5C51579469A7698931CEB8F5090507
    /// 39:d=1  hl=2 l=  10 cons:  cont [ 0 ]
    /// 41:d=2  hl=2 l=   8 prim:   OBJECT            :prime256v1
    /// 51:d=1  hl=2 l=  68 cons:  cont [ 1 ]
    /// 53:d=2  hl=2 l=  66 prim:   BIT STRING
    /// ```
    static let EC_256_PRIVATE = """
        -----BEGIN EC PRIVATE KEY-----
        MHcCAQEEIHwS3r7tdBfDPSOaT/x6A2qvXFFXlGmnaYkxzrj1CQUHoAoGCCqGSM49
        AwEHoUQDQgAE79HvsMQC9IyhZ7yCCYKmgz9zewM4KziWoVMXKN+7Cd5Ds+jK8V5q
        hD6YVbbo/v1udmM5DfhHJiUW3Ww5++suRg==
        -----END EC PRIVATE KEY-----
        """

    /// EC P384 Private Key
    ///
    /// openssl asn1parse -i -in ec_priv_384.pem
    /// ```
    /// 0:d=0  hl=3 l= 164 cons: SEQUENCE
    /// 3:d=1  hl=2 l=   1 prim:  INTEGER           :01
    /// 6:d=1  hl=2 l=  48 prim:  OCTET STRING      [HEX DUMP]:EB37EAA3BD6ED3A9C5EB2A54C56D23FC01D6EC21DAAF4408161E568189C3FB764C7E1CA42275289207644B5B28D2AECE
    /// 56:d=1  hl=2 l=   7 cons:  cont [ 0 ]
    /// 58:d=2  hl=2 l=   5 prim:   OBJECT            :secp384r1
    /// 65:d=1  hl=2 l= 100 cons:  cont [ 1 ]
    /// 67:d=2  hl=2 l=  98 prim:   BIT STRING
    /// ```
    static let EC_384_PRIVATE = """
        -----BEGIN EC PRIVATE KEY-----
        MIGkAgEBBDDrN+qjvW7TqcXrKlTFbSP8AdbsIdqvRAgWHlaBicP7dkx+HKQidSiS
        B2RLWyjSrs6gBwYFK4EEACKhZANiAAQrRiaztGpInYo1XqMnNokWY6g1TcgMuzgq
        Ug6LzFQbCAqCrcnM9+c9Z4/63dC06ulL/KbLQgThjSiqRzgbzvmOvB0OzlpX1weK
        usFrF4Pi0B9pKPmVCAlSzaxVEmRsbmw=
        -----END EC PRIVATE KEY-----
        """

    /// EC P521 Private Key
    ///
    /// openssl asn1parse -i -in ec_priv_521.pem
    /// ```
    /// 0:d=0  hl=3 l= 219 cons: SEQUENCE
    /// 3:d=1  hl=2 l=   1 prim:  INTEGER           :01
    /// 6:d=1  hl=2 l=  65 prim:  OCTET STRING      [HEX DUMP]:5A694D1575C8038BE99EEB94E7851A3DF80A1D715E4339F6E2F14B5E783A688D75B93DF90A5CD43EA89940C80D2756690996BC123A5A921FB5C6CC7B8E4EEFA777
    /// 73:d=1  hl=2 l=   7 cons:  cont [ 0 ]
    /// 75:d=2  hl=2 l=   5 prim:   OBJECT            :secp521r1
    /// 82:d=1  hl=3 l= 137 cons:  cont [ 1 ]
    /// 85:d=2  hl=3 l= 134 prim:   BIT STRING
    /// ```
    static let EC_521_PRIVATE = """
        -----BEGIN EC PRIVATE KEY-----
        MIHbAgEBBEFaaU0VdcgDi+me65TnhRo9+AodcV5DOfbi8UteeDpojXW5PfkKXNQ+
        qJlAyA0nVmkJlrwSOlqSH7XGzHuOTu+nd6AHBgUrgQQAI6GBiQOBhgAEAZMhoDRn
        GAeReuc4sKEq3fznP1rPZ4QDdwpNfxQbPLe0rzg4fk+J6BPlyQs74RfHtXxiHOiL
        3GZJLzo/pPbi96z7AG1AEABHWCcmi/uclGsjg0wNuKuWHwY8bJGvHZIBtd+px5+L
        6L0wg93uMy3o2nMEJd01n18LGvjdl3GUvgq2kXQN
        -----END EC PRIVATE KEY-----
        """

    struct SECP256k1_KeyPair {
        /// Can be derived from the PRIVATE Key
        static let PUBLIC = """
            -----BEGIN PUBLIC KEY-----
            MFYwEAYHKoZIzj0CAQYFK4EEAAoDQgAEIgC+scMFLUBdd3OlModp6SbEaBGrHyzw
            xDevjsbU1gOhdju+FQZaALwfX7XmsHhKFFNYpVS0GXhMMzzFf1Ld7w==
            -----END PUBLIC KEY-----
            """

        /// Should be able to derive the PUBLIC key
        static let PRIVATE = """
            -----BEGIN EC PRIVATE KEY-----
            MHQCAQEEIJmbpwD3mZhlEtiGzmgropJ/nSewc8UPyBE9wib742saoAcGBSuBBAAK
            oUQDQgAEIgC+scMFLUBdd3OlModp6SbEaBGrHyzwxDevjsbU1gOhdju+FQZaALwf
            X7XmsHhKFFNYpVS0GXhMMzzFf1Ld7w==
            -----END EC PRIVATE KEY-----
            """
    }

    struct Ed25519_KeyPair {
        /// Can be derived from the PRIVATE Key
        static let PUBLIC = """
            -----BEGIN PUBLIC KEY-----
            MCowBQYDK2VwAyEACM3Nzttt7KmXG9qDEYys++oQ9G749jqrbRRs92BUzpA=
            -----END PUBLIC KEY-----
            """

        /// Should be able to derive the PUBLIC key
        static let PRIVATE = """
            -----BEGIN PRIVATE KEY-----
            MC4CAQAwBQYDK2VwBCIEIOkK9EOHRqD5QueUrMZbia55UWpokoFpWco4r2GnRVZ+
            -----END PRIVATE KEY-----
            """
    }

    // MARK: Certificates

    static let CERT_FACEBOOK = """
        -----BEGIN CERTIFICATE-----
        MIIH5DCCBsygAwIBAgIQDACZt9eJyfZmJjF+vOp8HDANBgkqhkiG9w0BAQsFADBw
        MQswCQYDVQQGEwJVUzEVMBMGA1UEChMMRGlnaUNlcnQgSW5jMRkwFwYDVQQLExB3
        d3cuZGlnaWNlcnQuY29tMS8wLQYDVQQDEyZEaWdpQ2VydCBTSEEyIEhpZ2ggQXNz
        dXJhbmNlIFNlcnZlciBDQTAeFw0xNjEyMDkwMDAwMDBaFw0xODAxMjUxMjAwMDBa
        MGkxCzAJBgNVBAYTAlVTMRMwEQYDVQQIEwpDYWxpZm9ybmlhMRMwEQYDVQQHEwpN
        ZW5sbyBQYXJrMRcwFQYDVQQKEw5GYWNlYm9vaywgSW5jLjEXMBUGA1UEAwwOKi5m
        YWNlYm9vay5jb20wWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASg8YyvpzmIaFsT
        Vg4VFbSnRe8bx+WFPCsE1GWKMTEi6qOS7WSdumWB47YSdtizC0Xx/wooFJxP3HOp
        s0ktoHbTo4IFSjCCBUYwHwYDVR0jBBgwFoAUUWj/kK8CB3U8zNllZGKiErhZcjsw
        HQYDVR0OBBYEFMuYKIyhcufiMqmaPfINoYFWoRqLMIHHBgNVHREEgb8wgbyCDiou
        ZmFjZWJvb2suY29tgg4qLmZhY2Vib29rLm5ldIIIKi5mYi5jb22CCyouZmJjZG4u
        bmV0ggsqLmZic2J4LmNvbYIQKi5tLmZhY2Vib29rLmNvbYIPKi5tZXNzZW5nZXIu
        Y29tgg4qLnh4LmZiY2RuLm5ldIIOKi54eS5mYmNkbi5uZXSCDioueHouZmJjZG4u
        bmV0ggxmYWNlYm9vay5jb22CBmZiLmNvbYINbWVzc2VuZ2VyLmNvbTAOBgNVHQ8B
        Af8EBAMCB4AwHQYDVR0lBBYwFAYIKwYBBQUHAwEGCCsGAQUFBwMCMHUGA1UdHwRu
        MGwwNKAyoDCGLmh0dHA6Ly9jcmwzLmRpZ2ljZXJ0LmNvbS9zaGEyLWhhLXNlcnZl
        ci1nNS5jcmwwNKAyoDCGLmh0dHA6Ly9jcmw0LmRpZ2ljZXJ0LmNvbS9zaGEyLWhh
        LXNlcnZlci1nNS5jcmwwTAYDVR0gBEUwQzA3BglghkgBhv1sAQEwKjAoBggrBgEF
        BQcCARYcaHR0cHM6Ly93d3cuZGlnaWNlcnQuY29tL0NQUzAIBgZngQwBAgIwgYMG
        CCsGAQUFBwEBBHcwdTAkBggrBgEFBQcwAYYYaHR0cDovL29jc3AuZGlnaWNlcnQu
        Y29tME0GCCsGAQUFBzAChkFodHRwOi8vY2FjZXJ0cy5kaWdpY2VydC5jb20vRGln
        aUNlcnRTSEEySGlnaEFzc3VyYW5jZVNlcnZlckNBLmNydDAMBgNVHRMBAf8EAjAA
        MIICsAYKKwYBBAHWeQIEAgSCAqAEggKcApoAdgCkuQmQtBhYFIe7E6LMZ3AKPDWY
        BPkb37jjd80OyA3cEAAAAVjl02IEAAAEAwBHMEUCIQDvWFsUeqWE/xwIYcXPvbb5
        ExzfHBZTNwfnUf4RPO/lBgIgdOGmr0j7+u8/S+7tfFw71ZEjqpwJELl/sEFuQdPn
        pwQBLwCsO5rtf6lnR1cVnm19V1Zy+dmBAJQem97/7KExO3V4LQAAAVjl02IoAAAE
        AQEAYvnMV+BfP3Wrk4yFQE/Zx5WsjSabYOpLj1Tj5xFaoVoHdGqLCf/Hi+Vv0IRy
        ePKFBCSW0+3eA589+WnCDMwcJlBYeZV8MlvHFZg3a66Uhx/OAvoetb0mCtUpnmIE
        UwLX/eMNEvjg2qTH3/33ysCo2l25+/EcR8upF+2KIcmnk5WwaJzfq7cFPQc4Cvcz
        mTHasJi/jmVaIaJ9HC50g3dx584TQX26lDLddF/Li4uEbJ7TSopnTzjQdWBtWbMF
        h3bcfhFCKaqK2kIJV3bgup5HibEnZ2LPm6lekY072ZFCGM4QYc4ukqzou2JWCRmG
        o0dMHJhnvQXpnIQGwATqCD4Q1AB2AFYUBpov18Ls0/XhvUSyPsdGdrm8mRFcwO+U
        mFXWidDdAAABWOXTYrkAAAQDAEcwRQIgGhXXbwUO5bD4Ts/Q0gqZwUS2vl/A4Hem
        k7ovxl82v9oCIQCbtkflDXbcunY4MAQCbKlnesPGc/nftA84xDhJpxFHWQB3AO5L
        vbd1zmC64UJpH6vhnmajD35fsHLYgwDEe4l6qP3LAAABWOXTZBEAAAQDAEgwRgIh
        AKubngQoa5Iak8eCOrffH7Xx3AP1NMb5pFw35nt2VSeRAiEA47Kq1UQcDXIEsV+W
        nuPd9LM5kpdeu0+TiHKtTLRQr0swDQYJKoZIhvcNAQELBQADggEBADrNSsoonbj1
        YGjwy9t9wP9+kZBwrNMO2n5N5fQNhGawkEAX+lXlzgm3TqYlTNi6sCFbPBAErim3
        aMVlWuOlctgnjtAdmdWZ4qEONrBLHPGgukDJ3Uen/EC/gwK6KdBCb4Ttp6MMPY1c
        hb/ciTLi3QUUU4h4OJWqUjvccBCDs/LydNjKWZZTxLJmxRSmfpyCU3uU2XHHMNlo
        8UTIlqZsOtdqhg7/Q/cvMDHDkcI/tqelmg0MD2H9KpcmAvVkwgjn+BVpv5HELl+0
        EP0UhYknI1B6LBecJuj7jI26eXZdX35CYkpI/SZA9KK+OYKHh6vCxKqnRZ9ZQUOj
        XnIWKQeV5Hg=
        -----END CERTIFICATE-----
        """
}
