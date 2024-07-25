add_library(
    loader STATIC
    ${ATHCON_INCLUDE_DIR}/athcon/loader.h
    loader.c
)

add_library(athcon::loader ALIAS loader)
set_target_properties(loader PROPERTIES
    OUTPUT_NAME athcon-loader
    POSITION_INDEPENDENT_CODE TRUE
)
target_link_libraries(loader INTERFACE ${CMAKE_DL_LIBS} PUBLIC athcon::athcon)

if(ATHCON_INSTALL)
    install(TARGETS loader EXPORT athcon DESTINATION ${CMAKE_INSTALL_LIBDIR})
endif()
