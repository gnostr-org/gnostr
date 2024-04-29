#include <stdio.h>
#include <string.h>
#include <stdint.h>

int32_t copyx(unsigned char *output, const unsigned char *x32, const unsigned char *y32, void *data) {
	memcpy(output, x32, 32);
	return 1;
}

