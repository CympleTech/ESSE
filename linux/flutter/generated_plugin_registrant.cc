//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <esse_core/esse_core_plugin.h>
#include <file_selector_linux/file_selector_plugin.h>
#include <pasteboard/pasteboard_plugin.h>
#include <record_linux/record_linux_plugin.h>
#include <screen_retriever/screen_retriever_plugin.h>
#include <url_launcher_linux/url_launcher_plugin.h>
#include <window_manager/window_manager_plugin.h>

void fl_register_plugins(FlPluginRegistry* registry) {
  g_autoptr(FlPluginRegistrar) esse_core_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "EsseCorePlugin");
  esse_core_plugin_register_with_registrar(esse_core_registrar);
  g_autoptr(FlPluginRegistrar) file_selector_linux_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "FileSelectorPlugin");
  file_selector_plugin_register_with_registrar(file_selector_linux_registrar);
  g_autoptr(FlPluginRegistrar) pasteboard_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "PasteboardPlugin");
  pasteboard_plugin_register_with_registrar(pasteboard_registrar);
  g_autoptr(FlPluginRegistrar) record_linux_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "RecordLinuxPlugin");
  record_linux_plugin_register_with_registrar(record_linux_registrar);
  g_autoptr(FlPluginRegistrar) screen_retriever_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "ScreenRetrieverPlugin");
  screen_retriever_plugin_register_with_registrar(screen_retriever_registrar);
  g_autoptr(FlPluginRegistrar) url_launcher_linux_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "UrlLauncherPlugin");
  url_launcher_plugin_register_with_registrar(url_launcher_linux_registrar);
  g_autoptr(FlPluginRegistrar) window_manager_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "WindowManagerPlugin");
  window_manager_plugin_register_with_registrar(window_manager_registrar);
}
