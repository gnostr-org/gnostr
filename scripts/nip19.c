#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

/**
 * BIP-64MOD + GCC MONOLITHIC IMPLEMENTATION
 * Features: --prefix flag support for npub, nsec, note.
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
    for (size_t i = 0; i < hrp_len; ++i) {
        chk = bech32_polymod_step(chk) ^ (hrp[i] >> 5);
        output[i] = hrp[i];
    }
    output[hrp_len] = '1';
    chk = bech32_polymod_step(chk);
    for (size_t i = 0; i < hrp_len; ++i) chk = bech32_polymod_step(chk) ^ (hrp[i] & 0x1f);
    for (size_t i = 0; i < data_len; ++i) {
        chk = bech32_polymod_step(chk) ^ data[i];
        output[hrp_len + 1 + i] = CHARSET[data[i]];
    }
    for (int i = 0; i < 6; ++i) chk = bech32_polymod_step(chk);
    chk ^= 1; 
    for (int i = 0; i < 6; ++i) {
        output[hrp_len + 1 + data_len + i] = CHARSET[(chk >> ((5 - i) * 5)) & 0x1f];
    }
    output[hrp_len + 1 + data_len + 6] = 0;
    return 1;
}

// --- UTILS ---
int hex_to_bytes(const char *hex, uint8_t *bytes) {
    if (strlen(hex) != 64) return 0; // Nostr keys/ids are 32 bytes (64 hex chars)
    for (int i = 0; i < 32; i++) {
        sscanf(hex + 2 * i, "%02hhx", &bytes[i]);
    }
    return 1;
}

void print_usage() {
    printf("Usage: ./nostr_tool --prefix <npub|nsec|note> <hex_string>\n");
}

// --- MAIN ---
int main(int argc, char *argv[]) {
    if (argc < 4) {
        print_usage();
        return 1;
    }

    char *prefix = NULL;
    char *hex_input = NULL;

    // Basic Flag Parsing
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--prefix") == 0 && i + 1 < argc) {
            prefix = argv[++i];
        } else {
            hex_input = argv[i];
        }
    }

    if (!prefix || !hex_input) {
        print_usage();
        return 1;
    }

    uint8_t raw_data[32];
    if (!hex_to_bytes(hex_input, raw_data)) {
        fprintf(stderr, "Error: Input must be a 64-character hex string (32 bytes).\n");
        return 1;
    }

    uint8_t words[64];
    size_t words_len = 0;
    char encoded_output[1024];

    convert_bits(words, &words_len, 5, raw_data, 32, 8, 1);
    bech32_encode(encoded_output, prefix, words, words_len);

    printf("%s\n", encoded_output);

    return 0;
}
