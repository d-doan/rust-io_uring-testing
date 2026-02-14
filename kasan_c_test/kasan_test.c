#include <linux/init.h>
#include <linux/module.h>
#include <linux/slab.h>

// simple program to trigger KASAN

static int __init kasan_test_init(void)
{
    char *buf;

    buf = kmalloc(8, GFP_KERNEL);
    pr_info("allocated 8 bytes of kernel heap memory\n");

    // out of bounds write should trigger KASAN
    buf[16] = 0x42;

    pr_info("out of bounds write\n");

    kfree(buf);
    return 0;
}

static void __exit kasan_test_exit(void)
{
    pr_info("KASAN test module exit\n");
}

module_init(kasan_test_init);
module_exit(kasan_test_exit);

MODULE_LICENSE("GPL");
