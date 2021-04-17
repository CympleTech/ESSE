cmake_minimum_required(VERSION 3.10)
set(PROJECT_NAME "esse_core")
project(${PROJECT_NAME} LANGUAGES CXX)

# This value is used when generating builds using this plugin, so it must
# not be changed
set(PLUGIN_NAME "esse_core_plugin")

add_library(${PLUGIN_NAME} SHARED
  "esse_core_plugin.cc"
)
apply_standard_settings(${PLUGIN_NAME})
set_target_properties(${PLUGIN_NAME} PROPERTIES
  CXX_VISIBILITY_PRESET hidden)
target_compile_definitions(${PLUGIN_NAME} PRIVATE FLUTTER_PLUGIN_IMPL)
target_include_directories(${PLUGIN_NAME} INTERFACE
  "${CMAKE_CURRENT_SOURCE_DIR}/include")
target_include_directories(${PLUGIN_NAME} INTERFACE
  "${CMAKE_CURRENT_SOURCE_DIR}/share")

target_link_libraries(${PLUGIN_NAME} PRIVATE flutter)
target_link_libraries(${PLUGIN_NAME} PRIVATE PkgConfig::GTK)

## First Time & Debug To Lanuch install libraries.
target_link_directories(${PLUGIN_NAME} PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/share")
target_link_libraries(${PLUGIN_NAME} PRIVATE libesse.a)

## Secound Time, because libraries had in lib, and can found.
#find_library(ESSE_LIB NAMES libesse.so esse)
#target_link_libraries(${PLUGIN_NAME} PRIVATE ${ESSE_LIB})

# List of absolute paths to libraries that should be bundled with the plugin
set(esse_core_bundled_libraries
  "${CMAKE_CURRENT_SOURCE_DIR}/share/libesse.a"
  PARENT_SCOPE
)
