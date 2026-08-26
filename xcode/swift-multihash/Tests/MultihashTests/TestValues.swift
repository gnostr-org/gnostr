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

extension MultihashTests {

    /// An array of test tuples with the following key (algorithm, bits, input ,multihash)
    ///  - Note: in order to compute `multihash` from `input`
    ///    - let d = input.data(using: .utf8)
    ///    - let hash = HASH(d)
    ///    - let mh = try encodeBuf(hash, code: HASH_CODE)
    ///    - mh.asString(base: .base16) == `multihash`
    ///  - WARNING:
    ///    Do not attempt to decode `input` as a hex string. The value should be converted to data directly using .utf8 encoding...
    public static let TestFixtures: [(algorithm: String, bits: String, input: String, multihash: String)] = [
        (
            algorithm: "sha1", bits: "80", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "110ae861e452cfd84dca9a17"
        ),
        (
            algorithm: "sha1", bits: "160", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "1114e861e452cfd84dca9a176258c3d06e020a0c93d8"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "120affb31f07aa15348368c9"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "1214ffb31f07aa15348368c90c73a834dbf9f8710365"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "1220ffb31f07aa15348368c90c73a834dbf9f8710365860fa0fd4ddce9ac2782cc17"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "130aabffa1926b038a2f1833"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "1314abffa1926b038a2f1833092d93859ba2ee56e07a"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "1320abffa1926b038a2f1833092d93859ba2ee56e07a86b158200fc7d4d5d1eaf350"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash:
                "1340abffa1926b038a2f1833092d93859ba2ee56e07a86b158200fc7d4d5d1eaf350b0f0e511ea6f5043a9f4a7c7886f93355652114709d75b2db4c33741124fd4fe"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "140a9a7a8207a57d03e9c524"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "14149a7a8207a57d03e9c524ae7fd39563bfe1a466a3"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash: "14209a7a8207a57d03e9c524ae7fd39563bfe1a466a3a0323875eba8b034a1d59c3b"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "431fb5d4c9b735ba1a34d0df045118806ae2336f2c",
            multihash:
                "14409a7a8207a57d03e9c524ae7fd39563bfe1a466a3a0323875eba8b034a1d59c3b7218103543f7777f17ef03dcaf44d12c74dfb83726e7425cf61225e9a54b3b3a"
        ),
        (
            algorithm: "sha1", bits: "80", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "110aa29eea788860fb1b2884"
        ),
        (
            algorithm: "sha1", bits: "160", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "1114a29eea788860fb1b288408345592c8662a4dcde5"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "120a3afa34fba2f3572ac56d"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "12143afa34fba2f3572ac56d80f52002e8727dd95f17"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "12203afa34fba2f3572ac56d80f52002e8727dd95f17a2e7c9376a3147a96d3e78ac"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "130a1c452c7d9f572510fa1a"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "13141c452c7d9f572510fa1a3224e37a3030c8750b3e"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "13201c452c7d9f572510fa1a3224e37a3030c8750b3e1763f1ad0d38274b3ea58ca7"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash:
                "13401c452c7d9f572510fa1a3224e37a3030c8750b3e1763f1ad0d38274b3ea58ca7b4317dd0913250b4b16c9b0f6281aa74c42d8d5e72043e0fb214d3ad51d4ee8c"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "140a968e697b2d5e92470002"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "1414968e697b2d5e92470002f9e59e13557f47b895dc"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash: "1420968e697b2d5e92470002f9e59e13557f47b895dc9c79082a90e91515f0255637"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "2d6db2d7882fa8b7d56e74b8e24036deb475de8c94",
            multihash:
                "1440968e697b2d5e92470002f9e59e13557f47b895dc9c79082a90e91515f025563773aec0f70219c87350c79707de67500866d7fe084c5316e12c6930949b28865d"
        ),
        (
            algorithm: "sha1", bits: "80", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "110ac8a5b445ce470e4bc548"
        ),
        (
            algorithm: "sha1", bits: "160", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "1114c8a5b445ce470e4bc54884e60e48e31fe12cb1a6"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "120aef387c6fea7ab702d470"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "1214ef387c6fea7ab702d470a9432c4cde44199c8d06"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "1220ef387c6fea7ab702d470a9432c4cde44199c8d06c6eaa5a1f2c61cfb0b70a97f"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "130aa1e9a63a3835d69983dd"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "1314a1e9a63a3835d69983dddefe78f1d9b233c9844d"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "1320a1e9a63a3835d69983dddefe78f1d9b233c9844d42ac14bb4e62d39f1f007b35"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash:
                "1340a1e9a63a3835d69983dddefe78f1d9b233c9844d42ac14bb4e62d39f1f007b350dc152a0b3f3cbd20bfdf0e19d9b62ca71f1ae8a8c7e67268ca92325d549deb3"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "140a47f57e2056010afa03bc"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "141447f57e2056010afa03bc58141ba3754f41917518"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash: "142047f57e2056010afa03bc58141ba3754f41917518c81711236eaca3766e333b9a"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "6c00022ba29d15926c4580332ded091e666f0ec5d9",
            multihash:
                "144047f57e2056010afa03bc58141ba3754f41917518c81711236eaca3766e333b9a2a767a90363f7a179e776d85aa6610713709ee46531f9a454565f737c68bdc56"
        ),
        (
            algorithm: "sha1", bits: "80", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "110aae1351506b464f8e5deb"
        ),
        (
            algorithm: "sha1", bits: "160", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "1114ae1351506b464f8e5deb0f6a2b2a272dcacbfadf"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "120a8a54b2b6b031da07e720"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "12148a54b2b6b031da07e72055828cbf41ad2562a6f3"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "12208a54b2b6b031da07e72055828cbf41ad2562a6f3675fcc84857577cbd1a935c1"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "130aeb89b3c6225e2b0b984e"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "1314eb89b3c6225e2b0b984e5267260759ba43351a04"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "1320eb89b3c6225e2b0b984e5267260759ba43351a04d8769973a2d54b670d96b98e"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash:
                "1340eb89b3c6225e2b0b984e5267260759ba43351a04d8769973a2d54b670d96b98e0d90cb7dddcc370c5b4f75be260f0927da4215b0952a6cd4edefaf35f68821d9"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "140a89c62811bc2d3fec8163"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "141489c62811bc2d3fec81631112ad9f03de23d06569"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash: "142089c62811bc2d3fec81631112ad9f03de23d065697f758b8df9e5d791474ab2f2"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "1391551dc8f15110d256c493ed485bca8cfed07241",
            multihash:
                "144089c62811bc2d3fec81631112ad9f03de23d065697f758b8df9e5d791474ab2f20ddac684c23b203b730be465a5d0f06b76e9dbc48590def7d58e96d93b4e0418"
        ),
        (
            algorithm: "sha1", bits: "80", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "110a40c95ed89d5a648d4057"
        ),
        (
            algorithm: "sha1", bits: "160", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "111440c95ed89d5a648d4057e6e57d7906dd58d2bb66"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "120ab17e5d52e94b25c5fe09"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "1214b17e5d52e94b25c5fe098c233133c5d43556b35d"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "1220b17e5d52e94b25c5fe098c233133c5d43556b35d2a50c59cf617fc61df1713fd"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "130a0339347318e40bf23fc0"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "13140339347318e40bf23fc03c93569df19c767e2495"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "13200339347318e40bf23fc03c93569df19c767e2495cdcfa30aa7385fd43dd6a91f"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash:
                "13400339347318e40bf23fc03c93569df19c767e2495cdcfa30aa7385fd43dd6a91fdd0c3962be324cf4c2527cc17aa65aa0349df6260599e38c7bd2fea9a178a66b"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "140aa7ab209784597d2e2518"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "1414a7ab209784597d2e251860e2464c04c386c4887a"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash: "1420a7ab209784597d2e251860e2464c04c386c4887af778136414c80d7c643ccd3b"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "557f47c855a5ca40daa3c0904a1e43647b4021ce0c",
            multihash:
                "1440a7ab209784597d2e251860e2464c04c386c4887af778136414c80d7c643ccd3b2a0b61e31986a79ef8e4fdd41601fbe7c982c132df4bc77ecc2e27d3edd55f90"
        ),
        (
            algorithm: "sha1", bits: "80", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "110a96c5114c52a3ea787360"
        ),
        (
            algorithm: "sha1", bits: "160", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "111496c5114c52a3ea7873603eb59da1ab88c0ec2ceb"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "120ab8018a7f12b6da389260"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "1214b8018a7f12b6da3892608167fc5ead90fc57fd1e"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "1220b8018a7f12b6da3892608167fc5ead90fc57fd1e83aefe9453b623022d5f2b9b"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "130a6885c2f6127bc432ff56"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "13146885c2f6127bc432ff56198855ee6543a821c4b6"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "13206885c2f6127bc432ff56198855ee6543a821c4b6ced4e250d1b5c9d628e24b94"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash:
                "13406885c2f6127bc432ff56198855ee6543a821c4b6ced4e250d1b5c9d628e24b946a0aa1e5eba54310d8710e26075c82ad0ba6a8f8639dd56366976b79d067f368"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "140a3dc43b479f5e9e008a52"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "14143dc43b479f5e9e008a5256ca442e66286995c39d"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash: "14203dc43b479f5e9e008a5256ca442e66286995c39deb732a6fb4e2e791a3c4a6a6"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "4c44254356730838195a32cbb1b8be3bed4c6c05c0",
            multihash:
                "14403dc43b479f5e9e008a5256ca442e66286995c39deb732a6fb4e2e791a3c4a6a6e5322e571e945a78748896edc67866ab00c767320f6956833857eab991a73a77"
        ),
        (
            algorithm: "sha1", bits: "80", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "110afbce59ad61c0ec28ed14"
        ),
        (
            algorithm: "sha1", bits: "160", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "1114fbce59ad61c0ec28ed14406c1432c9793b57adec"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "120ae96d0371442d2a8eae11"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "1214e96d0371442d2a8eae119eefe1d8710e8c59562a"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "1220e96d0371442d2a8eae119eefe1d8710e8c59562a8fe70adcba25322de2a0f854"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "130ac41a063c611e882714d8"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "1314c41a063c611e882714d8424f4edd0009b4b7c525"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "1320c41a063c611e882714d8424f4edd0009b4b7c525f132446af45780bfe4ff9980"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash:
                "1340c41a063c611e882714d8424f4edd0009b4b7c525f132446af45780bfe4ff998001c6479f48ae3289ea40a876a16e33c9d6b90201a307549938e33beb5e73e9f0"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "140acc1a0acb60f17e825b4c"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "1414cc1a0acb60f17e825b4c41b1eff84fc42cf5dd2b"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash: "1420cc1a0acb60f17e825b4c41b1eff84fc42cf5dd2b4468a47d0e8053ff55a4e7aa"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "3647da9e56b1afffa9921ce9a9caaa51ac8a166706",
            multihash:
                "1440cc1a0acb60f17e825b4c41b1eff84fc42cf5dd2b4468a47d0e8053ff55a4e7aa95e00fae8fc8eaf241a1027275c8243b8a7341b7fbd1878ac35eb0a8e174b46e"
        ),
        (
            algorithm: "sha1", bits: "80", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "110a92094dd3eed1997cfa0d"
        ),
        (
            algorithm: "sha1", bits: "160", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "111492094dd3eed1997cfa0d4c9743c6c03d9ab9fee5"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "120a4341e7832a0625d57845"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "12144341e7832a0625d578451df1d6f07b36fdec4505"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "12204341e7832a0625d578451df1d6f07b36fdec4505484ede11615893a5d1162bdc"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "130a6f558362cb0c867f40bd"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "13146f558362cb0c867f40bd8fda18d004a14206d024"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "13206f558362cb0c867f40bd8fda18d004a14206d0243879c1ff80dc70c89d60eaf8"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash:
                "13406f558362cb0c867f40bd8fda18d004a14206d0243879c1ff80dc70c89d60eaf8ed19a96c6c5a0d38597a027a30856a6be4c25e5ce8473fa9ebd6f834c4bbdab3"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "140ad4d533c326e157fbc53e"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "1414d4d533c326e157fbc53e52f4dfa219d67e998a8f"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash: "1420d4d533c326e157fbc53e52f4dfa219d67e998a8f54e0a899b1fd32b07be18635"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "f7f96cf1e683e6f7e752dab4613a1b1225889bb15d",
            multihash:
                "1440d4d533c326e157fbc53e52f4dfa219d67e998a8f54e0a899b1fd32b07be18635f32cd8e9152282a9d52ce647ff4add11adffeecde5b3b00400fac68a0807e95f"
        ),
        (
            algorithm: "sha1", bits: "80", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "110a6cc39b02d728945459e0"
        ),
        (
            algorithm: "sha1", bits: "160", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "11146cc39b02d728945459e08e4a638b1d1bc3648e6e"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "120ac94ab4830a2f1eef4e37"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "1214c94ab4830a2f1eef4e378a499d91f09f1a909a24"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "1220c94ab4830a2f1eef4e378a499d91f09f1a909a249c48b27d3366ee6d97c2fc2e"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "130a763993febeab714f75ca"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "1314763993febeab714f75ca06511c52ddb69da2faaf"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "1320763993febeab714f75ca06511c52ddb69da2faaf6654a1e9a9dc545c639ee2c9"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash:
                "1340763993febeab714f75ca06511c52ddb69da2faaf6654a1e9a9dc545c639ee2c964deddf5b0a1aaae5b32d2c445b37278a1ba1de3948033012dbc4278dbba0ea0"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "140a459506c8c26789f203ac"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "1414459506c8c26789f203acd827c84bf1c68c821f80"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash: "1420459506c8c26789f203acd827c84bf1c68c821f800a1b977306d574d9c5fcc637"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "8fc7992293335bb2e2f6d962f66734a652e495022e",
            multihash:
                "1440459506c8c26789f203acd827c84bf1c68c821f800a1b977306d574d9c5fcc637eb2a4255742d3a6b247bfc188983a633e5074e98fcaee4c5f43fe211cc256891"
        ),
        (
            algorithm: "sha1", bits: "80", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "110a420ce85fe6648e367114"
        ),
        (
            algorithm: "sha1", bits: "160", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "1114420ce85fe6648e367114f073ae09581d2d5c92da"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "120a2efcc65fc37070bb5973"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "12142efcc65fc37070bb5973df7a99bbb2bc19a7dfe0"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "12202efcc65fc37070bb5973df7a99bbb2bc19a7dfe05c448dd2e052ace46df9a80b"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "130a6276d25a66a2731c3c2e"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "13146276d25a66a2731c3c2eb1def1b0cdd077f2d82d"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "13206276d25a66a2731c3c2eb1def1b0cdd077f2d82d0318e43746c928e49ca2026a"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash:
                "13406276d25a66a2731c3c2eb1def1b0cdd077f2d82d0318e43746c928e49ca2026a5f8e118393f46993e6e959212318a4c6fae8e66f8430b700097cd6192e8c3535"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "140aedf3dbcb6a229c00d6aa"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "1414edf3dbcb6a229c00d6aa35e9751d9609ab71d951"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash: "1420edf3dbcb6a229c00d6aa35e9751d9609ab71d95148a28fb8fc9f3bf26df4c9ed"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "a244e61f124cd3590173e2809b1177ca611ff15e9a",
            multihash:
                "1440edf3dbcb6a229c00d6aa35e9751d9609ab71d95148a28fb8fc9f3bf26df4c9eddcd4297a520f91d7a02972903197c92c2dc5e1f886a9d497a2c82a4d10d06307"
        ),
        (
            algorithm: "sha1", bits: "80", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "110a0987a972a0122c7f0788"
        ),
        (
            algorithm: "sha1", bits: "160", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "11140987a972a0122c7f0788fe35b3505adb2da889e7"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "120addc3fcda5d7fc1f3c48a"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "1214ddc3fcda5d7fc1f3c48a284aca1b5440b54ee8dd"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "1220ddc3fcda5d7fc1f3c48a284aca1b5440b54ee8dd1e2c4dbb7bcb77db696f71e2"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "130a3deb07d2b0fe9891149e"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "13143deb07d2b0fe9891149e5681625ba78c2b30777c"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "13203deb07d2b0fe9891149e5681625ba78c2b30777cfffb543d5672f83aa2b6a03e"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash:
                "13403deb07d2b0fe9891149e5681625ba78c2b30777cfffb543d5672f83aa2b6a03e61c72738ec3d1eb61df1d1691da9fe516b8f9668e14a38c0b620d1d7eac0e3bb"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "140acdbfc227b85b786ec47f"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "1414cdbfc227b85b786ec47f75bea9c19e7caaf70b8c"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash: "1420cdbfc227b85b786ec47f75bea9c19e7caaf70b8cbe61e080856df5650d4b2b6b"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "8a461139f4d5078f6ff77af4fbc1befc775c0e0cb0",
            multihash:
                "1440cdbfc227b85b786ec47f75bea9c19e7caaf70b8cbe61e080856df5650d4b2b6b671ce9f68d9c66b6a1a8de15d3c8d5978a737f21e4d89e74d598d9ae87c4694e"
        ),
        (
            algorithm: "sha1", bits: "80", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "110a69a49abea40f7b33515d"
        ),
        (
            algorithm: "sha1", bits: "160", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "111469a49abea40f7b33515d2c78ac3ad27c9659ddf8"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "120acf9991b40b273af2ef5c"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "1214cf9991b40b273af2ef5c3d41668539d7adfc2637"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "1220cf9991b40b273af2ef5c3d41668539d7adfc2637dad7102593ea36b200355124"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "130ad9bb89326f0d56c5d5c5"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "1314d9bb89326f0d56c5d5c5ee71c1a0cf2138db6146"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "1320d9bb89326f0d56c5d5c5ee71c1a0cf2138db6146126ea1f135bbebf136d29bf9"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash:
                "1340d9bb89326f0d56c5d5c5ee71c1a0cf2138db6146126ea1f135bbebf136d29bf9658bfc9ac3a5ac096c60842f1008d6162cefc4bbeabc413f5e5d0ceff94fa344"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "140acfe85f27f18a9b31eeb9"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "1414cfe85f27f18a9b31eeb941b4fb385b22cb6cb745"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash: "1420cfe85f27f18a9b31eeb941b4fb385b22cb6cb7458ff3ecec239d551ecf8b4645"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "eaf89d6d7fa2ee40c7357e3d627cbd1338009570f5",
            multihash:
                "1440cfe85f27f18a9b31eeb941b4fb385b22cb6cb7458ff3ecec239d551ecf8b4645652877f27fc71fcc927866409143613bbd2586315c53538f6df06d720a4452cb"
        ),
        (
            algorithm: "sha1", bits: "80", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "110a0088dbe5cfcac4d78f2a"
        ),
        (
            algorithm: "sha1", bits: "160", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "11140088dbe5cfcac4d78f2a6b9f0078007b3141e7fc"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "120a8988c716536184426e96"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "12148988c716536184426e96e1714964c18a3ec7c3df"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "12208988c716536184426e96e1714964c18a3ec7c3df537d692b410728bf79e16868"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "130acf9b4008bfd4d617035c"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "1314cf9b4008bfd4d617035c5686cc21bfb9c5d09936"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "1320cf9b4008bfd4d617035c5686cc21bfb9c5d0993685868921758a369a2434f3b2"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash:
                "1340cf9b4008bfd4d617035c5686cc21bfb9c5d0993685868921758a369a2434f3b26e01124a0d42797f0a30bfca8db324798b857816e7f4637e55a4941c5974c351"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "140a51e7bd1d173316156e0c"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "141451e7bd1d173316156e0cfeb28186da1a6df4d542"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash: "142051e7bd1d173316156e0cfeb28186da1a6df4d542ad968b1c038a5f66c7eed65a"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "4c98758072933abd55fef35782cf37386db7ce4dce",
            multihash:
                "144051e7bd1d173316156e0cfeb28186da1a6df4d542ad968b1c038a5f66c7eed65add6465e79d51899db0278e11b3cbfa04821967feaa822eea22948b05c464cf93"
        ),
        (
            algorithm: "sha1", bits: "80", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "110a81aaff208d63691d5176"
        ),
        (
            algorithm: "sha1", bits: "160", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "111481aaff208d63691d5176d4aaf27aa59559385887"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "120af28cd11dcb49a131f9b8"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "1214f28cd11dcb49a131f9b8773035bdc2605a6c2d64"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "1220f28cd11dcb49a131f9b8773035bdc2605a6c2d6459782b987d08c5a8c1fd1814"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "130ab184c8ffda3791a2916a"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "1314b184c8ffda3791a2916a632e9f59aaf13e762667"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "1320b184c8ffda3791a2916a632e9f59aaf13e762667d5cd8f514cf38d90933a6c37"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash:
                "1340b184c8ffda3791a2916a632e9f59aaf13e762667d5cd8f514cf38d90933a6c3734fd4ab3b0b3e5a9a3701eb761beca3e01a5efcc2558077f7d301f3092e8abec"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "140a9b6b98749e9b038091fa"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "14149b6b98749e9b038091fa50532ee3267a67ed3607"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash: "14209b6b98749e9b038091fa50532ee3267a67ed3607438f7bf25b0c29295f806b10"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "79f28aba59f256868f8b7f868e75aedcab8b068be0",
            multihash:
                "14409b6b98749e9b038091fa50532ee3267a67ed3607438f7bf25b0c29295f806b10bc1a8ae13e8c23744672bf7628114a557ff28ee3e2ccc73b0759f4a98a7f86de"
        ),
        (
            algorithm: "sha1", bits: "80", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "110a9daeb5b0eb9d59501635"
        ),
        (
            algorithm: "sha1", bits: "160", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "11149daeb5b0eb9d59501635b20b1f1c2e12e481adae"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "120a9eeab680ac6bd543dfc3"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "12149eeab680ac6bd543dfc34b3d90aa6e5a4bb27961"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "12209eeab680ac6bd543dfc34b3d90aa6e5a4bb27961faef39680e3f549516dc948e"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "130a621cb2040060c6024e57"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "1314621cb2040060c6024e57e9d68abb8c41ec55b327"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "1320621cb2040060c6024e57e9d68abb8c41ec55b327b9f2e5865e65a1ff8609d00f"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash:
                "1340621cb2040060c6024e57e9d68abb8c41ec55b327b9f2e5865e65a1ff8609d00fb27c419ec9a68978b83652a44f47d7866ba57717a0203ba117eb01cf449c1ec7"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "140a888e30d43451afe15d97"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "1414888e30d43451afe15d9711d6459f069471835705"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash: "1420888e30d43451afe15d9711d6459f06947183570586b38b20723e7f02fa478c13"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "fe2bd8cb6ad52767eae6fbd354ceb3e72e8f55669b",
            multihash:
                "1440888e30d43451afe15d9711d6459f06947183570586b38b20723e7f02fa478c13ac363e07528ee22e1319a1cbfcb329e6f687acc23d650ff599abbc9c828b5efd"
        ),
        (
            algorithm: "sha1", bits: "80", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "110a92ecbf3facfa79118d36"
        ),
        (
            algorithm: "sha1", bits: "160", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "111492ecbf3facfa79118d3611e7e406f4d4d6242f1c"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "120af000499860abadcf61ed"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "1214f000499860abadcf61edec7453c0a9112c5083ed"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "1220f000499860abadcf61edec7453c0a9112c5083ed9fa27c350ded0a8ed27ea236"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "130abf1acd49f89a81fa6099"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "1314bf1acd49f89a81fa60995a03709ae483850abb0e"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "1320bf1acd49f89a81fa60995a03709ae483850abb0e1f168e75a02db9d0b9ad58c0"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash:
                "1340bf1acd49f89a81fa60995a03709ae483850abb0e1f168e75a02db9d0b9ad58c0053202cf4dff68112835191b07067693d65d73727db81a0624ccffdd92a87fa2"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "140a39e3d69186b5252fd69c"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "141439e3d69186b5252fd69cff646c2f12eaa3f63772"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash: "142039e3d69186b5252fd69cff646c2f12eaa3f63772e6062a1657dc3af9623e2e57"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "7f0bb0ac4dd2404c097dade4c8b2292ae2419b7a70",
            multihash:
                "144039e3d69186b5252fd69cff646c2f12eaa3f63772e6062a1657dc3af9623e2e57bf3c74b7d1f96f4a4d0c8b85cd7b2bfdcb0f9712795726378d9ccccd192d8086"
        ),
        (
            algorithm: "sha1", bits: "80", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "110af7165b40913fe411ae00"
        ),
        (
            algorithm: "sha1", bits: "160", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "1114f7165b40913fe411ae0012435ac0ac9491cd68e9"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "120a88af105116b3006c14cc"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "121488af105116b3006c14cc9c38e5bd9049386dc827"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "122088af105116b3006c14cc9c38e5bd9049386dc82785fbb15a69a907fb9da8c831"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "130a7d185f12ce400ce8d3e5"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "13147d185f12ce400ce8d3e55ef3a126e8f6ed89dd5d"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "13207d185f12ce400ce8d3e55ef3a126e8f6ed89dd5da8e296e84b7da105bf28d643"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash:
                "13407d185f12ce400ce8d3e55ef3a126e8f6ed89dd5da8e296e84b7da105bf28d6437eec013a5e760065221791ea7a2d4a8458d7d454b9acbe7418db05dce0b5412c"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "140a9b596b0ed3e0fbe6be32"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "14149b596b0ed3e0fbe6be325b26e94f83a99119c95a"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash: "14209b596b0ed3e0fbe6be325b26e94f83a99119c95ad6b376115cc228f94087a4ca"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "b07cb1832e7e253f3eb7cf4029a82d881217d3c02f",
            multihash:
                "14409b596b0ed3e0fbe6be325b26e94f83a99119c95ad6b376115cc228f94087a4ca68059c964164757f76e08c89e5d521c256c00b21af19b0c8a5a0484ec9db65eb"
        ),
        (
            algorithm: "sha1", bits: "80", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "110a22b6cc60a0782c145dc9"
        ),
        (
            algorithm: "sha1", bits: "160", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "111422b6cc60a0782c145dc925d2c3d02e722d564fe9"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "120a66b0aa772b7562dbf534"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "121466b0aa772b7562dbf534ae653ab686735baf7ed3"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "122066b0aa772b7562dbf534ae653ab686735baf7ed3614caad398868d8e2b84a289"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "130a8cad36ff75b598968e94"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "13148cad36ff75b598968e94c5152800fe2a9d6de6cc"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "13208cad36ff75b598968e94c5152800fe2a9d6de6cca92617c0fb331bf9539a80f7"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash:
                "13408cad36ff75b598968e94c5152800fe2a9d6de6cca92617c0fb331bf9539a80f7b55de98fda8708dd771fb359ef72ce3794440d104ef7bef501ddb01735d2855f"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "140a28b681db6b5674389e1b"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "141428b681db6b5674389e1be7a410887260ecc8169e"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash: "142028b681db6b5674389e1be7a410887260ecc8169effa342907ee1294d7a13cd20"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "29a53e335621740c0fbc8dd85970fa0311b0d58e25",
            multihash:
                "144028b681db6b5674389e1be7a410887260ecc8169effa342907ee1294d7a13cd203ece5816b9bc3866e10d1c9d5be00e4e6ad05482e6b6b3e76e6f8b669a3aaef3"
        ),
        (
            algorithm: "sha1", bits: "80", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "110ac80fba1987567ea9f3ff"
        ),
        (
            algorithm: "sha1", bits: "160", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "1114c80fba1987567ea9f3ff4a587da9a537a3655f46"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "120a4577ee99ca45b00bae3a"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "12144577ee99ca45b00bae3afe7f3a50cbafb109ef1a"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "12204577ee99ca45b00bae3afe7f3a50cbafb109ef1a165ec698cb29a62daf1addab"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "130ac039899eef738c417dcf"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "1314c039899eef738c417dcf0b59358c4cf027496458"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "1320c039899eef738c417dcf0b59358c4cf02749645893734eda05871f7ccbe89064"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash:
                "1340c039899eef738c417dcf0b59358c4cf02749645893734eda05871f7ccbe89064dbf95c96edc6245c54ec5601f44afcea126d0ba66f0a5423ab7ac2e91e144897"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "140a6fd4a8be27a828434ce7"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "14146fd4a8be27a828434ce7b0935a50df11dcc43143"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash: "14206fd4a8be27a828434ce7b0935a50df11dcc431437cace07f460d22649745ce1a"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "f6896cafdd752d9615f22017dfce1a281fb0598038",
            multihash:
                "14406fd4a8be27a828434ce7b0935a50df11dcc431437cace07f460d22649745ce1abfc1d65a361e9f3e0852cb4f425c7c9427328499d2a4e6999b35f05e37195908"
        ),
        (
            algorithm: "sha1", bits: "80", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "110abd23220dec583addf275"
        ),
        (
            algorithm: "sha1", bits: "160", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "1114bd23220dec583addf27599f0dc3a53c2a1dcf806"
        ),
        (
            algorithm: "sha2-256", bits: "80", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "120af6a17055da6921b92a23"
        ),
        (
            algorithm: "sha2-256", bits: "160", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "1214f6a17055da6921b92a23f6fa4e2434160dec6ccf"
        ),
        (
            algorithm: "sha2-256", bits: "256", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "1220f6a17055da6921b92a23f6fa4e2434160dec6ccfe1ce5979415ccf6467a8d8ac"
        ),
        (
            algorithm: "sha2-512", bits: "80", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "130aaeeaaf6cbfe3b61b01f4"
        ),
        (
            algorithm: "sha2-512", bits: "160", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "1314aeeaaf6cbfe3b61b01f4dabee8cad7c3063b3ae8"
        ),
        (
            algorithm: "sha2-512", bits: "256", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "1320aeeaaf6cbfe3b61b01f4dabee8cad7c3063b3ae80e0f346c5212699c5bf369fb"
        ),
        (
            algorithm: "sha2-512", bits: "512", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash:
                "1340aeeaaf6cbfe3b61b01f4dabee8cad7c3063b3ae80e0f346c5212699c5bf369fb57143a7b4a980fe5bfdf0420da7ff2667530a16f2c2d96c7c5c858584b151b49"
        ),
        (
            algorithm: "sha3-512", bits: "80", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "140a5541d5148a03e5ba0a91"
        ),
        (
            algorithm: "sha3-512", bits: "160", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "14145541d5148a03e5ba0a916b8f91175219b15e260f"
        ),
        (
            algorithm: "sha3-512", bits: "256", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash: "14205541d5148a03e5ba0a916b8f91175219b15e260f0b26c4fdfa24b24bc54ac640"
        ),
        (
            algorithm: "sha3-512", bits: "512", input: "444a8ca03cce5b47991b3baf807f4eaa1d8175a72e",
            multihash:
                "14405541d5148a03e5ba0a916b8f91175219b15e260f0b26c4fdfa24b24bc54ac64045ea813027252d05c43a3f0d5def492937a33fbc04c3eaf7ad61af5e16bb7c3f"
        ),
    ]

}
