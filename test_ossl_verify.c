
#include <stdio.h>
#include <openssl/sha.h>

int main() {
    SHA256_CTX ctx;
    SHA256_Init(&ctx);
    unsigned char block[64] = {0};
    block[0] = 0x80;
    // Call internal transform or Update + Final
    SHA256_Update(&ctx, "", 0);
    unsigned char digest[32];
    SHA256_Final(digest, &ctx);
    for(int i=0; i<32; i++) printf("%02x", digest[i]);
    printf("\n");
    return 0;
}
