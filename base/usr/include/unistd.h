#ifndef __UNISTD_H
#define __UNISTD_H

#define STDIN_FILENO 0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

int chdir(const char *path);
char *getcwd(char *buf, size_t size);
ssize_t read(int fildes, void *buf, size_t nbyte);
ssize_t write(int fildes, const void * buf, size_t nbyte);

#endif // __UNISTD_H
