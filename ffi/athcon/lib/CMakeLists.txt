add_library(athcon INTERFACE)
add_library(athcon::athcon ALIAS athcon)
target_compile_features(athcon INTERFACE c_std_99)
target_include_directories(athcon INTERFACE $<BUILD_INTERFACE:${ATHCON_INCLUDE_DIR}>$<INSTALL_INTERFACE:include>)

add_library(athcon_cpp INTERFACE)
add_library(athcon::athcon_cpp ALIAS athcon_cpp)
target_compile_features(athcon_cpp INTERFACE cxx_std_17)
target_include_directories(athcon_cpp INTERFACE $<BUILD_INTERFACE:${ATHCON_INCLUDE_DIR}>$<INSTALL_INTERFACE:include>)
target_link_libraries(athcon_cpp INTERFACE athcon::athcon)

# add_subdirectory(instructions)
add_subdirectory(loader)
# add_subdirectory(mocked_host)
# add_subdirectory(tooling)

if(ATHCON_INSTALL)
    install(TARGETS athcon athcon_cpp EXPORT athconTargets)
endif()
