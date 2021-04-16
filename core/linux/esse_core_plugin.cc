#include "include/esse_core/esse_core_plugin.h"
#include "share/esse.h"

#include <flutter_linux/flutter_linux.h>
#include <gtk/gtk.h>
#include <sys/utsname.h>

#include <cstring>

#define ESSE_CORE_PLUGIN(obj) \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), esse_core_plugin_get_type(), \
                              EsseCorePlugin))

struct _EsseCorePlugin {
  GObject parent_instance;
};

G_DEFINE_TYPE(EsseCorePlugin, esse_core_plugin, g_object_get_type())

gpointer daemon(gpointer pp)
{
  start((gchar*) pp);
  return NULL;
}

// Called when a method call is received from Flutter.
static void esse_core_plugin_handle_method_call(
    EsseCorePlugin* self,
    FlMethodCall* method_call) {
  g_autoptr(FlMethodResponse) response = nullptr;

  const gchar* method = fl_method_call_get_name(method_call);

  if (strcmp(method, "getPlatformVersion") == 0) {
    struct utsname uname_data = {};
    uname(&uname_data);
    g_autofree gchar *version = g_strdup_printf("Linux %s", uname_data.version);
    g_autoptr(FlValue) result = fl_value_new_string(version);
    response = FL_METHOD_RESPONSE(fl_method_success_response_new(result));
  } else if (strcmp(method, "daemon") == 0) {
    const gchar *path = fl_value_get_string(
        fl_value_lookup(
             fl_method_call_get_args(method_call),
             fl_value_new_string("path")
        )
    );

    int len = strlen(path);
    char *str;
    int i;
    str = (char*)malloc((len+1)*sizeof(char));
    for(i=0; i<len; i++){
      str[i] = *(path+i);
    }
    str[len] = '\0';
    gpointer pp = (gpointer) (gchar*) str;
    GThread *gthread = NULL;
    gthread = g_thread_new("daemon", daemon, pp);

    // TODO check gthread is ok.

    g_autoptr(FlValue) result = fl_value_new_string(path);
    response = FL_METHOD_RESPONSE(fl_method_success_response_new(result));
  } else {
    response = FL_METHOD_RESPONSE(fl_method_not_implemented_response_new());
  }

  fl_method_call_respond(method_call, response, nullptr);
}

static void esse_core_plugin_dispose(GObject* object) {
  G_OBJECT_CLASS(esse_core_plugin_parent_class)->dispose(object);
}

static void esse_core_plugin_class_init(EsseCorePluginClass* klass) {
  G_OBJECT_CLASS(klass)->dispose = esse_core_plugin_dispose;
}

static void esse_core_plugin_init(EsseCorePlugin* self) {}

static void method_call_cb(FlMethodChannel* channel, FlMethodCall* method_call,
                           gpointer user_data) {
  EsseCorePlugin* plugin = ESSE_CORE_PLUGIN(user_data);
  esse_core_plugin_handle_method_call(plugin, method_call);
}

void esse_core_plugin_register_with_registrar(FlPluginRegistrar* registrar) {
  EsseCorePlugin* plugin = ESSE_CORE_PLUGIN(
      g_object_new(esse_core_plugin_get_type(), nullptr));

  g_autoptr(FlStandardMethodCodec) codec = fl_standard_method_codec_new();
  g_autoptr(FlMethodChannel) channel =
      fl_method_channel_new(fl_plugin_registrar_get_messenger(registrar),
                            "esse_core",
                            FL_METHOD_CODEC(codec));
  fl_method_channel_set_method_call_handler(channel, method_call_cb,
                                            g_object_ref(plugin),
                                            g_object_unref);

  g_object_unref(plugin);
}
