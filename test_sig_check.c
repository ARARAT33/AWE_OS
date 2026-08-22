
#include <stdio.h>
#include <stdint.h>

#define ROTR(x, n) (((x) >> (n)) | ((x) << (32 - (n))))
#define CH(x, y, z) (((x) & (y)) ^ (~(x) & (z)))
#define MAJ(x, y, z) (((x) & (y)) ^ ((x) & (z)) ^ ((y) & (z)))
#define SIG0(x) (ROTR(x, 2) ^ ROTR(x, 13) ^ ROTR(x, 22))
#define SIG1(x) (ROTR(x, 6) ^ ROTR(x, 11) ^ ROTR(x, 25))
#define sig0(x) (ROTR(x, 7) ^ ROTR(x, 18) ^ ((x) >> 3))
#define sig1(x) (ROTR(x, 17) ^ ROTR(x, 19) ^ ((x) >> 10))

int main() {
    uint32_t H[8] = {
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
    };
    uint8_t block[64] = {0};
    block[0] = 0x80;

    uint32_t W[64];
    for (int i = 0; i < 16; i++) {
        W[i] = ((uint32_t)block[i*4] << 24) | ((uint32_t)block[i*4+1] << 16) | ((uint32_t)block[i*4+2] << 8) | (uint32_t)block[i*4+3];
    }
    for (int i = 16; i < 64; i++) {
        W[i] = sig1(W[i-2]) + W[i-7] + sig0(W[i-15]) + W[i-16];
    }

    // Wait! Look at W[i] expansion in FIPS 180-4:
    // W_i = sig1(W_{i-2}) + W_{i-7} + sig0(W_{i-15}) + W_{i-16}
    // Is sig0 on W[i-15] or W[i-15] is sig0?
    // In FIPS 180-4:
    // W_i = sig1(W_{i-2}) + W_{i-7} + sig0(W_{i-15}) + W_{i-16}

    // WAIT! Let us check SIG0 vs sig0 definition!
    // SIG0(x) = ROTR2(x) ^ ROTR13(x) ^ ROTR22(x)  (used in state update T2)
    // sig0(x) = ROTR7(x) ^ ROTR18(x) ^ (x >> 3)    (used in message schedule W)
    // SIG1(x) = ROTR6(x) ^ ROTR11(x) ^ ROTR25(x)  (used in state update T1)
    // sig1(x) = ROTR17(x) ^ ROTR19(x) ^ (x >> 10)  (used in message schedule W)

    printf("sig0(0x80000000) = %08x\n", sig0(0x80000000));
    return 0;
}
