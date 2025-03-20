// Include shared lib
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include "./cpixelsortery.h"

uint32_t *data;
int width, height, maxColorValue;

int readHeader(FILE* in, int* w, int*  h, int* max){

	// read the PPM metadata (magic number, width, height, max color value)
	char magicnumber [3];
	fgets(magicnumber,3, in);
	if (magicnumber[0] != 'P' || magicnumber[1] != '6')
		return -1;

	// the newline, very messy code here, yes
	fgetc(in);

	char line[20];
	// We scan the width, height and max from their lines
	fgets(line, 20, in);
	printf("%s", line);
	sscanf(line,"%d%d", w, h);
	fgets(line, 20, in);
	sscanf(line,"%d", max);

	return 0;
}
int main (int argc, char *argv[])
{
	if(argc < 2 || argc > 3){
		printf("Usage: %s <input_path> [<output_path>]\n", argv[0]);
		return -1;
	}
	char cwd[4096];
	char cmdbuffer[4096];
	char* in_path = argv[1];
	char* tempin_path  = "/tmp/tempin.ppm";
	char* tempout_path = "/tmp/tempout.ppm";
	char* out_path;

	// Open file, must be ppm
	FILE* in = fopen(tempin_path, "r");
	FILE* out = fopen(tempout_path, "w");

	// read from file
		// header
	readHeader(in, &width, &height, &maxColorValue);
		// Just Debugging, for fun
		printf("Width: %d\nHeight: %d\n", width, height);
		printf("Max: %d\n", maxColorValue);
		// data
	data = calloc(width*height, sizeof(uint32_t));
	for (int i = 0; i < width*height; i++) {
		uint32_t px = (fgetc(in) << 24)
					+ (fgetc(in) << 16)
					+ (fgetc(in) << 8)
					+ 0x000000ff;
		data[i] = px;
	}
	fclose(in);

	printf("Sorting...\n");
	// ##### Sorting ######
	swaylock_effect(data, width, height);
//	hilbertsort(data, width, height);
//	glitch(data, width, height);
	// ####################



	// writing to file
		// header
	fprintf(out, "P6\n");
	fprintf(out, "%d%1c%d\n", width, 0x20, height);
	fprintf(out, "%d\n", maxColorValue);
		// data
	for (int i = 0; i < width*height; i++) {
		uint32_t px = data[i];
		px = px >> 8; // alpha be gone
		char b = px; px = px >> 8;
		char g = px; px = px >> 8;
		char r = px;
		fprintf(out, "%c%c%c", r, g, b);
	}
	fclose(out);
	printf("Done.\n");
	return 0;
}

