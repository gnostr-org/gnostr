#include <stdio.h>
#include <stdint.h>
#include <string.h>

/**
 * BIP-64MOD + GCC MONOLITHIC IMPLEMENTATION
 * This file contains:
 * 1. Bech32 Constants & Polymod logic
 * 2. Bit conversion (8-bit to 5-bit)
 * 3. NIP-19 npub encoding
 * 4. Main execution loop
 */

// --- BECH32 ENGINE ---
static const char* CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

static uint32_t bech32_polymod_step(uint32_t pre) {
    uint8_t b = pre >> 25;
    return ((pre & 0x1FFFFFF) << 5) ^
           (-((b >> 0) & 1) & 0x3b6a57b2UL) ^
           (-((b >> 1) & 1) & 0x26508e6dUL) ^
           (-((b >> 2) & 1) & 0x1ea119faUL) ^
           (-((b >> 3) & 1) & 0x3d4233ddUL) ^
           (-((b >> 4) & 1) & 0x2a1462b3UL);
}

int convert_bits(uint8_t *out, size_t *outlen, int outbits, const uint8_t *in, size_t inlen, int inbits, int pad) {
    uint32_t val = 0;
    int bits = 0;
    uint32_t maxv = (1 << outbits) - 1;
    *outlen = 0;
    for (size_t i = 0; i < inlen; ++i) {
        val = (val << inbits) | in[i];
        bits += inbits;
        while (bits >= outbits) {
            bits -= outbits;
            out[(*outlen)++] = (val >> bits) & maxv;
        }
    }
    if (pad) {
        if (bits) out[(*outlen)++] = (val << (outbits - bits)) & maxv;
    } else if (((val << (outbits - bits)) & maxv) || bits >= inbits) {
        return 0;
    }
    return 1;
}

int bech32_encode(char *output, const char *hrp, const uint8_t *data, size_t data_len) {
    uint32_t chk = 1;
    size_t hrp_len = strlen(hrp);
    
    // HRP Checksum part 1
    for (size_t i = 0; i < hrp_len; ++i) {
        chk = bech32_polymod_step(chk) ^ (hrp[i] >> 5);
        output[i] = hrp[i];
    }
    output[hrp_len] = '1';
    chk = bech32_polymod_step(chk);
    
    // HRP Checksum part 2
    for (size_t i = 0; i < hrp_len; ++i) chk = bech32_polymod_step(chk) ^ (hrp[i] & 0x1f);
    
    // Data encoding
    for (size_t i = 0; i < data_len; ++i) {
        chk = bech32_polymod_step(chk) ^ data[i];
        output[hrp_len + 1 + i] = CHARSET[data[i]];
    }
    
    // Final checksum
    for (int i = 0; i < 6; ++i) chk = bech32_polymod_step(chk);
    chk ^= 1; // NIP-19 uses Bech32 constant 1
    
    for (int i = 0; i < 6; ++i) {
        output[hrp_len + 1 + data_len + i] = CHARSET[(chk >> ((5 - i) * 5)) & 0x1f];
    }
    output[hrp_len + 1 + data_len + 6] = 0;
    return 1;
}

// --- NOSTR WRAPPER ---
void nostr_hex_to_npub(const uint8_t *pubkey, char *output) {
    uint8_t words[64];
    size_t words_len = 0;
    
    // Step 1: Convert 8-bit hex to 5-bit words
    convert_bits(words, &words_len, 5, pubkey, 32, 8, 1);
    
    // Step 2: Encode with 'npub' prefix
    bech32_encode(output, "npub", words, words_len);
}

// --- MAIN TEST ---
int main() {
    // Example: This is a 32-byte Ed25519 hex public key
    uint8_t my_pubkey[32] = {
        0x3b, 0xf0, 0xc6, 0x3f, 0xc0, 0x3d, 0x07, 0x37, 
        0x30, 0x47, 0xc5, 0xed, 0x2e, 0xcc, 0x46, 0x94, 
        0x47, 0xa9, 0x62, 0x15, 0x9b, 0x67, 0x7c, 0x7a, 
        0x7b, 0xbc, 0x05, 0x5d, 0x0d, 0x7e, 0x11, 0x23
    };

    char npub_string[128];
    nostr_hex_to_npub(my_pubkey, npub_string);

    printf("BIP-64MOD Context: NIP-19 Monolithic Encoder\n");
    printf("------------------------------------------\n");
    printf("Input Hex: ");
    for(int i=0; i<32; i++) printf("%02x", my_pubkey[i]);
    
    printf("\nNostr npub: %s\n", npub_string);
    printf("------------------------------------------\n");

    return 0;
}
