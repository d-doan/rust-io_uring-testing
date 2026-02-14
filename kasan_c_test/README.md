## KASAN Testing
Small program to test that KASAN is enabled on the kernel by running the following commands

```bash
make
# insert module
sudo insmod kasan_test.ko

# displays KASAN info
sudo dmesg | tail -100

# should display similar to below
[  123.596831] The buggy address belongs to the physical page:
[  123.596834] page:000000003e1e29bd refcount:1 mapcount:0 mapping:0000000000000000 index:0xffff88810460c208 pfn:0x10460c
[  123.596838] flags: 0x17ffffc0000200(slab|node=0|zone=2|lastcpupid=0x1fffff)
[  123.596844] raw: 0017ffffc0000200 ffffea0004115cc0 dead000000000002 ffff888100042280
[  123.596848] raw: ffff88810460c208 0000000080660024 00000001ffffffff 0000000000000000
[  123.596850] page dumped because: kasan: bad access detected

[  123.596853] Memory state around the buggy address:
[  123.596855]  ffff88810460cc80: fb fc fc fc fc fb fc fc fc fc fb fc fc fc fc fb
[  123.596858]  ffff88810460cd00: fc fc fc fc 00 fc fc fc fc fb fc fc fc fc 00 fc
[  123.596861] >ffff88810460cd80: fc fc fc fb fc fc fc fc fb fc fc fc fc 06 fc fc
[  123.596864]                    ^
[  123.596866]  ffff88810460ce00: fc fc fb fc fc fc fc 00 fc fc fc fc 00 fc fc fc
[  123.596869]  ffff88810460ce80: fc 00 fc fc fc fc fb fc fc fc fc 00 fc fc fc fc
[  123.596872] ==================================================================
[  123.596874] Disabling lock debugging due to kernel taint
[  123.596875] out of bounds write


# used to remove module if needed to reload
sudo rmmod kasan_test
```
