#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "bzlib.h"

int main() {
    unsigned char compressed_data[] = {
      0x42, 0x5a, 0x68, 0x39, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59, 0xa2, 0x9d,
      0x5a, 0x47, 0x00, 0x00, 0x02, 0xd1, 0x80, 0x00, 0x10, 0x60, 0x00, 0x06,
      0x44, 0x90, 0x80, 0x20, 0x00, 0x31, 0x00, 0x30, 0x20, 0x34, 0x62, 0x59,
      0x04, 0xea, 0x42, 0x19, 0x7e, 0x2e, 0xe4, 0x8a, 0x70, 0xa1, 0x21, 0x45,
      0x3a, 0xb4, 0x8e
    };
    unsigned int compressed_data_len = 51;

    // Allocate memory for the decompressed data
    int destLen = 1024; // Adjust this as needed
    char *decompressed_data = (char *)malloc(destLen);

    // Perform decompression
    int result = BZ2_bzBuffToBuffDecompress(decompressed_data, &destLen, 
                                           compressed_data, compressed_data_len, 
                                           0, 0);

    if (result == BZ_OK) {
        printf("Decompression successful!\n");
        printf("Decompressed data: %s\n", decompressed_data);
    } else {
        printf("Decompression failed with error code: %d\n", result);
    }

    free(decompressed_data);
    return 0;
}

extern void bz_internal_error(int errcode);

void bz_internal_error(int errcode) {
    fprintf(stderr, "bzip2 internal error: %d\n", errcode);
    exit(EXIT_FAILURE);
}
