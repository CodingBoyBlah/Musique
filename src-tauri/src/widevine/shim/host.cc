// Native half of the Widevine CDM host process.
//
// The CDM is a C++ library whose interface is Chromium's `cdm::ContentDecryptionModule_N`
// together with a `cdm::Host_N` the embedder supplies. This file loads the module, gives it a
// host, and exposes a flat C API (`wvh_*`) that src/widevine/host.rs drives. It runs only inside
// the `--widevine-host` child process, on a single thread: every CDM call and every host
// callback happen on the thread that calls into this file.
//
// The CDM answers most requests synchronously (the promise is resolved before the call
// returns). Where it does not, the request waits here, firing the timers the CDM asked for,
// until its promise settles or a deadline passes.
//
// Written for musique (MIT). The headers under cdm/ are Chromium's, under the BSD license in
// cdm/LICENSE.

#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <io.h>
#include <stdio.h>
#else
#include <dlfcn.h>
#include <fcntl.h>
#include <unistd.h>
#endif

#include "cdm/content_decryption_module.h"

#include <algorithm>
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <string>
#include <thread>
#include <vector>

static_assert(sizeof(cdm::SubsampleEntry) == 8, "subsample entries are two u32s");

namespace {

using Clock = std::chrono::steady_clock;

// ---------------------------------------------------------------------------------------------
// State the host callbacks write into.

struct Waiting {
  uint32_t promise = 0;
  bool settled = false;
  bool resolved = false;
  std::string session;  // set by OnResolveNewSessionPromise
  std::string error;    // set by OnRejectPromise
};

struct Timer {
  Clock::time_point due;
  void* context;
};

class Output;

struct State {
  bool init_settled = false;
  bool init_ok = false;
  uint32_t next_promise = 1;
  Waiting waiting;

  // The last license request the CDM produced, and the session it belongs to.
  bool have_message = false;
  std::string message_session;
  std::vector<uint8_t> message;

  // Keys reported usable by the last OnSessionKeysChange.
  uint32_t usable_keys = 0;
  bool keys_changed = false;

  std::vector<Timer> timers;
  Output* spare = nullptr;  // one recycled output buffer
};

State g_state;

// Output buffers the CDM fills. One is kept for reuse so the per-sample decrypt path does not
// allocate once it has warmed up.
class Output final : public cdm::Buffer {
 public:
  explicit Output(uint32_t capacity) : bytes_(capacity), size_(0) {}
  ~Output() override = default;

  void Destroy() override {
    if (!g_state.spare) {
      size_ = 0;
      g_state.spare = this;
    } else {
      delete this;
    }
  }
  uint32_t Capacity() const override { return static_cast<uint32_t>(bytes_.size()); }
  uint8_t* Data() override { return bytes_.data(); }
  void SetSize(uint32_t size) override { size_ = std::min(size, Capacity()); }
  uint32_t Size() const override { return size_; }

  void grow(uint32_t capacity) {
    if (bytes_.size() < capacity) bytes_.resize(capacity);
  }

 private:
  std::vector<uint8_t> bytes_;
  uint32_t size_;
};

cdm::Buffer* allocate(uint32_t capacity) {
  if (Output* reused = g_state.spare) {
    g_state.spare = nullptr;
    reused->grow(capacity);
    return reused;
  }
  return new Output(capacity);
}

void settle(uint32_t promise, bool ok, const char* session, uint32_t session_size,
            const std::string& error) {
  Waiting& w = g_state.waiting;
  if (w.promise != promise || w.settled) return;
  w.settled = true;
  w.resolved = ok;
  if (session && session_size) w.session.assign(session, session_size);
  w.error = error;
}

// ---------------------------------------------------------------------------------------------
// The host interface. Host_10 and Host_11 share every callback; Host_11 adds ReportMetrics.

#define WVH_HOST_CALLBACKS                                                                      \
  cdm::Buffer* Allocate(uint32_t capacity) override { return allocate(capacity); }              \
  void SetTimer(int64_t delay_ms, void* context) override {                                     \
    g_state.timers.push_back(                                                                   \
        {Clock::now() + std::chrono::milliseconds(delay_ms < 0 ? 0 : delay_ms), context});      \
  }                                                                                             \
  cdm::Time GetCurrentWallTime() override {                                                     \
    using namespace std::chrono;                                                                \
    return duration_cast<duration<double>>(system_clock::now().time_since_epoch()).count();     \
  }                                                                                             \
  void OnInitialized(bool success) override {                                                   \
    g_state.init_settled = true;                                                                \
    g_state.init_ok = success;                                                                  \
  }                                                                                             \
  void OnResolveKeyStatusPromise(uint32_t promise, cdm::KeyStatus) override {                   \
    settle(promise, true, nullptr, 0, std::string());                                           \
  }                                                                                             \
  void OnResolveNewSessionPromise(uint32_t promise, const char* id, uint32_t size) override {   \
    settle(promise, id != nullptr, id, size, id ? std::string() : "no session was created");    \
  }                                                                                             \
  void OnResolvePromise(uint32_t promise) override {                                            \
    settle(promise, true, nullptr, 0, std::string());                                           \
  }                                                                                             \
  void OnRejectPromise(uint32_t promise, cdm::Exception exception, uint32_t code,               \
                       const char* text, uint32_t size) override {                              \
    std::string why = "the module refused (exception " +                                        \
                      std::to_string(static_cast<uint32_t>(exception)) + ", code " +            \
                      std::to_string(code) + ")";                                               \
    if (text && size) why += ": " + std::string(text, size);                                    \
    settle(promise, false, nullptr, 0, why);                                                    \
  }                                                                                             \
  void OnSessionMessage(const char* id, uint32_t id_size, cdm::MessageType type,                \
                        const char* message, uint32_t size) override {                          \
    if (type != cdm::kLicenseRequest) return;                                                   \
    g_state.have_message = true;                                                                \
    g_state.message_session.assign(id ? id : "", id ? id_size : 0);                             \
    g_state.message.assign(reinterpret_cast<const uint8_t*>(message),                           \
                           reinterpret_cast<const uint8_t*>(message) + size);                   \
  }                                                                                             \
  void OnSessionKeysChange(const char*, uint32_t, bool, const cdm::KeyInformation* keys,        \
                           uint32_t count) override {                                           \
    uint32_t usable = 0;                                                                        \
    for (uint32_t i = 0; i < count; ++i)                                                        \
      if (keys[i].status == cdm::kUsable) ++usable;                                             \
    g_state.usable_keys = usable;                                                               \
    g_state.keys_changed = true;                                                                \
  }                                                                                             \
  void OnExpirationChange(const char*, uint32_t, cdm::Time) override {}                         \
  void OnSessionClosed(const char*, uint32_t) override {}                                       \
  void SendPlatformChallenge(const char*, uint32_t, const char*, uint32_t) override {}          \
  void EnableOutputProtection(uint32_t) override {}                                             \
  void QueryOutputProtectionStatus() override {                                                 \
    if (on_output_status) on_output_status();                                                   \
  }                                                                                             \
  void OnDeferredInitializationDone(cdm::StreamType, cdm::Status) override {}                   \
  cdm::FileIO* CreateFileIO(cdm::FileIOClient*) override { return nullptr; }                    \
  void RequestStorageId(uint32_t) override {                                                    \
    if (on_storage_id) on_storage_id();                                                         \
  }                                                                                             \
  void (*on_output_status)() = nullptr;                                                         \
  void (*on_storage_id)() = nullptr;

class Host10 final : public cdm::Host_10 {
 public:
  WVH_HOST_CALLBACKS
};

class Host11 final : public cdm::Host_11 {
 public:
  WVH_HOST_CALLBACKS
  void ReportMetrics(cdm::MetricName, uint64_t) override {}
};

// ---------------------------------------------------------------------------------------------
// The module instance, whichever interface version it was created with.

class Instance {
 public:
  virtual ~Instance() = default;
  virtual int version() const = 0;
  virtual void initialize() = 0;
  virtual void generate(uint32_t promise, const uint8_t* init, uint32_t size) = 0;
  virtual void update(uint32_t promise, const std::string& session, const uint8_t* license,
                      uint32_t size) = 0;
  virtual void close(uint32_t promise, const std::string& session) = 0;
  virtual cdm::Status decrypt(const cdm::InputBuffer_2& in, cdm::DecryptedBlock* out) = 0;
  virtual void timer(void* context) = 0;
  virtual void output_status() = 0;
  virtual void storage_id() = 0;
};

template <class Cdm, class HostImpl>
class Typed final : public Instance {
 public:
  Typed() = default;
  ~Typed() override {
    if (cdm_) cdm_->Destroy();
  }

  HostImpl host;

  void adopt(Cdm* cdm) { cdm_ = cdm; }
  int version() const override { return Cdm::kVersion; }
  void initialize() override {
    cdm_->Initialize(/*allow_distinctive_identifier=*/false, /*allow_persistent_state=*/false,
                     /*use_hw_secure_codecs=*/false);
  }
  void generate(uint32_t promise, const uint8_t* init, uint32_t size) override {
    cdm_->CreateSessionAndGenerateRequest(promise, cdm::kTemporary, cdm::kCenc, init, size);
  }
  void update(uint32_t promise, const std::string& session, const uint8_t* license,
              uint32_t size) override {
    cdm_->UpdateSession(promise, session.data(), static_cast<uint32_t>(session.size()), license,
                        size);
  }
  void close(uint32_t promise, const std::string& session) override {
    cdm_->CloseSession(promise, session.data(), static_cast<uint32_t>(session.size()));
  }
  cdm::Status decrypt(const cdm::InputBuffer_2& in, cdm::DecryptedBlock* out) override {
    return cdm_->Decrypt(in, out);
  }
  void timer(void* context) override { cdm_->TimerExpired(context); }
  void output_status() override {
    // No protected outputs: audio only, nothing to report beyond "none".
    cdm_->OnQueryOutputProtectionStatus(cdm::kQuerySucceeded, 0, 0);
  }
  void storage_id() override { cdm_->OnStorageId(0, nullptr, 0); }

 private:
  Cdm* cdm_ = nullptr;
};

Instance* g_instance = nullptr;
std::string g_module_version;

// The host callbacks that must answer back into the instance do so through these.
void answer_output_status() {
  if (g_instance) g_instance->output_status();
}
void answer_storage_id() {
  if (g_instance) g_instance->storage_id();
}

using CreateFn = void* (*)(int, const char*, uint32_t, GetCdmHostFunc, void*);

template <class Cdm, class HostImpl>
void* hand_host(int wanted, void* user_data) {
  if (wanted != HostImpl::kVersion) return nullptr;
  auto* typed = static_cast<Typed<Cdm, HostImpl>*>(user_data);
  return static_cast<typename Cdm::Host*>(&typed->host);
}

template <class Cdm, class HostImpl>
Instance* make(CreateFn create) {
  auto* typed = new Typed<Cdm, HostImpl>();
  typed->host.on_output_status = answer_output_status;
  typed->host.on_storage_id = answer_storage_id;
  static const char kKeySystem[] = "com.widevine.alpha";
  void* raw = create(Cdm::kVersion, kKeySystem, sizeof(kKeySystem) - 1,
                     &hand_host<Cdm, HostImpl>, typed);
  if (!raw) {
    delete typed;
    return nullptr;
  }
  typed->adopt(static_cast<Cdm*>(raw));
  return typed;
}

// ---------------------------------------------------------------------------------------------
// Timers and waiting.

int64_t next_timer_ms() {
  if (g_state.timers.empty()) return -1;
  auto soonest = std::min_element(g_state.timers.begin(), g_state.timers.end(),
                                  [](const Timer& a, const Timer& b) { return a.due < b.due; });
  auto left = std::chrono::duration_cast<std::chrono::milliseconds>(soonest->due - Clock::now());
  return left.count() < 0 ? 0 : left.count();
}

void fire_due_timers() {
  if (!g_instance || g_state.timers.empty()) return;
  auto now = Clock::now();
  std::vector<Timer> due;
  auto split = std::stable_partition(g_state.timers.begin(), g_state.timers.end(),
                                     [&](const Timer& t) { return t.due > now; });
  due.assign(split, g_state.timers.end());
  g_state.timers.erase(split, g_state.timers.end());
  for (const Timer& t : due) g_instance->timer(t.context);
}

// Fires timers until `done()` holds or `limit_ms` passes. Answers whether it holds.
template <class Done>
bool wait_for(Done done, int limit_ms) {
  auto deadline = Clock::now() + std::chrono::milliseconds(limit_ms);
  while (!done()) {
    int64_t next = next_timer_ms();
    if (next < 0) return done();  // nothing scheduled: nothing more will happen
    auto wake = Clock::now() + std::chrono::milliseconds(next);
    if (wake > deadline) return done();
    if (next > 0) std::this_thread::sleep_until(wake);
    fire_due_timers();
  }
  return true;
}

uint32_t begin_request() {
  g_state.waiting = Waiting();
  g_state.waiting.promise = g_state.next_promise++;
  return g_state.waiting.promise;
}

void write_error(char* err, uint32_t cap, const std::string& text) {
  if (!err || cap == 0) return;
  size_t n = std::min<size_t>(text.size(), cap - 1);
  memcpy(err, text.data(), n);
  err[n] = '\0';
}

// ---------------------------------------------------------------------------------------------
// Loading.

void* load_library(const char* path, std::string& why) {
#if defined(_WIN32)
  int wide_len = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, nullptr, 0);
  if (wide_len <= 0) {
    why = "the module path is not valid utf-8";
    return nullptr;
  }
  std::wstring wide(static_cast<size_t>(wide_len), L'\0');
  MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, &wide[0], wide_len);
  // Look for the module's own dependencies beside it, as a browser does.
  HMODULE lib = LoadLibraryExW(wide.c_str(), nullptr, LOAD_WITH_ALTERED_SEARCH_PATH);
  if (!lib) why = "cannot load the module (windows error " + std::to_string(GetLastError()) + ")";
  return reinterpret_cast<void*>(lib);
#else
  void* lib = dlopen(path, RTLD_NOW | RTLD_LOCAL);
  if (!lib) {
    const char* reason = dlerror();
    why = std::string("cannot load the module: ") + (reason ? reason : "unknown error");
  }
  return lib;
#endif
}

void* find_symbol(void* lib, const char* name) {
#if defined(_WIN32)
  return reinterpret_cast<void*>(GetProcAddress(reinterpret_cast<HMODULE>(lib), name));
#else
  return dlsym(lib, name);
#endif
}

// A DecryptedBlock living on the caller's stack.
class Block final : public cdm::DecryptedBlock {
 public:
  ~Block() override = default;
  void SetDecryptedBuffer(cdm::Buffer* buffer) override { buffer_ = buffer; }
  cdm::Buffer* DecryptedBuffer() override { return buffer_; }
  void SetTimestamp(int64_t timestamp) override { timestamp_ = timestamp; }
  int64_t Timestamp() const override { return timestamp_; }

 private:
  cdm::Buffer* buffer_ = nullptr;
  int64_t timestamp_ = 0;
};

}  // namespace

extern "C" {

// Moves the process's stdout out of the way so nothing the module prints can land in the IPC
// pipe: the original stdout is duplicated for the caller (a HANDLE on Windows, an fd on Unix)
// and stdout itself is pointed at stderr. 0 on success.
int wvh_take_stdout(intptr_t* ipc) {
#if defined(_WIN32)
  HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
  if (out == nullptr || out == INVALID_HANDLE_VALUE) return 1;
  HANDLE self = GetCurrentProcess();
  HANDLE copy = nullptr;
  if (!DuplicateHandle(self, out, self, &copy, 0, FALSE, DUPLICATE_SAME_ACCESS)) return 2;
  HANDLE err = GetStdHandle(STD_ERROR_HANDLE);
  SetStdHandle(STD_OUTPUT_HANDLE, err);
  // The C runtime keeps its own descriptor 1; point it at stderr too, when there is one.
  if (_fileno(stderr) >= 0 && _fileno(stdout) >= 0) _dup2(_fileno(stderr), _fileno(stdout));
  CloseHandle(out);
  *ipc = reinterpret_cast<intptr_t>(copy);
  return 0;
#else
  int copy = fcntl(1, F_DUPFD_CLOEXEC, 3);
  if (copy < 0) return 1;
  if (dup2(2, 1) < 0) {
    close(copy);
    return 2;
  }
  *ipc = static_cast<intptr_t>(copy);
  return 0;
#endif
}

// Loads and initializes the module at `path` (UTF-8). 0 on success; otherwise `err` says why.
int wvh_open(const char* path, char* err, uint32_t err_cap) {
  if (g_instance) return 0;
  std::string why;
  void* lib = load_library(path, why);
  if (!lib) {
    write_error(err, err_cap, why);
    return 1;
  }
  auto init = reinterpret_cast<void (*)()>(find_symbol(lib, "InitializeCdmModule_4"));
  auto create = reinterpret_cast<CreateFn>(find_symbol(lib, "CreateCdmInstance"));
  auto version = reinterpret_cast<const char* (*)()>(find_symbol(lib, "GetCdmVersion"));
  if (!init || !create) {
    write_error(err, err_cap, "the library is not a content decryption module");
    return 2;
  }
  if (version) {
    const char* v = version();
    if (v) g_module_version = v;
  }
  init();
  Instance* instance = make<cdm::ContentDecryptionModule_11, Host11>(create);
  if (!instance) instance = make<cdm::ContentDecryptionModule_10, Host10>(create);
  if (!instance) {
    write_error(err, err_cap, "the module offers no interface this host speaks (10 or 11)");
    return 3;
  }
  g_instance = instance;
  instance->initialize();
  if (!wait_for([] { return g_state.init_settled; }, 10000) || !g_state.init_ok) {
    write_error(err, err_cap, "the module did not initialize");
    delete g_instance;
    g_instance = nullptr;
    return 4;
  }
  return 0;
}

int wvh_interface_version(void) { return g_instance ? g_instance->version() : 0; }

const char* wvh_module_version(void) { return g_module_version.c_str(); }

// Milliseconds until the next timer the module set is due, or -1 when none is set.
int64_t wvh_next_timer_ms(void) { return next_timer_ms(); }

void wvh_fire_timers(void) { fire_due_timers(); }

// Opens a temporary session for CENC init data (a pssh box) and produces its license request.
// On success the request is at wvh_message() and the session id at wvh_session(), both valid
// until the next call into this file.
int wvh_generate(const uint8_t* init, uint32_t size, char* err, uint32_t err_cap) {
  if (!g_instance) {
    write_error(err, err_cap, "the module is not open");
    return 1;
  }
  g_state.have_message = false;
  g_state.message.clear();
  g_state.message_session.clear();
  uint32_t promise = begin_request();
  g_instance->generate(promise, init, size);
  bool done = wait_for(
      [] { return g_state.waiting.settled && (!g_state.waiting.resolved || g_state.have_message); },
      10000);
  if (!g_state.waiting.settled) {
    write_error(err, err_cap, "the module did not answer the request for a session");
    return 2;
  }
  if (!g_state.waiting.resolved) {
    write_error(err, err_cap, g_state.waiting.error);
    return 3;
  }
  if (!done || !g_state.have_message) {
    write_error(err, err_cap, "the module made no license request");
    return 4;
  }
  return 0;
}

const uint8_t* wvh_message(uint32_t* size) {
  *size = static_cast<uint32_t>(g_state.message.size());
  return g_state.message.data();
}

const char* wvh_session(uint32_t* size) {
  const std::string& id =
      g_state.waiting.session.empty() ? g_state.message_session : g_state.waiting.session;
  *size = static_cast<uint32_t>(id.size());
  return id.data();
}

// Gives a session its license. `usable` receives how many keys the module reports usable.
int wvh_update(const char* session, uint32_t session_size, const uint8_t* license, uint32_t size,
               uint32_t* usable, char* err, uint32_t err_cap) {
  if (!g_instance) {
    write_error(err, err_cap, "the module is not open");
    return 1;
  }
  std::string id(session, session_size);
  g_state.keys_changed = false;
  g_state.usable_keys = 0;
  uint32_t promise = begin_request();
  g_instance->update(promise, id, license, size);
  wait_for([] { return g_state.waiting.settled; }, 10000);
  if (!g_state.waiting.settled) {
    write_error(err, err_cap, "the module did not answer the license");
    return 2;
  }
  if (!g_state.waiting.resolved) {
    write_error(err, err_cap, g_state.waiting.error);
    return 3;
  }
  // Keys may be announced just after the promise; give the module its timers briefly.
  wait_for([] { return g_state.keys_changed; }, 500);
  *usable = g_state.usable_keys;
  return 0;
}

// Closes a session and drops its keys. 0 on success.
int wvh_close(const char* session, uint32_t session_size) {
  if (!g_instance) return 1;
  std::string id(session, session_size);
  uint32_t promise = begin_request();
  g_instance->close(promise, id);
  wait_for([] { return g_state.waiting.settled; }, 2000);
  return g_state.waiting.settled && g_state.waiting.resolved ? 0 : 2;
}

// Decrypts one CENC ('cenc', AES-CTR) sample of `size` bytes into `out`, which has room for
// `size` bytes. `subsamples` are `count` (clear, encrypted) pairs of u32; none means the whole
// sample is encrypted. Answers a cdm::Status (0 = success), or 100+ for a host-side failure.
int wvh_decrypt(const uint8_t* data, uint32_t size, const uint8_t* key_id, uint32_t key_id_size,
                const uint8_t* iv, uint32_t iv_size, const uint32_t* subsamples, uint32_t count,
                uint8_t* out) {
  if (!g_instance) return 100;
  cdm::InputBuffer_2 in;
  memset(&in, 0, sizeof(in));
  in.data = data;
  in.data_size = size;
  in.encryption_scheme = cdm::EncryptionScheme::kCenc;
  in.key_id = key_id;
  in.key_id_size = key_id_size;
  in.iv = iv;
  in.iv_size = iv_size;
  in.subsamples = count ? reinterpret_cast<const cdm::SubsampleEntry*>(subsamples) : nullptr;
  in.num_subsamples = count;
  Block block;
  cdm::Status status = g_instance->decrypt(in, &block);
  cdm::Buffer* buffer = block.DecryptedBuffer();
  int result = static_cast<int>(status);
  if (status == cdm::kSuccess) {
    if (!buffer) {
      result = 101;
    } else if (buffer->Size() != size) {
      result = 102;
    } else if (size) {
      memcpy(out, buffer->Data(), size);
    }
  }
  if (buffer) buffer->Destroy();
  return result;
}

}  // extern "C"
