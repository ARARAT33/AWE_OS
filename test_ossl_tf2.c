
#include <stdio.h>
#include <openssl/sha.h>

int main() {
    SHA256_CTX ctx;
    SHA256_Init(&ctx);
    unsigned char block[64] = {0};
    block[0] = 0x80;
    // Note: bit length in big endian at offset 56..63 is 0!
    SHA256_Transform(&ctx, block);
    printf("ctx.h[0] = %08x\n", ctx.h[0]);
    return 0;
}
