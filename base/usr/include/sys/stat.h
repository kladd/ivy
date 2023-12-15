#ifndef __STAT_H
#define __STAT_H

#define S_IFMT  00170000
#define S_IFDIR 0040000

#define S_ISDIR(m) (((m) & S_IFMT) == S_IFDIR)

struct stat {
	mode_t st_mode;
	off_t st_size;
};

int fstat(int fildes, struct stat *buf);

#endif //__STAT_H
