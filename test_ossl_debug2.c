
#include <stdio.h>
#include <openssl/sha.h>

int main() {
    SHA256_CTX ctx;
    SHA256_Init(&ctx);
    unsigned char block[64] = {0};
    block[0] = 0x80;
    // Let us check what ctx.h is before and after SHA256_Transform
    printf("Before transform: ");
    for(int i=0; i<8; i++) printf("%08x ", ctx.h[i]);
    printf("\n");

    SHA256_Transform(&ctx, block);

    printf("After transform:  ");
    for(int i=0; i<8; i++) printf("%08x ", ctx.h[i]);
    printf("\n");
    return 0;
}
