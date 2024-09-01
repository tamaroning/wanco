#include "chkpt.h"
#include "lz4/lz4.h"
#include "tobiaslocker/base64.h"
#include <fstream>
#include <iostream>
#include <string_view>
#include "chkpt.h"

static void
write_value_json (std::ofstream &ofs, const Value v)
{
  ofs << "{ \"type\": \"";
  switch (v.get_type ())
    {
    case Value::Type::I32:
      ofs << "i32";
      break;
    case Value::Type::I64:
      ofs << "i64";
      break;
    case Value::Type::F32:
      ofs << "f32";
      break;
    case Value::Type::F64:
      ofs << "f64";
      break;
    }
  ofs << "\", \"value\": ";
  switch (v.get_type ())
    {
    case Value::Type::I32:
      ofs << v.i32;
      break;
    case Value::Type::I64:
      ofs << v.i64;
      break;
    case Value::Type::F32:
      ofs << v.f32;
      break;
    case Value::Type::F64:
      ofs << v.f64;
      break;
    }
  ofs << " }";
}

void
encode_checkpoint_json (std::ofstream &ofs, Checkpoint &chkpt)
{
  ofs << "{\n";
  ofs << "  \"version\": 1,\n";
  // frames
  ofs << "  \"frames\": [\n";
  for (size_t i = 0; i < chkpt.frames.size (); i++)
    {
      const Frame &frame = chkpt.frames[i];
      ofs << "    {\n";
      ofs << "      \"fn_index\": " << frame.fn_index << ",\n";
      ofs << "      \"pc\": " << frame.pc << ",\n";
      ofs << "      \"locals\": [\n";
      for (size_t j = 0; j < frame.locals.size (); j++)
	{
	  const Value &local = frame.locals[j];
	  ofs << "        ";
	  write_value_json (ofs, local);
	  if (j != frame.locals.size () - 1)
	    ofs << ",";
	  ofs << "\n";
	}
      ofs << "      ]\n";
      ofs << "    }";
      if (i != chkpt.frames.size () - 1)
	ofs << ",";
      ofs << "\n";
    }
  ofs << "  ],\n";
  // stack
  ofs << "  \"stack\": [\n";
  for (size_t i = 0; i < chkpt.stack.size (); i++)
    {
      const Value &value = chkpt.stack[i];
      ofs << "    ";
      write_value_json (ofs, value);
      if (i != chkpt.stack.size () - 1)
	ofs << ",";
      ofs << "\n";
    }
  ofs << "  ],\n";
  // globals
  ofs << "  \"globals\": [\n";
  for (size_t i = 0; i < chkpt.globals.size (); i++)
    {
      const Value &value = chkpt.globals[i];
      ofs << "    ";
      write_value_json (ofs, value);
      if (i != chkpt.globals.size () - 1)
	ofs << ",";
      ofs << "\n";
    }
  ofs << "  ],\n";
  // memory
  ofs << "  \"memory-size\": " << chkpt.memory_size << ",\n";

  /*
  ofs << "  \"memory\": [\n";
  for (size_t i = 0; i < chkpt.memory.size(); i++) {
    int8_t byte = chkpt.memory[i];
    if (i % 64 == 0)
      ofs << "    ";
    ofs << (int)byte;
    if (i != chkpt.memory.size() - 1)
      ofs << ",";
    if (i % 64 == 63)
      ofs << "\n";
  }
  ofs << "  ],\n";
  */

  std::cerr << "[info] Compressing memory" << std::endl;
  int guarantee = LZ4_compressBound (chkpt.memory.size ());
  std::vector<char> compressed;
  compressed.resize (guarantee);
  int sz
    = LZ4_compress_default ((char *) chkpt.memory.data (), compressed.data (),
			    chkpt.memory.size (), compressed.capacity ());
  compressed.resize (sz);

  std::cout << "[info] compression ratio: "
	    << (double) sz / chkpt.memory.size () << std::endl;

  auto base64 = base64::to_base64 (
    std::string_view (compressed.data (), compressed.size ()));
  ofs << "  \"memory-lz4\": \"" << base64 << "\"\n";

  ofs << "}\n";
}
