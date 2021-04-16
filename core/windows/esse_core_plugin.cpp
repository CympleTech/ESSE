#include "include/esse_core/esse_core_plugin.h"
#include "share/esse.h"

// This must be included before many other Windows headers.
#include <windows.h>

// For getPlatformVersion; remove unless needed for your plugin implementation.
#include <VersionHelpers.h>

#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>
#include <flutter/standard_method_codec.h>

#include <map>
#include <memory>
#include <sstream>
#include <thread>

namespace {

using flutter::EncodableMap;
using flutter::EncodableValue;

static void StartDaemon(std::string path)
{
    start(path.c_str());
}

class EsseCorePlugin : public flutter::Plugin {
 public:
  static void RegisterWithRegistrar(flutter::PluginRegistrarWindows *registrar);

  EsseCorePlugin();

  virtual ~EsseCorePlugin();

 private:
  // Called when a method is called on this plugin's channel from Dart.
  void HandleMethodCall(
      const flutter::MethodCall<flutter::EncodableValue> &method_call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result);
};

// static
void EsseCorePlugin::RegisterWithRegistrar(
    flutter::PluginRegistrarWindows *registrar) {
  auto channel =
      std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
          registrar->messenger(), "esse_core",
          &flutter::StandardMethodCodec::GetInstance());

  auto plugin = std::make_unique<EsseCorePlugin>();

  channel->SetMethodCallHandler(
      [plugin_pointer = plugin.get()](const auto &call, auto result) {
        plugin_pointer->HandleMethodCall(call, std::move(result));
      });

  registrar->AddPlugin(std::move(plugin));
}

EsseCorePlugin::EsseCorePlugin() {}

EsseCorePlugin::~EsseCorePlugin() {}

void EsseCorePlugin::HandleMethodCall(
    const flutter::MethodCall<flutter::EncodableValue> &method_call,
    std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
  if (method_call.method_name().compare("getPlatformVersion") == 0) {
    std::ostringstream version_stream;
    version_stream << "Windows ";
    if (IsWindows10OrGreater()) {
      version_stream << "10+";
    } else if (IsWindows8OrGreater()) {
      version_stream << "8";
    } else if (IsWindows7OrGreater()) {
      version_stream << "7";
    }
    result->Success(flutter::EncodableValue(version_stream.str()));
  } else if (method_call.method_name().compare("daemon") == 0) {
    std::string path;
    const auto* arguments = std::get_if<EncodableMap>(method_call.arguments());
    if (arguments) {
      auto path_it = arguments->find(EncodableValue("path"));
      if (path_it != arguments->end()) {
        path = std::get<std::string>(path_it->second);
      }
    }
    if (path.empty()) {
      result->Success(flutter::EncodableValue("Missing path"));
      return;
    }
    auto thread1 = std::thread(StartDaemon, path);
    thread1.detach();
    result->Success(flutter::EncodableValue("Daemon success"));
  } else {
    result->NotImplemented();
  }
}

}  // namespace

void EsseCorePluginRegisterWithRegistrar(
    FlutterDesktopPluginRegistrarRef registrar) {
  EsseCorePlugin::RegisterWithRegistrar(
      flutter::PluginRegistrarManager::GetInstance()
          ->GetRegistrar<flutter::PluginRegistrarWindows>(registrar));
}
