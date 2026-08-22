
#include <stdio.h>
#include <openssl/sha.h>

void SHA256_Transform(SHA256_CTX *c, const unsigned char *data);

int main() {
    SHA256_CTX ctx;
    SHA256_Init(&ctx);
    unsigned char block[64] = {0};
    block[0] = 0x80;
    SHA256_Transform(&ctx, block);
    for(int i=0; i<8; i++) printf("%08x ", ctx.h[i]);
    printf("\n");
    return 0;
}
