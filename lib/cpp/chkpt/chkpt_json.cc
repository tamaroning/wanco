#include "chkpt.h"
#include "lz4/lz4.h"
#include "tobiaslocker/base64.h"
#include <cstddef>
#include <fstream>
#include <iostream>
#include <string_view>
#include "chkpt.h"
#include "lz4/lz4.h"
#include "nlohmann/json.h"
#include "tobiaslocker/base64.h"

namespace wanco {

using nlohmann::json;

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
      // function index
      ofs << "      \"fn_index\": " << frame.fn_index << ",\n";
      // instruction offset
      ofs << "      \"pc\": " << frame.pc << ",\n";
      // locals
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
      ofs << "      ],\n";

      // stack
      ofs << "      \"stack\": [\n";
      for (size_t i = 0; i < frame.stack.size (); i++)
	{
	  const Value &value = frame.stack[i];
	  ofs << "        ";
	  write_value_json (ofs, value);
	  if (i != frame.stack.size () - 1)
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

  // table
  ofs << "  \"table\": [";
  for (size_t i = 0; i < chkpt.table.size (); i++)
    {
      ofs << chkpt.table[i];
      if (i != chkpt.table.size () - 1)
	ofs << ", ";
    }
  ofs << "],\n";

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

static Value
decode_value_json (json &j)
{
  Value v = {0};
  std::string ty = j["type"];
  if (ty == "i32")
    {
      v = Value (j["value"].get<int32_t> ());
    }
  else if (ty == "i64")
    {
      v = Value (j["value"].get<int64_t> ());
    }
  else if (ty == "f32")
    {
      v = Value (j["value"].get<float> ());
    }
  else if (ty == "f64")
    {
      v = Value (j["value"].get<double> ());
    }
  else
    {
      ASSERT (false && "unreachable");
      //__builtin_unreachable();
    }
  return v;
}

Checkpoint
decode_checkpoint_json (std::ifstream &f)
{
  Checkpoint chkpt;
  json j = json::parse (f);

  for (auto &fr : j["frames"])
    {
      Frame frame;
      frame.fn_index = fr["fn_index"].get<int32_t> ();
      frame.pc = fr["pc"].get<int32_t> ();
      for (auto &v : fr["locals"])
	{
	  Value value = decode_value_json (v);
	  frame.locals.push_back (value);
	}

      for (auto &v : fr["stack"])
	{
	  Value value = decode_value_json (v);
	  frame.stack.push_back (value);
	}
      chkpt.frames.push_front (frame);
    }

  for (auto &g : j["globals"])
    {
      Value value = decode_value_json (g);
      chkpt.globals.push_back (value);
    }

  chkpt.table = j["table"].get<std::deque<int32_t>> ();

  chkpt.memory_size = j["memory-size"].get<int32_t> ();

  /*
  for (auto &m : j["memory"]) {
    chkpt.memory.push_back(m.get<uint8_t>());
  }
  */

  std::cerr << "[info] Decompressing memory" << std::endl;
  std::string base64 = j["memory-lz4"];
  std::string compressed = base64::from_base64 (base64);
  chkpt.memory.resize (chkpt.memory_size * PAGE_SIZE);
  size_t size
    = LZ4_decompress_safe (compressed.data (), (char *) chkpt.memory.data (),
			   compressed.size (), chkpt.memory.size ());
  ASSERT (size == chkpt.memory.size ());

  return chkpt;
}

} // namespace wanco
