#include <stdio.h>
#include <string.h>
#include <assert.h>
#include "bzlib.h"

extern void bz_internal_error(int errcode) {
    fprintf(stderr, "bzip2 hit internal error code: %d\n", errcode);
}

#if defined(MSDOS) || defined(OS2) || defined(WIN32) || defined(__CYGWIN__)
#  include <fcntl.h>
#  include <io.h>
#  define SET_BINARY_MODE(file) setmode(fileno(file), O_BINARY)
#else
#  define SET_BINARY_MODE(file)
#endif

#define CHUNK 256

/* Compress from file source to file dest until EOF on source.
   def() returns BZ_OK on success, BZ_MEM_ERROR if memory could not be
   allocated for processing, Z_STREAM_ERROR if an invalid compression
   level is supplied, Z_VERSION_ERROR if the version of zlib.h and the
   version of the library linked do not match, or BZ_IO_ERROR if there is
   an error reading or writing the files. */
int def(FILE *source, FILE *dest)
{
    int ret;
    unsigned have;
    bz_stream strm;
    unsigned char in[CHUNK];
    unsigned char out[CHUNK];

    /* allocate deflate state */
    strm.bzalloc = NULL;
    strm.bzfree = NULL;
    strm.opaque = NULL;
    ret = BZ2_bzCompressInit(&strm, 9, 0, 0);
    if (ret != BZ_OK)
        return ret;

    /* compress until end of file */
    int done = 0;
    while (!done) {
        strm.avail_in = fread(in, 1, CHUNK, source);
        done = feof(source);

        if (ferror(source)) {
            (void)BZ2_bzCompressEnd(&strm);
            return BZ_IO_ERROR;
        }
        strm.next_in = in;

        /* run deflate() on input until output buffer not full, finish
           compression if all of source has been read in */
        do {
            strm.avail_out = CHUNK;
            strm.next_out = out;
            ret = BZ2_bzCompress(&strm, BZ_FLUSH);    /* no bad return value */
            assert(ret != BZ_PARAM_ERROR);  /* state not clobbered */
            have = CHUNK - strm.avail_out;
            if (fwrite(out, 1, have, dest) != have || ferror(dest)) {
                (void)BZ2_bzCompressEnd(&strm);
                return BZ_IO_ERROR;
            }
        } while (strm.avail_out == 0);
        assert(strm.avail_in == 0);     /* all input will be used */

        /* done when last data in file processed */
    }

    strm.avail_out = CHUNK;
    strm.next_out = out;
    ret = BZ2_bzCompress(&strm, BZ_FINISH);    /* no bad return value */
    have = CHUNK - strm.avail_out;
    if (fwrite(out, 1, have, dest) != have || ferror(dest)) {
        (void)BZ2_bzCompressEnd(&strm);
        return BZ_IO_ERROR;
    }

    assert(ret == BZ_STREAM_END);        /* stream will be complete */

    /* clean up and return */
    (void)BZ2_bzCompressEnd(&strm);
    return BZ_OK;
}

/* Decompress from file source to file dest until stream ends or EOF.
   inf() returns BZ_OK on success, BZ_MEM_ERROR if memory could not be
   allocated for processing, BZ_DATA_ERROR if the deflate data is
   invalid or incomplete, Z_VERSION_ERROR if the version of zlib.h and
   the version of the library linked do not match, or BZ_IO_ERROR if there
   is an error reading or writing the files. */
int inf(FILE *source, FILE *dest)
{
    int ret;
    unsigned have;
    bz_stream strm;
    unsigned char in[CHUNK];
    unsigned char out[CHUNK];

    /* allocate inflate state */
    strm.bzalloc = NULL;
    strm.bzfree = NULL;
    strm.opaque = NULL;
    strm.avail_in = 0;
    strm.next_in = NULL;
    ret = BZ2_bzDecompressInit(&strm, 0, 0);
    if (ret != BZ_OK)
        return ret;

    /* decompress until deflate stream ends or end of file */
    do {
        strm.avail_in = fread(in, 1, CHUNK, source);
        if (ferror(source)) {
            (void)BZ2_bzDecompressEnd(&strm);
            return BZ_IO_ERROR;
        }
        if (strm.avail_in == 0)
            break;
        strm.next_in = in;

        /* run inflate() on input until output buffer not full */
        do {
            strm.avail_out = CHUNK;
            strm.next_out = out;
            ret = BZ2_bzDecompress(&strm);
            assert(ret != BZ_PARAM_ERROR);  /* state not clobbered */
            switch (ret) {
            case BZ_DATA_ERROR:
            case BZ_MEM_ERROR:
                (void)BZ2_bzDecompressEnd(&strm);
                return ret;
            }
            have = CHUNK - strm.avail_out;
            if (fwrite(out, 1, have, dest) != have || ferror(dest)) {
                (void)BZ2_bzDecompressEnd(&strm);
                return BZ_IO_ERROR;
            }
        } while (strm.avail_out == 0);

        /* done when BZ2_bzDecompress() says it's done */
    } while (ret != BZ_STREAM_END);

    /* clean up and return */
    (void)BZ2_bzDecompressEnd(&strm);
    return ret == BZ_STREAM_END ? BZ_OK : BZ_DATA_ERROR;
}

/* report a zlib or i/o error */
void zerr(int ret)
{
    fputs("zpipe: ", stderr);
    switch (ret) {
    case BZ_IO_ERROR:
        if (ferror(stdin))
            fputs("error reading stdin\n", stderr);
        if (ferror(stdout))
            fputs("error writing stdout\n", stderr);
        break;
    case BZ_PARAM_ERROR:
        fputs("invalid block size\n", stderr);
        break;
    case BZ_DATA_ERROR:
    case BZ_DATA_ERROR_MAGIC:
        fputs("invalid or incomplete data\n", stderr);
        break;
    case BZ_MEM_ERROR:
        fputs("out of memory\n", stderr);
        break;
    }
}

/* compress or decompress from stdin to stdout */
int main(int argc, char **argv)
{
    int ret;

    /* avoid end-of-line conversions */
    SET_BINARY_MODE(stdin);
    SET_BINARY_MODE(stdout);

    /* do compression if no arguments */
    if (argc == 1) {
        ret = def(stdin, stdout);
        if (ret != BZ_OK)
            zerr(ret);
        return ret;
    }

    /* do decompression if -d specified */
    else if (argc == 2 && strcmp(argv[1], "-d") == 0) {
        ret = inf(stdin, stdout);
        if (ret != BZ_OK)
            zerr(ret);
        return ret;
    }

    /* otherwise, report usage */
    else {
        fputs("bzpipe usage: bzpipe [-d] < source > dest\n", stderr);
        return 1;
    }
}
