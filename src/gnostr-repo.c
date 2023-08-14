// C program to create .gnostr folder
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

int main()
{
	int check;
	char* dirname = ".gnostr";
	check = mkdir(dirname,0777);
	if (!check)
		printf(".gnostr created\n");
        //more
	else {
		printf("Unable to create .gnostr\n");
		exit(1);
	}
	system("mkdir -p .gnostr");
}


