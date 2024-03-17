/* Print pi to n decimal places (default 1000): pi n */

#define VERSION "v0.0.7"

#define DEBUG  0
#define DEBUG2 0
#define PRIMER 0

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <math.h>

#include <sys/types.h>
#include <sys/stat.h>
#include <stdio.h>
#include <time.h>
#include <stdlib.h>
#include <assert.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <unistd.h>

//#include "../include/argparse.h"

// #define M_LOG2E 1.44269504088896340736 // log2(e)
// inline long double log2(const long double x){
//     return log(x) * M_LOG2E;
// }

int help();
int help(){

    printf("gnostr-pi - deterministic entropy %s                                   \n", VERSION);
    printf("                                                                       \n");
    printf("gnostr-pi   depth                                                      \n");
    printf("gnostr-pi   <int>                                                      \n");
    printf("                                                                       \n");
    printf("gnostr-pi   0                                                            \n");
    printf("gnostr-pi   4               1415                                         \n");
    printf("gnostr-pi   8               14159265                                     \n");
    printf("gnostr-pi   12              141592653589                                 \n");
    printf("gnostr-pi   16              1415926535897932                             \n");
    printf("gnostr-pi   20              14159265358979323846                         \n");
    printf("gnostr-pi   24              141592653589793238462643                     \n");
    printf("gnostr-pi   28              1415926535897932384626433832                 \n");
    printf("gnostr-pi   32              14159265358979323846264338327950             \n");
    printf("                                                                         \n");
    printf("gnostr-pi   depth   offset      default 1000 digits of pi mantissa       \n");
    printf("                                The mantissa of π is the fractional part.\n");
    printf("            <int>   <int>                                                \n");
    printf("            0       0           14159265...64201989                      \n");
    printf("            0       1           offset 1*4 digits                        \n");
    printf("            0       2           offset 2*4 digits                        \n");
    printf("            0       3           offset 3*4 digits                        \n");
    printf("            0       4           offset 4*4 digits                        \n");
    printf("            0       n           offset n*4 digits                        \n");
    printf("                                                                         \n");
    printf("                                                                         \n");
    printf("                                                                       \n");
    printf("Context:                                                               \n");
    printf("                                                                       \n");
    printf("                                                                       \n");
    printf("gnostr-pi - deterministic entropy %s                                   \n", VERSION);
    printf("                                                                       \n");
return 0;//exit(0);
}
int about();
int about(){

    printf("About:                                                                  \n");
    printf("                                                                        \n");
    printf("Compute pi to B bits precision by the Spigot algorithm given by         \n");
    printf("Rabinowitz and Wagon, Am. Math. Monthly, March 1995, 195-203.           \n");
    printf("                                                                        \n");
    printf("    pi = 4;                                                             \n");
    printf("    for (i = B; i>0; --i)                                               \n");
    printf("        pi = 2 + pi * i / (2*i+1)                                       \n");
    printf("                                                                        \n");
    printf("pi is represented by a base 10000 array of digits with 2 digits before  \n");
    printf("the decimal point (pi[0], pi[1]), and one extra digit (pi[n-1]) at      \n");
    printf("the end to allow for roundoff error, which is not printed.  Note that a \n");
    printf("base 10 digit is equivalent to log(10)/log(2) = 3.322 bits.             \n");
    printf("                                                                        \n");
    printf("                                                                        \n");
    printf("3.3219280948873623478703194294893901758648313930245806120547563958159...\n");
    printf("if π = log(10)/log(x) then x = 10^(1/π)                                 \n");
return 0;//exit(0);
}
int version();
int version(){
    printf("%s", VERSION);
    return 0;
}
void int2bin(int n, int* bin, int* bin_size, const int bits);
void int2bin(int n, int* bin, int *bin_size, const int bits)
{
    int i = 0;
    int temp[64];
    for (int j = 0; j < 64; j++)
    {
        temp[j] = 0;
    }
    for (int l = 0; l < bits; l++)
    {
        bin[l] = 0;
    }

    while (n > 0)
    {
        temp[i] = n % 2;
        n = n / 2;
        i++;
    }
    *bin_size = i;

    //reverse modulus values
    for (int k = 0; k < *bin_size; k++)
    {
        bin[bits-*bin_size+k] = temp[*bin_size - 1 - k];
    }
}

void printbin(int *binary);
void printbin(int *binary){

    if (DEBUG2){
        printf("printbin\n");
        for (int i = 0; i < 32; i++){ printf("%p=%d\n", &binary[i], binary[i]); }
    }

}
/* Print pi as an array of n digits in base 10000 */
void print(unsigned short *pi, int n, int offset) {

                                         // Additional Properties
// printf("SIZE_MAX=%lu\n",SIZE_MAX);    // 18446744073709551615
                                         // 19 decimal digits
                                         // 1111111111111111111111111111111111111111111111111111111111111_2
                                         // 18446744073709551615/x;
                                         // sqrt(abs(x)^2 + 340282366920938463426481119284349108225/abs(x)^2)
                                         //
                                         //
// printf("SIZE_MAX/8=%lu\n",SIZE_MAX/8);// 2305843009213693951
                                         // 2305843009213693951/x;
                                         // sqrt(abs(x)^2 + 5316911983139663487003542222693990401/abs(x)^2)
                                         //

  int i;

  /* REMOVE characteristic '3.'
   * we are only concerned with mantissa
   * printf("%d.", pi[1]);
   */

  for (i=2+offset; i < (n-1); ++i){
  //for (i=2; i<n-1; ++i){ //more analysis needed

      printf("%04d", pi[i]);

  }
}

/* Compute pi to B bits precision by the Spigot algorithm given by
   Rabinowitz and Wagon, Am. Math. Monthly, March 1995, 195-203.

      pi = 4;
      for (i = B; i>0; --i)
          pi = 2 + pi * i / (2*i+1)

   pi is represented by a base 10000 array of digits with 2 digits before
   the decimal point (pi[0], pi[1]), and one extra digit (pi[n-1]) at
   the end to allow for roundoff error, which is not printed.  Note that a
   base 10 digit is equivalent to log(10)/log(2) = 3.322 bits.

   For shorter versions, see
   http://www1.physik.tu-muenchen.de/~gammel/matpack/html/Mathematics/Pi.html
   http://numbers.computation.free.fr/Constants/TinyPrograms/tinycodes.html

   and for an explanation of how they work, see
   Unbounded Spigot Algorithms for the Digits of Pi,
   Jeremy Gibbons, University of Oxford, 2004,
   http://web.comlab.ox.ac.uk/oucl/work/jeremy.gibbons/publications/spigot.pdf

*/

int main(int argc, char** argv) {

    if ( argc > 2 ){
    // lower bound of offset is -253
    // -250 keeps the 3 (characteristic) as the last digit in the output
    // the output is pseudo-random rounding error
    // TODO: add test to output to ensure the 3 (characteristic is present)
    // error otherwise
        if ( atoi(argv[2]) < -1*250){ help(); exit(0); }
    }
/*
   input 0 4 8 12 16 20 24 28 etc...
*/

  if (DEBUG2){

     /* begin int2bin */

     char ch;
     ch = 'A';

     int binary[sizeof(int)*8];
     int binary_size = 0;

     int2bin(0, binary, &binary_size, sizeof(int)-1);
     printf("case:0\n");

     printf("sizeof(binary)=%lu\n",sizeof(binary));
     printf("sizeof(binary_size)=%lu\n",sizeof(binary_size));
     printbin(binary);printf("\n");
     printf("binary=%d\n",binary[(sizeof(binary)/4-1)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-2)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-3)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-4)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-5)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-6)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-7)]);

     int2bin(ch, binary, &binary_size, 32);
     printf("case:%c\n",ch);
     printf("sizeof(binary)=%lu\n",sizeof(binary));
     printf("sizeof(binary_size)=%lu\n",sizeof(binary_size));
     printbin(binary);printf("\n");
     printf("*binary=%d\n",*binary);
     printf("binary=%d\n",binary[(sizeof(binary)/4-1)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-2)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-3)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-4)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-5)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-6)]);
     printf("binary=%d\n",binary[(sizeof(binary)/4-7)]);

     int2bin(1324, binary, &binary_size, 32);
     // printf("sizeof(binary)=%lu\n",sizeof(binary));
     // printf("*binary=%d\n",*binary);
     // printbin(binary);
     // printf("binary=%d\n",binary[(sizeof(binary)/4-1)]);
     // printf("binary=%d\n",binary[(sizeof(binary)/4-2)]);
     // printf("binary=%d\n",binary[(sizeof(binary)/4-3)]);
     // printf("binary=%d\n",binary[(sizeof(binary)/4-4)]);
     // printf("binary=%d\n",binary[(sizeof(binary)/4-5)]);
     // printf("binary=%d\n",binary[(sizeof(binary)/4-6)]);

     // static char buf[128] = {0};
     // const char *sub = argv[1];
     // if (strlen(sub) >= 1 && sub[0] != '-') {
     //     snprintf(buf, sizeof(buf)-1, "echo %s", sub);
     //     execvp(buf, (char * const *)argv+1);
     // }


     //exit(0);

  /*
     end int2bin
  */
  }
/*
TODO:
{A = 1, B = 0, C = 0, N!=0, x = 7/N^2, sqrt(343/N^6 + 7) = y}
*/

  int n = {253};
  int offset = {0};

  if (argc == 1){ help(); exit(0); }
  if (argc == 2){

      if (!strcmp(argv[1],"about")){
          about(); exit(0);
      }
      if (!strcmp(argv[1],"help")){
          help(); exit(0);
      }
      if (!strcmp(argv[1],"-h")){
          help(); exit(0);
      }
      if (!strcmp(argv[1],"--help")){
          help(); exit(0);
      }
      if (!strcmp(argv[1],"-v")){
          version(); exit(0);
      }
      if (!strcmp(argv[1],"--version")){
          version(); exit(0);
      }

      /* 253 -> 1000 default number of pi digits */
      n = argc == 2 ? (atoi(argv[1]) + 3)/4 + 3 : 253;

      if (!strcmp(argv[1],"debug")){
          printf("atoi(argv[0])=%s\n", argv[0]);
          printf("atoi(argv[1])=%d\n", atoi(argv[1]));
          printf("            n=%d\n", n);
          printf("       log(n)=%f\n", log(n));
          printf("      log2(n)=%f\n", log2(n));
          printf("       log(2)=%f\n", log(2));
          printf("      log2(2)=%f\n", log2(2));
          printf("          bit/%f\n", log(10) / log(2));
          printf("         bits/%f\n", log(n) / log(2));
      }
  }

  if (argc == 3){

      if (!strcmp(argv[2],"debug")){
          //printf("atoi(argv[0])=%d\n", atoi(argv[0]));
          printf("atoi(argv[0])=%s\n", argv[0]);
          printf("atoi(argv[1])=%d\n", atoi(argv[1]));
          printf("atoi(argv[2])=%s\n", argv[2]);
      }

      offset = atoi(argv[2]);
      n = argc == 2 ? (atoi(argv[1]) + 3 + offset)/4 + 3 + offset : 253; /* 253 default number of pi digits */
      n += offset;

      if (DEBUG) { printf("n += offset:%d\n", n); }
  }

if (argc == 4){

    if (DEBUG) {

        printf("TODO:handle <path,salt>\n");
        printf("%d\n", atoi(argv[3]));

    }

      exit(0);

}
  if (argc > 4){ help(); }

    if (DEBUG) {

        printf("bits/%f\n", log2(n) / log2(2));// keep percision
        printf("bits/%f\n", log2(atoi(argv[1])) / log2(2));// keep percision

    }

  unsigned short *pi = (unsigned short*) malloc(n * sizeof(unsigned short));

  if (DEBUG){ printf("*pi = n * sizeof(unsigned short)=%lu\n", n * sizeof(unsigned short));}

  div_t d;
  int i, j, t;

  /* pi = 4  */
  memset(pi, 0, n*sizeof(unsigned short));
  pi[1]=4;

  /* for i = B down to 1 */
  // NOTE:
  for (i=(int)(3.322*4*n); i>0; --i) {

    if (DEBUG2){ printf("i=%d\n",i); }

    /* pi *= i; */
    t = 0;
    for (j=n-1; j>=0; --j) {  /* pi *= i; */
      if (DEBUG2){ printf("j=%d\n",j); }
      t += pi[j] * i;
      if (DEBUG2){ printf("t=%d\n",t); }
      pi[j] = t % 10000;
      if (DEBUG2){ printf("pi[j]=%d\n",pi[j]); }
      t /= 10000;
      if (DEBUG2){ printf("t=%d\n",t); }
    }
    if (PRIMER){
        if (offset <= 933){

            printf("offset <= 933\n");

            if (offset >= -1*253){

                printf("offset > -1*253\n");

                print((unsigned short *)&t, n, offset); exit(0);

            }

            if (offset < -1*253){

                printf("offset < -253\n"); exit(0);

            }

        } else {}

        exit(0);

    }

    /* pi /= (2*i+1) */
    d.quot = d.rem = 0;
    for (j=0; j<n; ++j) {  /* pi /= 2*i+1; */
      if (DEBUG){ printf("j=%d\n",j); }
      if (DEBUG){ printf("i=%d\n",i); }
      d = div(pi[j]+d.rem*10000, i+i+1);
      if (DEBUG){ printf("d.quot=%d\n",d.quot); }
      if (DEBUG){ printf("d.rem=%d\n",d.rem); }
      pi[j] = d.quot;
     if (DEBUG){  printf("pi[j]=%d\n",pi[j]); }
    }

    /* pi += 2 */
    pi[1] += 2;
    if (DEBUG){ printf("pi[1]=%d\n",pi[1]); }
  }
  // exit(0);
  print(pi, n, offset);
  return 0;
}

