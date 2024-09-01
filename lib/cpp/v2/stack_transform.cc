#include "stack_transform.h"
#include <stdio.h>
#include <libunwind.h>
#include <dlfcn.h>
#include <iostream>
#include <vector>

std::vector<FrameV2>
get_stack_trace ()
{
  unw_cursor_t cursor;
  unw_context_t context;

  unw_getcontext (&context);
  unw_init_local (&cursor, &context);

  std::vector<FrameV2> frames;
  printf ("Stack trace:\n");
  while (unw_step (&cursor) > 0)
    {
      unw_word_t offset, pc;
      char function_name[256];
      unw_word_t sp;

      unw_get_reg (&cursor, UNW_REG_IP, &pc);
      unw_get_reg (&cursor, UNW_REG_SP, &sp);
      if (unw_get_proc_name (&cursor, function_name, sizeof (function_name),
			     &offset)
	  != 0)
	std::cout << "Error: unable to obtain function name for address"
		  << std::endl;

      std::string name = function_name;
      if (name == "aot_main")
	break;

      printf ("0x%lx : (%s+0x%lx) [SP=0x%lx]\n", pc, function_name, offset, sp);

      frames.push_back (FrameV2{name, offset, sp});
    }

  return frames;
}
