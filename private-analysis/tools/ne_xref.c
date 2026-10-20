/* Private forensic helper: locate direct 16-bit memory references in an NE file.
 * It deliberately emits offsets only; it never extracts or republishes payloads. */
#include <stdio.h>
#include <stdlib.h>

static int match(const unsigned char *p, size_t n, size_t i,
                 const unsigned char *needle, size_t needle_n) {
    size_t j;
    if (i + needle_n > n) return 0;
    for (j = 0; j < needle_n; ++j) if (p[i + j] != needle[j]) return 0;
    return 1;
}

int main(int argc, char **argv) {
    FILE *f;
    long size;
    unsigned char *data;
    size_t i, got;
    const unsigned char patterns[][4] = {
        {0x91, 0x2d, 0, 0}, {0x93, 0x2d, 0, 0},
        {0x8d, 0x2d, 0, 0}, {0x8f, 0x2d, 0, 0}
    };
    if (argc != 2) return fprintf(stderr, "usage: ne_xref <file>\n"), 2;
    f = fopen(argv[1], "rb");
    if (!f) return perror(argv[1]), 1;
    fseek(f, 0, SEEK_END); size = ftell(f); rewind(f);
    if (size < 1) return fclose(f), 1;
    data = malloc((size_t)size);
    if (!data) return fclose(f), 1;
    got = fread(data, 1, (size_t)size, f); fclose(f);
    for (i = 0; i + 4 <= got; ++i) {
        size_t p;
        for (p = 0; p < sizeof(patterns)/sizeof(patterns[0]); ++p) {
            if (match(data, got, i, patterns[p], 2)) {
                size_t start = i > 8 ? i - 8 : 0;
                size_t end = i + 12 < got ? i + 12 : got;
                size_t j;
                printf("file+0x%05zX  bytes", i);
                for (j = start; j < end; ++j) printf(" %02X", data[j]);
                printf("\n");
                break;
            }
        }
    }
    free(data);
    return 0;
}
