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

import Testing

@testable import Multibase

let BasicTests = [
    "base2": "001111001011001010111001100100000011011010110000101101110011010010010000000100001",
    "base8": "7362625631006654133464440102",
    "base10": "9573277761329450583662625",
    "base16": "f796573206d616e692021",
    "base16upper": "F796573206D616E692021",
    "base32": "bpfsxgidnmfxgsibb",
    "base32upper": "BPFSXGIDNMFXGSIBB",
    "base32hex": "vf5in683dc5n6i811",
    "base32hexupper": "VF5IN683DC5N6I811",
    "base32pad": "cpfsxgidnmfxgsibb",
    "base32padupper": "CPFSXGIDNMFXGSIBB",
    "base32hexpad": "tf5in683dc5n6i811",
    "base32hexpadupper": "TF5IN683DC5N6I811",
    "base32z": "hxf1zgedpcfzg1ebb",
    "base36": "k2lcpzo5yikidynfl",
    "base36upper": "K2LCPZO5YIKIDYNFL",
    "base58flickr": "Z7Pznk19XTTzBtx",
    "base58btc": "z7paNL19xttacUY",
    "base64": "meWVzIG1hbmkgIQ",
    "base64pad": "MeWVzIG1hbmkgIQ==",
    "base64url": "ueWVzIG1hbmkgIQ",
    "base64urlpad": "UeWVzIG1hbmkgIQ==",
]

let CaseInsensitivityTests = [
    "base16": "f68656c6c6f20776F726C64",
    "base16upper": "F68656c6c6f20776F726C64",
    "base32": "bnbswy3dpeB3W64TMMQ",
    "base32upper": "Bnbswy3dpeB3W64TMMQ",
    "base32hex": "vd1imor3f41RMUSJCCG",
    "base32hexupper": "Vd1imor3f41RMUSJCCG",
    "base32pad": "cnbswy3dpeB3W64TMMQ======",
    "base32padupper": "Cnbswy3dpeB3W64TMMQ======",
    "base32hexpad": "td1imor3f41RMUSJCCG======",
    "base32hexpadupper": "Td1imor3f41RMUSJCCG======",
    "base36": "kfUvrsIvVnfRbjWaJo",
    "base36upper": "KfUVrSIVVnFRbJWAJo",
]

let CaseLeadingZero = [
    "base2": "00000000001111001011001010111001100100000011011010110000101101110011010010010000000100001",
    "base8": "7000745453462015530267151100204",
    "base10": "90573277761329450583662625",
    "base16": "f00796573206d616e692021",
    "base16upper": "F00796573206D616E692021",
    "base32": "bab4wk4zanvqw42jaee",
    "base32upper": "BAB4WK4ZANVQW42JAEE",
    "base32hex": "v01smasp0dlgmsq9044",
    "base32hexupper": "V01SMASP0DLGMSQ9044",
    "base32pad": "cab4wk4zanvqw42jaee======",
    "base32padupper": "CAB4WK4ZANVQW42JAEE======",
    "base32hexpad": "t01smasp0dlgmsq9044======",
    "base32hexpadupper": "T01SMASP0DLGMSQ9044======",
    "base32z": "hybhskh3ypiosh4jyrr",
    "base36": "k02lcpzo5yikidynfl",
    "base36upper": "K02LCPZO5YIKIDYNFL",
    "base58flickr": "Z17Pznk19XTTzBtx",
    "base58btc": "z17paNL19xttacUY",
    "base64": "mAHllcyBtYW5pICE",
    "base64pad": "MAHllcyBtYW5pICE=",
    "base64url": "uAHllcyBtYW5pICE",
    "base64urlpad": "UAHllcyBtYW5pICE=",
]

let CaseTwoLeadingZeros = [
    "base2": "0000000000000000001111001011001010111001100100000011011010110000101101110011010010010000000100001",
    "base8": "700000171312714403326055632220041",
    "base10": "900573277761329450583662625",
    "base16": "f0000796573206d616e692021",
    "base16upper": "F0000796573206D616E692021",
    "base32": "baaahszltebwwc3tjeaqq",
    "base32upper": "BAAAHSZLTEBWWC3TJEAQQ",
    "base32hex": "v0007ipbj41mm2rj940gg",
    "base32hexupper": "V0007IPBJ41MM2RJ940GG",
    "base32pad": "caaahszltebwwc3tjeaqq====",
    "base32padupper": "CAAAHSZLTEBWWC3TJEAQQ====",
    "base32hexpad": "t0007ipbj41mm2rj940gg====",
    "base32hexpadupper": "T0007IPBJ41MM2RJ940GG====",
    "base32z": "hyyy813murbssn5ujryoo",
    "base36": "k002lcpzo5yikidynfl",
    "base36upper": "K002LCPZO5YIKIDYNFL",
    "base58flickr": "Z117Pznk19XTTzBtx",
    "base58btc": "z117paNL19xttacUY",
    "base64": "mAAB5ZXMgbWFuaSAh",
    "base64pad": "MAAB5ZXMgbWFuaSAh",
    "base64url": "uAAB5ZXMgbWFuaSAh",
    "base64urlpad": "UAAB5ZXMgbWFuaSAh",
]

@Suite("Multibase Tests")
struct MultibaseTests {

    @Test func testBinary() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base2
        #expect(testString.encodeASCII(base: .base2) == BasicTests["base2"])  //BasicTests["base2"])
        #expect(
            testString2.encodeUTF8(base: .base2)
                == "001000100011001010110001101100101011011100111010001110010011000010110110001101001011110100110010100100000011001010111011001100101011100100111100101110100011010000110100101101110011001110010000100100001"
        )

        let multibase1 = try BaseEncoding.decodeIntoString(
            "001111001011001010111001100100000011011010110000101101110011010010010000000100001"
        )
        #expect(multibase1.base == .base2)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString(
            "001000100011001010110001101100101011011100111010001110010011000010110110001101001011110100110010100100000011001010111011001100101011100100111100101110100011010000110100101101110011001110010000100100001"
        )
        #expect(multibase2.base == .base2)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testOctal() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base 8 (3 bits per char)
        #expect(testString.encodeUTF8(base: .base8) == BasicTests["base8"])
        #expect(
            testString2.encodeUTF8(base: .base8)
                == "72106254331267164344605543227514510062566312711713506415133463441102"
        )

        let multibase1 = try BaseEncoding.decodeIntoString("7362625631006654133464440102")
        #expect(multibase1.base == .base8)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString(
            "72106254331267164344605543227514510062566312711713506415133463441102"
        )
        #expect(multibase2.base == .base8)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testDecimal() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        // Base 10 (4 bits per character) (works with the Base58 algo using BigInt)
        #expect(testString.encodeUTF8(base: .base10) == BasicTests["base10"])
        #expect(
            testString2.encodeUTF8(base: .base10)
                == "9429328951066508984658627669258025763026247056774804621697313"
        )

        let multibase1 = try BaseEncoding.decodeIntoString("9573277761329450583662625")
        #expect(multibase1.base == .base10)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString(
            "9429328951066508984658627669258025763026247056774804621697313"
        )
        #expect(multibase2.base == .base10)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testHex() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        // Base 16
        #expect(testString.encodeUTF8(base: .base16) == BasicTests["base16"])
        #expect(testString2.encodeUTF8(base: .base16) == "f446563656e7472616c697a652065766572797468696e672121")

        let multibase1 = try BaseEncoding.decodeIntoString("f796573206d616e692021")
        #expect(multibase1.base == .base16)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("f446563656e7472616c697A652065766572797468696e672121")
        #expect(multibase2.base == .base16)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testHexUpper() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        // Base 16
        #expect(testString.encodeUTF8(base: .base16Upper) == BasicTests["base16upper"])
        #expect(
            testString2.encodeUTF8(base: .base16Upper)
                == "F446563656E7472616C697A652065766572797468696E672121"
        )

        let multibase1 = try BaseEncoding.decodeIntoString("F796573206d616e692021")
        #expect(multibase1.base == .base16Upper)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("F446563656e7472616c697a652065766572797468696e672121")
        #expect(multibase2.base == .base16Upper)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase32() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base32
        #expect(testString.encodeUTF8(base: .base32) == BasicTests["base32"])
        #expect(testString2.encodeUTF8(base: .base32) == "birswgzloorzgc3djpjssazlwmvzhs5dinfxgoijb")

        let multibase1 = try BaseEncoding.decodeIntoString("bpfsxgidnmfxgsibb")
        #expect(multibase1.base == .base32)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("birswgzloorzgc3djpjssazlwmvzhs5dinfxgoijb")
        #expect(multibase2.base == .base32)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase32Upper() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base32Upper
        #expect(testString.encodeUTF8(base: .base32Upper) == BasicTests["base32upper"])
        #expect(testString2.encodeUTF8(base: .base32Upper) == "BIRSWGZLOORZGC3DJPJSSAZLWMVZHS5DINFXGOIJB")

        let multibase1 = try BaseEncoding.decodeIntoString("BPFSXGIDNMFXGSIBB")
        #expect(multibase1.base == .base32Upper)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("BIRSWGZLOORZGC3DJPJSSAZLWMVZHS5DINFXGOIJB")
        #expect(multibase2.base == .base32Upper)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase32Pad() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base32Pad
        #expect(testString.encodeUTF8(base: .base32Pad) == BasicTests["base32pad"])
        #expect(testString2.encodeUTF8(base: .base32Pad) == "cirswgzloorzgc3djpjssazlwmvzhs5dinfxgoijb")

        let multibase1 = try BaseEncoding.decodeIntoString("cpfsxgidnmfxgsibb")
        #expect(multibase1.base == .base32Pad)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("cirswgzloorzgc3djpjssazlwmvzhs5dinfxgoijb")
        #expect(multibase2.base == .base32Pad)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase32PadUpper() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base32PadUpper
        #expect(testString.encodeUTF8(base: .base32PadUpper) == BasicTests["base32padupper"])
        #expect(testString2.encodeUTF8(base: .base32PadUpper) == "CIRSWGZLOORZGC3DJPJSSAZLWMVZHS5DINFXGOIJB")

        let multibase1 = try BaseEncoding.decodeIntoString("CPFSXGIDNMFXGSIBB")
        #expect(multibase1.base == .base32PadUpper)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("CIRSWGZLOORZGC3DJPJSSAZLWMVZHS5DINFXGOIJB")
        #expect(multibase2.base == .base32PadUpper)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase32Hex() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base32PadUpper
        #expect(testString.encodeUTF8(base: .base32Hex) == BasicTests["base32hex"])
        #expect(testString2.encodeUTF8(base: .base32Hex) == "v8him6pbeehp62r39f9ii0pbmclp7it38d5n6e891")

        let multibase1 = try BaseEncoding.decodeIntoString("vf5in683dc5n6i811")
        #expect(multibase1.base == .base32Hex)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("v8him6pbeehp62r39f9ii0pbmclp7it38d5n6e891")
        #expect(multibase2.base == .base32Hex)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase32HexUpper() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base32PadUpper
        #expect(testString.encodeUTF8(base: .base32HexUpper) == BasicTests["base32hexupper"])
        #expect(testString2.encodeUTF8(base: .base32HexUpper) == "V8HIM6PBEEHP62R39F9II0PBMCLP7IT38D5N6E891")

        let multibase1 = try BaseEncoding.decodeIntoString("VF5IN683DC5N6I811")
        #expect(multibase1.base == .base32HexUpper)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("V8HIM6PBEEHP62R39F9II0PBMCLP7IT38D5N6E891")
        #expect(multibase2.base == .base32HexUpper)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase32HexPad() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base32PadUpper
        #expect(testString.encodeUTF8(base: .base32HexPad) == BasicTests["base32hexpad"])
        #expect(testString2.encodeUTF8(base: .base32HexPad) == "t8him6pbeehp62r39f9ii0pbmclp7it38d5n6e891")

        let multibase1 = try BaseEncoding.decodeIntoString("tf5in683dc5n6i811")
        #expect(multibase1.base == .base32HexPad)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("t8him6pbeehp62r39f9ii0pbmclp7it38d5n6e891")
        #expect(multibase2.base == .base32HexPad)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase32HexPadUpper() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base32PadUpper
        #expect(testString.encodeUTF8(base: .base32HexPadUpper) == BasicTests["base32hexpadupper"])
        #expect(testString2.encodeUTF8(base: .base32HexPadUpper) == "T8HIM6PBEEHP62R39F9II0PBMCLP7IT38D5N6E891")

        let multibase1 = try BaseEncoding.decodeIntoString("TF5IN683DC5N6I811")
        #expect(multibase1.base == .base32HexPadUpper)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("T8HIM6PBEEHP62R39F9II0PBMCLP7IT38D5N6E891")
        #expect(multibase2.base == .base32HexPadUpper)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase32z() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base32z
        #expect(testString.encodeUTF8(base: .base32z) == BasicTests["base32z"])
        #expect(testString2.encodeUTF8(base: .base32z) == "het1sg3mqqt3gn5djxj11y3msci3817depfzgqejb")

        let multibase1 = try BaseEncoding.decodeIntoString("hxf1zgedpcfzg1ebb")
        #expect(multibase1.base == .base32z)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("het1sg3mqqt3gn5djxj11y3msci3817depfzgqejb")
        #expect(multibase2.base == .base32z)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase36() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base36
        #expect(testString.encodeUTF8(base: .base36) == BasicTests["base36"])
        #expect(testString2.encodeUTF8(base: .base36) == "k343ixo7d49hqj1ium15pgy1wzww5fxrid21td7l")

        let multibase1 = try BaseEncoding.decodeIntoString("k2lcpzo5yikidynFl")
        #expect(multibase1.base == .base36)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("k343ixo7d49hqj1ium15pgy1wzww5fxrid21td7l")
        #expect(multibase2.base == .base36)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase36Upper() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base36 Upper
        #expect(testString.encodeUTF8(base: .base36Upper) == BasicTests["base36upper"])
        #expect(testString2.encodeUTF8(base: .base36Upper) == "K343IXO7D49HQJ1IUM15PGY1WZWW5FXRID21TD7L")

        let multibase1 = try BaseEncoding.decodeIntoString("K2LCPZO5YIKIDYNfL")
        #expect(multibase1.base == .base36Upper)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("K343IXO7D49HQJ1IUM15PGY1WZWW5FXRID21TD7L")
        #expect(multibase2.base == .base36Upper)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBTCBase58() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"
        //Base58 BTC
        #expect(testString.encodeUTF8(base: .base58btc) == BasicTests["base58btc"])
        #expect(testString2.encodeUTF8(base: .base58btc) == "zUXE7GvtEk8XTXs1GF8HSGbVA9FCX9SEBPe")

        let multibase1 = try BaseEncoding.decodeIntoString("z7paNL19xttacUY")
        #expect(multibase1.base == .base58btc)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("zUXE7GvtEk8XTXs1GF8HSGbVA9FCX9SEBPe")
        #expect(multibase2.base == .base58btc)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testFlickrBase58() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"
        //Base58 BTC
        #expect(testString.encodeUTF8(base: .base58flickr) == BasicTests["base58flickr"])
        #expect(testString2.encodeUTF8(base: .base58flickr) == "Ztwe7gVTeK8wswS1gf8hrgAua9fcw9reboD")

        let multibase1 = try BaseEncoding.decodeIntoString("Z7Pznk19XTTzBtx")
        #expect(multibase1.base == .base58flickr)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("Ztwe7gVTeK8wswS1gf8hrgAua9fcw9reboD")
        #expect(multibase2.base == .base58flickr)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase64() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base64
        #expect(testString.encodeUTF8(base: .base64) == BasicTests["base64"])
        #expect(testString2.encodeUTF8(base: .base64) == "mRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ")

        let multibase1 = try BaseEncoding.decodeIntoString("meWVzIG1hbmkgIQ")
        #expect(multibase1.base == .base64)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("mRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ")
        #expect(multibase2.base == .base64)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase64Pad() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base64
        #expect(testString.encodeUTF8(base: .base64Pad) == BasicTests["base64pad"])
        #expect(testString2.encodeUTF8(base: .base64Pad) == "MRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ==")

        let multibase1 = try BaseEncoding.decodeIntoString("MeWVzIG1hbmkgIQ==")
        #expect(multibase1.base == .base64Pad)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("MRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ==")
        #expect(multibase2.base == .base64Pad)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase64Url() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base64
        #expect(testString.encodeUTF8(base: .base64Url) == BasicTests["base64url"])
        #expect(testString2.encodeUTF8(base: .base64Url) == "uRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ")

        let multibase1 = try BaseEncoding.decodeIntoString("ueWVzIG1hbmkgIQ")
        #expect(multibase1.base == .base64Url)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("uRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ==")
        #expect(multibase2.base == .base64Url)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase64UrlPad() throws {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base64
        #expect(testString.encodeUTF8(base: .base64UrlPad) == BasicTests["base64urlpad"])
        #expect(testString2.encodeUTF8(base: .base64UrlPad) == "URGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ==")

        let multibase1 = try BaseEncoding.decodeIntoString("UeWVzIG1hbmkgIQ==")
        #expect(multibase1.base == .base64UrlPad)
        #expect(multibase1.string == "yes mani !")

        let multibase2 = try BaseEncoding.decodeIntoString("URGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ==")
        #expect(multibase2.base == .base64UrlPad)
        #expect(multibase2.string == "Decentralize everything!!")
    }

    @Test func testBase64CrazyStrings() throws {
        let tests = [
            "f": "mZg",
            "fo": "mZm8",
            "foo": "mZm9v",
            "foob": "mZm9vYg",
            "fooba": "mZm9vYmE",
            "foobar": "mZm9vYmFy",
            "÷ïÿ": "mw7fDr8O/",
            "÷ïÿ🥰÷ïÿ😎🥶🤯": "mw7fDr8O/8J+lsMO3w6/Dv/CfmI7wn6W28J+krw",
        ]

        for test in tests.sorted(by: { $0.key.count < $1.key.count }) {
            let encoded = test.key.encode(as: .base64, using: .utf8)
            //print("\"\(test.key)\".encode(as: .base64, using: .utf8) => \(encoded)")
            #expect(encoded == test.value)

            let decoded = try BaseEncoding.decodeIntoString(encoded)
            #expect(decoded.base == .base64)
            #expect(decoded.string == test.key)
        }
    }

    @Test func testBase64PadCrazyStrings() throws {
        let tests = [
            "f": "MZg==",
            "fo": "MZm8=",
            "foo": "MZm9v",
            "foob": "MZm9vYg==",
            "fooba": "MZm9vYmE=",
            "foobar": "MZm9vYmFy",
            "÷ïÿ": "Mw7fDr8O/",
            "÷ïÿ🥰÷ïÿ😎🥶🤯": "Mw7fDr8O/8J+lsMO3w6/Dv/CfmI7wn6W28J+krw==",
        ]

        for test in tests.sorted(by: { $0.key.count < $1.key.count }) {
            let encoded = test.key.encode(as: .base64Pad, using: .utf8)
            //print("\"\(test.key)\".encode(as: .base64Pad, using: .utf8) => \(encoded)")
            #expect(encoded == test.value)

            let decoded = try BaseEncoding.decodeIntoString(encoded)
            #expect(decoded.base == .base64Pad)
            #expect(decoded.string == test.key)
        }
    }

    @Test func testBase64URLCrazyStrings() throws {
        let tests = [
            "f": "uZg",
            "fo": "uZm8",
            "foo": "uZm9v",
            "foob": "uZm9vYg",
            "fooba": "uZm9vYmE",
            "foobar": "uZm9vYmFy",
            "÷ïÿ": "uw7fDr8O_",
            "÷ïÿ🥰÷ïÿ😎🥶🤯": "uw7fDr8O_8J-lsMO3w6_Dv_CfmI7wn6W28J-krw",
        ]

        for test in tests.sorted(by: { $0.key.count < $1.key.count }) {
            let encoded = test.key.encode(as: .base64Url, using: .utf8)
            //print("\"\(test.key)\".encode(as: .base64Url, using: .utf8) => \(encoded)")
            #expect(encoded == test.value)

            let decoded = try BaseEncoding.decodeIntoString(encoded)
            #expect(decoded.base == .base64Url)
            #expect(decoded.string == test.key)
        }
    }

    @Test func testBase64URLPadCrazyStrings() throws {
        let tests = [
            "f": "UZg==",
            "fo": "UZm8=",
            "foo": "UZm9v",
            "foob": "UZm9vYg==",
            "fooba": "UZm9vYmE=",
            "foobar": "UZm9vYmFy",
            "÷ïÿ": "Uw7fDr8O_",
            "÷ïÿ🥰÷ïÿ😎🥶🤯": "Uw7fDr8O_8J-lsMO3w6_Dv_CfmI7wn6W28J-krw==",
        ]

        for test in tests.sorted(by: { $0.key.count < $1.key.count }) {
            let encoded = test.key.encode(as: .base64UrlPad, using: .utf8)
            //print("\"\(test.key)\".encode(as: .base64UrlPad, using: .utf8) => \(encoded)")
            #expect(encoded == test.value)

            let decoded = try BaseEncoding.decodeIntoString(encoded)
            #expect(decoded.base == .base64UrlPad)
            #expect(decoded.string == test.key)
        }
    }

    @Test func testBase32CrazyStrings() throws {
        let tests = [
            "f": "cmy======",
            "fo": "cmzxq====",
            "foo": "cmzxw6===",
            "foob": "cmzxw6yq=",
            "fooba": "cmzxw6ytb",
            "foobar": "cmzxw6ytboi======",
            "÷ïÿ": "cyo34hl6dx4======",
            "÷ïÿ🥰÷ïÿ😎🥶🤯": "cyo34hl6dx7yj7jnqyo34hl6dx7yj7geo6cp2lnxqt6sk6===",
        ]

        for test in tests.sorted(by: { $0.key.count < $1.key.count }) {
            let encoded = test.key.encode(as: .base32Pad, using: .utf8)
            //print("\"\(test.key)\".encode(as: .base32Pad, using: .utf8) => \(encoded)")
            #expect(encoded == test.value)

            let decoded = try BaseEncoding.decodeIntoString(encoded)
            #expect(decoded.base == .base32Pad)
            #expect(decoded.string == test.key)
        }
    }

    @Test func testUTF8() {
        //hello world
        //        let helloBytes:[UInt8] = [
        //            104,
        //            101,
        //            108,
        //            108,
        //            111,
        //            32,
        //            119,
        //            111,
        //            114,
        //            108,
        //            100,
        //          ]
        //
        //        let encoded = Array("hello world".utf8)
        //        print(encoded)
        //
        //        let decoded = String(bytes: helloBytes, encoding: .utf8)
        //        print(decoded)
        //
        //        let base16 = "hello world".encodeUTF8(base: .base16)
        //        let base16Test = "yes mani !".encodeUTF8(base: .base16Upper)
        //        print(base16)
        //        print(base16Test)
    }

    @Test func testBasicEncoding() {
        let testString = "yes mani !"
        let testString2 = "Decentralize everything!!"

        //Base2
        #expect(testString.encodeUTF8(base: .base2) == BasicTests["base2"])
        #expect(
            testString2.encodeUTF8(base: .base2)
                == "001000100011001010110001101100101011011100111010001110010011000010110110001101001011110100110010100100000011001010111011001100101011100100111100101110100011010000110100101101110011001110010000100100001"
        )

        // Base 8 (3 bits per character)
        #expect(testString.encodeUTF8(base: .base8) == BasicTests["base8"])
        #expect(
            testString2.encodeUTF8(base: .base8)
                == "72106254331267164344605543227514510062566312711713506415133463441102"
        )

        // Base 10
        #expect(testString.encodeUTF8(base: .base10) == BasicTests["base10"])
        #expect(
            testString2.encodeUTF8(base: .base10)
                == "9429328951066508984658627669258025763026247056774804621697313"
        )

        //Base16 lowercased
        #expect(testString.encodeUTF8(base: .base16) == BasicTests["base16"])
        #expect(testString2.encodeUTF8(base: .base16) == "f446563656e7472616c697a652065766572797468696e672121")

        //Base16 uppercased
        #expect(testString.encodeUTF8(base: .base16Upper) == BasicTests["base16upper"])
        #expect(
            testString2.encodeUTF8(base: .base16Upper) == "F446563656E7472616C697A652065766572797468696E672121"
        )

        //Base32
        #expect(testString.encodeUTF8(base: .base32) == BasicTests["base32"])
        #expect(testString2.encodeUTF8(base: .base32) == "birswgzloorzgc3djpjssazlwmvzhs5dinfxgoijb")

        //Base32Upper
        #expect(testString.encodeUTF8(base: .base32Upper) == BasicTests["base32upper"])
        #expect(testString2.encodeUTF8(base: .base32Upper) == "BIRSWGZLOORZGC3DJPJSSAZLWMVZHS5DINFXGOIJB")

        //Base32Hex
        #expect(testString.encodeUTF8(base: .base32Hex) == BasicTests["base32hex"])
        #expect(testString2.encodeUTF8(base: .base32Hex) == "v8him6pbeehp62r39f9ii0pbmclp7it38d5n6e891")

        //Base32HexUpper
        #expect(testString.encodeUTF8(base: .base32HexUpper) == BasicTests["base32hexupper"])
        #expect(testString2.encodeUTF8(base: .base32HexUpper) == "V8HIM6PBEEHP62R39F9II0PBMCLP7IT38D5N6E891")

        //Base32Pad
        #expect(testString.encodeUTF8(base: .base32Pad) == BasicTests["base32pad"])
        #expect(testString2.encodeUTF8(base: .base32Pad) == "cirswgzloorzgc3djpjssazlwmvzhs5dinfxgoijb")

        //Base32PadUpper
        #expect(testString.encodeUTF8(base: .base32PadUpper) == BasicTests["base32padupper"])
        #expect(testString2.encodeUTF8(base: .base32PadUpper) == "CIRSWGZLOORZGC3DJPJSSAZLWMVZHS5DINFXGOIJB")

        //Base32HexPad
        #expect(testString.encodeUTF8(base: .base32HexPad) == BasicTests["base32hexpad"])
        #expect(testString2.encodeUTF8(base: .base32HexPad) == "t8him6pbeehp62r39f9ii0pbmclp7it38d5n6e891")

        //Base32HexPadUpper
        #expect(testString.encodeUTF8(base: .base32HexPadUpper) == BasicTests["base32hexpadupper"])
        #expect(testString2.encodeUTF8(base: .base32HexPadUpper) == "T8HIM6PBEEHP62R39F9II0PBMCLP7IT38D5N6E891")

        //Base32z
        #expect(testString.encodeUTF8(base: .base32z) == BasicTests["base32z"])
        #expect(testString2.encodeUTF8(base: .base32z) == "het1sg3mqqt3gn5djxj11y3msci3817depfzgqejb")

        //Base36
        #expect(testString.encodeUTF8(base: .base36) == BasicTests["base36"])
        #expect(testString2.encodeUTF8(base: .base36) == "k343ixo7d49hqj1ium15pgy1wzww5fxrid21td7l")

        //Base36Upper
        #expect(testString.encodeUTF8(base: .base36Upper) == BasicTests["base36upper"])
        #expect(testString2.encodeUTF8(base: .base36Upper) == "K343IXO7D49HQJ1IUM15PGY1WZWW5FXRID21TD7L")

        //Base58 Flickr
        #expect(testString.encodeUTF8(base: .base58flickr) == BasicTests["base58flickr"])
        #expect(testString2.encodeUTF8(base: .base58flickr) == "Ztwe7gVTeK8wswS1gf8hrgAua9fcw9reboD")

        //Base58 BTC
        #expect(testString.encodeUTF8(base: .base58btc) == BasicTests["base58btc"])
        #expect(testString2.encodeUTF8(base: .base58btc) == "zUXE7GvtEk8XTXs1GF8HSGbVA9FCX9SEBPe")

        //Base64
        #expect(testString.encodeUTF8(base: .base64) == BasicTests["base64"])
        #expect(testString2.encodeUTF8(base: .base64) == "mRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ")

        //Base64Pad
        #expect(testString.encodeUTF8(base: .base64Pad) == BasicTests["base64pad"])
        #expect(testString2.encodeUTF8(base: .base64Pad) == "MRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ==")

        //Base64Url
        #expect(testString.encodeUTF8(base: .base64Url) == BasicTests["base64url"])
        #expect(testString2.encodeUTF8(base: .base64Url) == "uRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ")

        //Base64UrlPad
        #expect(testString.encodeUTF8(base: .base64UrlPad) == BasicTests["base64urlpad"])
        #expect(testString2.encodeUTF8(base: .base64UrlPad) == "URGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ==")

    }

    @Test func testCaseInsensitivity() throws {
        let testString = "hello world"

        //Base16 Lower
        let base16Lower = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base16"]!)
        #expect(base16Lower.base == .base16)
        #expect(base16Lower.string == testString)

        //Base16 Upper
        let base16Upper = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base16upper"]!)
        #expect(base16Upper.base == .base16Upper)
        #expect(base16Upper.string == testString)

        //Base32
        let base32 = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base32"]!)
        #expect(base32.base == .base32)
        #expect(base32.string == testString)

        //Base32 Upper
        let base32Upper = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base32upper"]!)
        #expect(base32Upper.base == .base32Upper)
        #expect(base32Upper.string == testString)

        //Base32 Pad
        let base32Pad = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base32pad"]!)
        #expect(base32Pad.base == .base32Pad)
        #expect(base32Pad.string == testString)

        //Base32 Upper Pad
        let base32PadUpper = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base32padupper"]!)
        #expect(base32PadUpper.base == .base32PadUpper)
        #expect(base32PadUpper.string == testString)

        //Base32 Hex
        let base32Hex = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base32hex"]!)
        #expect(base32Hex.base == .base32Hex)
        #expect(base32Hex.string == testString)

        //Base32 Hex Upper
        let base32HexUpper = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base32hexupper"]!)
        #expect(base32HexUpper.base == .base32HexUpper)
        #expect(base32HexUpper.string == testString)

        //Base32 Hex Pad
        let base32HexPad = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base32hexpad"]!)
        #expect(base32HexPad.base == .base32HexPad)
        #expect(base32HexPad.string == testString)

        //Base32 Hex Pad Upper
        let base32HexPadUpper = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base32hexpadupper"]!)
        #expect(base32HexPadUpper.base == .base32HexPadUpper)
        #expect(base32HexPadUpper.string == testString)

        //Base36
        let base36 = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base36"]!)
        #expect(base36.base == .base36)
        #expect(base36.string == testString)

        //Base36 Upper
        let base36Upper = try BaseEncoding.decodeIntoString(CaseInsensitivityTests["base36upper"]!)
        #expect(base36Upper.base == .base36Upper)
        #expect(base36Upper.string == testString)
    }

    @Test func testLeadingZero() {
        //let testString = "\\x00yes mani !"
        let testString = "\0yes mani !"

        //Base2
        #expect(testString.encodeUTF8(base: .base2) == CaseLeadingZero["base2"])

        //Base8
        #expect(testString.encodeUTF8(base: .base8) == CaseLeadingZero["base8"])

        //Base10
        #expect(testString.encodeUTF8(base: .base10) == CaseLeadingZero["base10"])

        //Base16 Lower
        #expect(testString.encodeUTF8(base: .base16) == CaseLeadingZero["base16"])

        //Base16 Upper
        #expect(testString.encodeUTF8(base: .base16Upper) == CaseLeadingZero["base16upper"])

        //Base32
        #expect(testString.encodeUTF8(base: .base32) == CaseLeadingZero["base32"])

        //Base32 Upper
        #expect(testString.encodeUTF8(base: .base32Upper) == CaseLeadingZero["base32upper"])

        //Base32 Pad
        #expect(testString.encodeUTF8(base: .base32Pad) == CaseLeadingZero["base32pad"])

        //Base32 Upper Pad
        #expect(testString.encodeUTF8(base: .base32PadUpper) == CaseLeadingZero["base32padupper"])

        //Base32 Hex
        #expect(testString.encodeUTF8(base: .base32Hex) == CaseLeadingZero["base32hex"])

        //Base32 Hex Upper
        #expect(testString.encodeUTF8(base: .base32HexUpper) == CaseLeadingZero["base32hexupper"])

        //Base32 Hex Pad
        #expect(testString.encodeUTF8(base: .base32HexPad) == CaseLeadingZero["base32hexpad"])

        //Base32 Hex Pad Upper
        #expect(testString.encodeUTF8(base: .base32HexPadUpper) == CaseLeadingZero["base32hexpadupper"])

        //Base32 Z
        #expect(testString.encodeUTF8(base: .base32z) == CaseLeadingZero["base32z"])

        //Base36
        #expect(testString.encodeUTF8(base: .base36) == CaseLeadingZero["base36"])

        //Base36 Upper
        #expect(testString.encodeUTF8(base: .base36Upper) == CaseLeadingZero["base36upper"])

        //Base58 BTC
        #expect(testString.encodeUTF8(base: .base58btc) == CaseLeadingZero["base58btc"])

        //Base58 Flickr
        #expect(testString.encodeUTF8(base: .base58flickr) == CaseLeadingZero["base58flickr"])

        //Base64
        #expect(testString.encodeUTF8(base: .base64) == CaseLeadingZero["base64"])

        //Base64 Pad
        #expect(testString.encodeUTF8(base: .base64Pad) == CaseLeadingZero["base64pad"])

        //Base64 URL
        #expect(testString.encodeUTF8(base: .base64Url) == CaseLeadingZero["base64url"])

        //Base64 URL Pad
        #expect(testString.encodeUTF8(base: .base64UrlPad) == CaseLeadingZero["base64urlpad"])
    }

    @Test func testTwoLeadingZero() {
        //let testString = "\\x00\\x00yes mani !"
        let testString = "\0\0yes mani !"

        //Base2
        #expect(testString.encodeUTF8(base: .base2) == CaseTwoLeadingZeros["base2"])

        //Base8
        #expect(testString.encodeUTF8(base: .base8) == CaseTwoLeadingZeros["base8"])

        //Base10
        #expect(testString.encodeUTF8(base: .base10) == CaseTwoLeadingZeros["base10"])

        //Base16
        #expect(testString.encodeUTF8(base: .base16) == CaseTwoLeadingZeros["base16"])

        //Base16 Upper
        #expect(testString.encodeUTF8(base: .base16Upper) == CaseTwoLeadingZeros["base16upper"])

        //Base32
        #expect(testString.encodeUTF8(base: .base32) == CaseTwoLeadingZeros["base32"])

        //Base32 Upper
        #expect(testString.encodeUTF8(base: .base32Upper) == CaseTwoLeadingZeros["base32upper"])

        //Base32 Pad
        #expect(testString.encodeUTF8(base: .base32Pad) == CaseTwoLeadingZeros["base32pad"])

        //Base32 Upper Pad
        #expect(testString.encodeUTF8(base: .base32PadUpper) == CaseTwoLeadingZeros["base32padupper"])

        //Base32 Hex
        #expect(testString.encodeUTF8(base: .base32Hex) == CaseTwoLeadingZeros["base32hex"])

        //Base32 Hex Upper
        #expect(testString.encodeUTF8(base: .base32HexUpper) == CaseTwoLeadingZeros["base32hexupper"])

        //Base32 Hex Pad
        #expect(testString.encodeUTF8(base: .base32HexPad) == CaseTwoLeadingZeros["base32hexpad"])

        //Base32 Hex Pad Upper
        #expect(testString.encodeUTF8(base: .base32HexPadUpper) == CaseTwoLeadingZeros["base32hexpadupper"])

        //Base32 Z
        #expect(testString.encodeUTF8(base: .base32z) == CaseTwoLeadingZeros["base32z"])

        //Base36
        #expect(testString.encodeUTF8(base: .base36) == CaseTwoLeadingZeros["base36"])

        //Base36 Upper
        #expect(testString.encodeUTF8(base: .base36Upper) == CaseTwoLeadingZeros["base36upper"])

        //Base58 BTC
        #expect(testString.encodeUTF8(base: .base58btc) == CaseTwoLeadingZeros["base58btc"])

        //Base58 Flickr
        #expect(testString.encodeUTF8(base: .base58flickr) == CaseTwoLeadingZeros["base58flickr"])

        //Base64
        #expect(testString.encodeUTF8(base: .base64) == CaseTwoLeadingZeros["base64"])

        //Base64 Pad
        #expect(testString.encodeUTF8(base: .base64Pad) == CaseTwoLeadingZeros["base64pad"])

        //Base64 URL
        #expect(testString.encodeUTF8(base: .base64Url) == CaseTwoLeadingZeros["base64url"])

        //Base64 URL Pad
        #expect(testString.encodeUTF8(base: .base64UrlPad) == CaseTwoLeadingZeros["base64urlpad"])
    }

    //@Test func testBase32StringEncoding() {
    //    #expect("".base32(upper: true, padded: true) == "")
    //    #expect("f".base32(upper: true, padded: true) == "MY======")
    //    #expect("fo".base32(upper: true, padded: true) == "MZXQ====")
    //    #expect("foo".base32(upper: true, padded: true) == "MZXW6===")
    //    #expect("foob".base32(upper: true, padded: true) == "MZXW6YQ=")
    //    #expect("fooba".base32(upper: true, padded: true) == "MZXW6YTB")
    //    #expect("foobar".base32(upper: true, padded: true) == "MZXW6YTBOI======")
    //    #expect("yes mani !".base32() == "pfsxgidnmfxgsibb")
    //    #expect("hello world".base32() == "nbswy3dpeb3w64tmmq")
    //    #expect("Decentralize everything!!".base32() == "irswgzloorzgc3djpjssazlwmvzhs5dinfxgoijb")
    //}
}
