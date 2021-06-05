MEMORY
{
  /* First 8KB used by bootloader */
  FLASH (rx) : ORIGIN = 0x00000000 + 8K, LENGTH = 128K - 8K

  /* Use this instead if you don't have a bootloader */
  /*FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 128K*/

  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 16K
}
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
