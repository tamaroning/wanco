#include "lz4/lz4.h"
#include "snapshot.h"
#include "snapshot.pb.h"
#include "wanco.h"
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <google/protobuf/util/json_util.h>
#include <ios>
#include <ostream>
#include <string>
#include <utility>
#include <vector>

namespace wanco {

static auto decode_value_proto(const chkpt::Value &v) -> wanco::Value {
  switch (v.type()) {
  case chkpt::Type::I32: {
    int32_t const i32 = v.i32();
    return {i32};
  } break;
  case chkpt::Type::I64: {
    int64_t const i64 = v.i64();
    return {i64};
  } break;
  case chkpt::Type::F32: {
    float const f32 = v.f32();
    return {f32};
  } break;
  case chkpt::Type::F64: {
    double const f64 = v.f64();
    return {f64};
  } break;
  default:
    ASSERT(false && "Invalid type");
    return {0};
  }
}

static auto decode_frame_proto(const chkpt::Frame &f) -> wanco::Frame {
  wanco::Frame frame;
  frame.fn_index = f.fn_idx();
  frame.pc = f.pc();

  for (const auto &l : f.locals()) {
    wanco::Value const v = decode_value_proto(l);
    frame.locals.push_back(v);
  }

  for (const auto &s : f.stack()) {
    wanco::Value const v = decode_value_proto(s);
    frame.stack.push_back(v);
  }

  return frame;
}

auto decode_checkpoint_proto(std::ifstream &f)
    -> std::pair<wanco::Checkpoint, int8_t *> {
  GOOGLE_PROTOBUF_VERIFY_VERSION;
  Checkpoint ret;
  chkpt::Checkpoint buf;
  if (!buf.ParseFromIstream(&f)) {
    Fatal() << "Failed to parse checkpoint file (protobuf)" << '\n';
    if (f.eof()) {
      Fatal() << "Error: Reached end of file unexpectedly." << '\n';
    }
    if (f.fail()) {
      Fatal() << "Error: Logical error on input stream." << '\n';
    }
    if (f.bad()) {
      Fatal() << "Error: Read/write error on input stream." << '\n';
    }
    Info() << "Debug information for the checkpoint buffer:" << '\n';
    Info() << buf.DebugString() << '\n';
    exit(1);
  }

  for (const auto &fr : buf.frames()) {
    wanco::Frame const frame = decode_frame_proto(fr);
    ret.frames.push_front(frame);
  }

  for (const auto &g : buf.globals()) {
    wanco::Value const v = decode_value_proto(g);
    ret.globals.push_back(v);
  }

  for (const auto &t : buf.table()) {
    ret.table.push_back(t);
  }

  ret.memory_size = buf.memory_size();
  int8_t *memory_base = allocate_memory(ret.memory_size);

  if (USE_LZ4) {
    Info() << "Decompressing memory: " << std::dec << ret.memory_size
           << " pages (" << ret.memory_size * PAGE_SIZE << " bytes)" << '\n';
    const std::string &compressed = buf.memory_lz4();
    size_t const size = LZ4_decompress_safe(
        compressed.data(), reinterpret_cast<char *>(memory_base),
        compressed.size(), ret.memory_size * PAGE_SIZE);
    if (size < 0) {
      Fatal() << "Failed to decompress memory" << '\n';
      exit(1);
    }
  } else {
    ASSERT(buf.memory().size() == (std::size_t)ret.memory_size * PAGE_SIZE);
    Info() << "Copying memory: " << std::dec << ret.memory_size << " pages ("
           << buf.memory().size() << " bytes)" << '\n';
    memcpy(memory_base, buf.memory().data(), buf.memory().size());
  }

  return {ret, memory_base};
}

static auto encode_value_proto(const wanco::Value &v) -> chkpt::Value {
  chkpt::Value ret;
  switch (v.get_type()) {
  case wanco::Value::Type::I32:
    ret.set_type(chkpt::Type::I32);
    ret.set_i32(v.i32);
    break;
  case wanco::Value::Type::I64:
    ret.set_type(chkpt::Type::I64);
    ret.set_i64(v.i64);
    break;
  case wanco::Value::Type::F32:
    ret.set_type(chkpt::Type::F32);
    ret.set_f32(v.f32);
    break;
  case wanco::Value::Type::F64:
    ret.set_type(chkpt::Type::F64);
    ret.set_f64(v.f64);
    break;
  default:
    ASSERT(false && "Invalid type");
  }
  return ret;
}

static auto encode_frame_proto(const wanco::Frame &f) -> chkpt::Frame {
  chkpt::Frame ret;
  ret.set_fn_idx(f.fn_index);
  ret.set_pc(f.pc);

  for (const auto &l : f.locals) {
    chkpt::Value const v = encode_value_proto(l);
    ret.add_locals()->CopyFrom(v);
  }

  for (const auto &s : f.stack) {
    chkpt::Value const v = encode_value_proto(s);
    ret.add_stack()->CopyFrom(v);
  }

  return ret;
}

void encode_checkpoint_proto(std::ofstream &ofs, Checkpoint &chkpt,
                             int8_t *memory_base) {
  GOOGLE_PROTOBUF_VERIFY_VERSION;
  chkpt::Checkpoint buf;
  for (const auto &fr : chkpt.frames) {
    chkpt::Frame const f = encode_frame_proto(fr);
    buf.add_frames()->CopyFrom(f);
  }

  for (const auto &g : chkpt.globals) {
    chkpt::Value const v = encode_value_proto(g);
    buf.add_globals()->CopyFrom(v);
  }

  for (const auto &t : chkpt.table) {
    buf.add_table(t);
  }

  buf.set_memory_size(chkpt.memory_size);
  if constexpr (USE_LZ4) {
    uint64_t const time_ms =
        std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::system_clock::now().time_since_epoch())
            .count();
    Info() << "Compressing memory" << '\n';
    int const guarantee = LZ4_compressBound(chkpt.memory_size * PAGE_SIZE);
    std::vector<char> compressed;
    compressed.resize(guarantee);
    int const sz = LZ4_compress_default(
        reinterpret_cast<char *>(memory_base), compressed.data(),
        chkpt.memory_size * PAGE_SIZE, compressed.capacity());
    compressed.resize(sz);

    Info() << "Compression ratio: "
           << static_cast<double>(sz) / (chkpt.memory_size * PAGE_SIZE) << '\n';
    uint64_t const end_time_ms =
        std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::system_clock::now().time_since_epoch())
            .count();
    Info() << "Compression time: " << end_time_ms - time_ms << " ms" << '\n';

    buf.set_memory_lz4(std::string(compressed.begin(), compressed.end()));
  } else {
    Info() << "Copying memory" << '\n';
    buf.set_memory(memory_base, chkpt.memory_size * PAGE_SIZE);
  }

  if (!ofs.is_open()) {
    Fatal() << "Failed to open checkpoint file" << '\n';
    exit(1);
  }
  if (!buf.SerializeToOstream(&ofs)) {
    Fatal() << "Failed to write checkpoint file" << '\n';
    exit(1);
  }

  // write out pb.json for debugging
  if constexpr (DEBUG_ENABLED) {
    google::protobuf::util::JsonPrintOptions options;
    options.add_whitespace = true;
    options.always_print_primitive_fields = true;
    std::string json;
    google::protobuf::util::MessageToJsonString(buf, &json, options);
    std::ofstream json_ofs("checkpoint.pb.json");
    json_ofs << json;
    Info() << "Wrote JSON vesion to checkpoint.pb.json" << '\n';
  }
}

} // namespace wanco
