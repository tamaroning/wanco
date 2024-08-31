#include <stdio.h>
#include <stdlib.h>
#include <dirent.h>

void list_directory(const char *path) {
    struct dirent *entry;
    DIR *dp = opendir(path);

    if (dp == NULL) {
        perror("opendir");
        return;
    }

    while ((entry = readdir(dp))) {
        printf("%s\n", entry->d_name);
    }

    closedir(dp);
}

int main(int argc, char *argv[]) {
    const char *path;

    if (argc > 1) {
        path = argv[1];
    } else {
        path = ".";
    }

    list_directory(path);
    return 0;
}
