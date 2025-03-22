#include <stdio.h>
#include <stdint.h>
#include <unistd.h>
#include <poll.h>
#include <sys/eventfd.h>

int main() {
    int efd = eventfd(0, 0);  // eventfd作成
    if (efd == -1) {
        perror("eventfd");
        return 1;
    }

    struct pollfd pfd = { .fd = efd, .events = POLLIN };

    printf("Waiting for event...\n");

    if (fork() == 0) {  // 子プロセスで通知
        sleep(1);
        uint64_t u = 1;
        write(efd, &u, sizeof(u));  // カウンタを増やす
        return 0;
    }

    poll(&pfd, 1, -1);  // イベント待機
    uint64_t value;
    read(efd, &value, sizeof(value));  // カウンタをリセット
    printf("Event received!\n");

    close(efd);
    return 0;
}
