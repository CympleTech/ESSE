//
//  Generated file. Do not edit.
//

#include "generated_plugin_registrant.h"

#include <esse_core/esse_core_plugin.h>
#include <file_selector_windows/file_selector_plugin.h>
#include <url_launcher_windows/url_launcher_plugin.h>

void RegisterPlugins(flutter::PluginRegistry* registry) {
  EsseCorePluginRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("EsseCorePlugin"));
  FileSelectorPluginRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("FileSelectorPlugin"));
  UrlLauncherPluginRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("UrlLauncherPlugin"));
}
