#include<stdio.h>

#include <assert.h>
#include <limits.h>
#define TO_BASE_N (sizeof(unsigned)*CHAR_BIT + 1)

//                               v--compound literal--v
#define TO_BASE(x, b) my_to_base((char [TO_BASE_N]){""}, (x), (b))

// Tailor the details of the conversion function as needed
// This one does not display unneeded leading zeros
// Use return value, not `buf`
char *my_to_base(char buf[TO_BASE_N], unsigned i, int base) {
  assert(base >= 2 && base <= 36); //0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ // 10 + 26
                                   //REF: https://en.wikipedia.org/wiki/English_alphabet
  char *s = &buf[TO_BASE_N - 1];
  *s = '\0';
  do {
    s--;
    *s = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"[i % base];
    i /= base;
  } while (i);

  // Could employ memmove here to move the used buffer to the beginning
  // size_t len = &buf[TO_BASE_N] - s;
  // memmove(buf, s, len);

  return s;
}

#include <stdio.h>
int main()
{
                                          //0000 0000   0
                                          //0000 0001   1
                                          //0000 0010   2
                                          //0000 0011   3
                                          //0000 0100   4
                                          //0000 0101   5
                                          //0000 0110   6
                                          //0000 0111   7
                                          //0000 1000   8
                                          //0001 0000  16
                                          //0010 0000  32
                                          //0100 0000  64
                                          //1000 0000 128

    int var = 2;                          //0000 0010

    printf("%d * 2  = %d \n",var,var<<1); //1 position to the left
                                          //0000 0100
    printf("%d * 4  = %d \n",var,var<<2); //2 position to the left
                                          //0000 1000
    printf("%d * 8  = %d \n",var,var<<3); //3 position to the left
                                          //0001 0000
    printf("%d * 16 = %d \n",var,var<<4); //4 position to the left
                                          //0010 0000
    printf("%d * 32 = %d \n",var,var<<5); //5 position to the left
                                          //0100 0000
    printf("%d * 64 = %d \n",var,var<<6); //6 position to the left
                                          //1000 0000

    int ip1 = 0x01020304;
    int ip2 = 0x05060708;
    printf("base10:ip1=%s ip2=%s\n", TO_BASE(ip1, 10), TO_BASE(ip2, 10));
    printf("base16:ip1=%s ip2=%s\n", TO_BASE(ip1, 16), TO_BASE(ip2, 16));
    printf("base32:ip1=%s ip2=%s\n", TO_BASE(ip1, 32), TO_BASE(ip2, 32));
    //puts(TO_BASE(ip1, 1));
    printf("ip1=%d base 2 3 4 5 6 7 8 16 32\n", ip1);
    puts(TO_BASE(ip1, 2));
    puts(TO_BASE(ip1, 3));
    puts(TO_BASE(ip1, 4));
    puts(TO_BASE(ip1, 5));
    puts(TO_BASE(ip1, 6));
    puts(TO_BASE(ip1, 7));
    puts(TO_BASE(ip1, 8));
    puts(TO_BASE(ip1, 16));
    puts(TO_BASE(ip1, 32));
    printf("ip2=%d base 2 3 4 5 6 7 8 16 32\n", ip2);
    puts(TO_BASE(ip2, 2));
    puts(TO_BASE(ip2, 3));
    puts(TO_BASE(ip2, 4));
    puts(TO_BASE(ip2, 5));
    puts(TO_BASE(ip2, 6));
    puts(TO_BASE(ip2, 7));
    puts(TO_BASE(ip2, 8));
    puts(TO_BASE(ip2, 16));
    puts(TO_BASE(ip2, 32));

    return 0;
}
