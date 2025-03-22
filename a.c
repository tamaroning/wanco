#include <execinfo.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <sys/ucontext.h>
#include <unistd.h>

int *safepoint = NULL;

// シグナルハンドラ
void signal_segv_handler(int signo, siginfo_t *info, void *context) {
  void *buffer[1000];
  int nptrs;
  ucontext_t *ucontext;
  ucontext = (ucontext_t *)context;

  nptrs = backtrace(buffer, sizeof(buffer) / sizeof(void *));

  // バックトレースを標準出力に表示
  backtrace_symbols_fd(buffer, nptrs, fileno(stdout));
  exit(0);
}

void signal_usr1_handler(int signo) { mprotect(safepoint, 4096, PROT_NONE); }

// スレッドの処理
void cause_segfault() {
  while (1) {
    int hoge = *safepoint;
    printf("hoge: %d\n", hoge);
    sleep(1);
  }
}

int main() {
  // mmap
  safepoint = mmap(NULL, 4096, PROT_READ | PROT_WRITE,
                   MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
  *safepoint = 42;

  struct sigaction sa;

  sa.sa_sigaction = signal_segv_handler;
  sa.sa_flags = SA_SIGINFO;
  sigaction(SIGSEGV, &sa, NULL);

  signal(SIGUSR1, signal_usr1_handler);

  cause_segfault();

  return 0;
}
