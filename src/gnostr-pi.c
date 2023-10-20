/* Print pi to n decimal places (default 1000): pi n */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

/* Print pi as an array of n digits in base 10000 */
void print(unsigned short *pi, int n) {
  int i;

/* REF: https://en.wikipedia.org/wiki/Common_logarithm#Mantissa_and_characteristic
 * REMOVE characteristic '3.'
 * we are only concerned with mantissa
  printf("%d.", pi[1]);
*/
  for (i=2; i<n-1; ++i)
    printf("%04d", pi[i]);
  printf("\n");
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

/*
   input 0 4 8 12 16 20 24 28 etc...
*/

/*
   begin int2bin
*/

   char ch;
   ch = 'A';
   int binary[32];
   int binary_size = 0;

/*
   int2bin(int n, int* bin, int* bin_size, const int bits);
*/
   int2bin(1324, binary, &binary_size, 32);
   //for (int i = 0; i < 32; i++){ printf("\n%d\n", binary[i]); }

/*
   end int2bin
*/

/*
TODO:
{A = 1, B = 0, C = 0, N!=0, x = 7/N^2, sqrt(343/N^6 + 7) = y}
*/

  int n = argc > 1 ? (atoi(argv[1])+3)/4+3 : 360;  /* 360 default number of pi digits */

  unsigned short *pi = (unsigned short*) malloc(n * sizeof(unsigned short));
  div_t d;
  int i, j, t;

  /* pi = 4  */
  memset(pi, 0, n*sizeof(unsigned short));
  pi[1]=4;

  /* for i = B down to 1 */
  for (i=(int)(3.322*4*n); i>0; --i) {

    /* pi *= i; */
    t = 0;
    for (j=n-1; j>=0; --j) {  /* pi *= i; */
      t += pi[j] * i;
      pi[j] = t % 10000;
      t /= 10000;
    }

    /* pi /= (2*i+1) */
    d.quot = d.rem = 0;
    for (j=0; j<n; ++j) {  /* pi /= 2*i+1; */
      d = div(pi[j]+d.rem*10000, i+i+1);
      pi[j] = d.quot;
    }

    /* pi += 2 */
    pi[1] += 2;
  }

  print(pi, n);
  return 0;
}

