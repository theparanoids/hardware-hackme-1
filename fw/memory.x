MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K - 256K /* Reserved for settings */
  CCM : ORIGIN = 0x10000000, LENGTH = 16K
  RAM : ORIGIN = 0x20000000, LENGTH = 32K /* 128K but reserve 3/4 of it for various payloads */
}

_stack_start = ORIGIN(CCM) + LENGTH(CCM);

_heap_size = ORIGIN(RAM) + LENGTH(RAM) - _sheap;
