#ifndef __DIRENT_H
#define __DIRENT_H

#include <sys/types.h>

typedef struct dirent {
	ino_t d_ino;
	char d_name[256];
} dirent;

typedef struct __dirstream DIR;

DIR* opendir(const char *);
dirent* readdir(DIR *);

#endif //__DIRENT_H
