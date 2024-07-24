# Adds a CMake test to check the given ATHCON VM implementation with the athcon-vmtester tool.
#
# athcon_add_vm_test(NAME <test_name> TARGET <vm>)
# - NAME argument specifies the name of the added test,
# - TARGET argument specifies the CMake target being a shared library with ATHCON VM implementation.
function(athcon_add_vm_test)
    if(NOT TARGET athcon::athcon-vmtester)
        message(FATAL_ERROR "The athcon-vmtester has not been installed with this ATHCON package")
    endif()

    cmake_parse_arguments("" "" NAME;TARGET "" ${ARGN})
    add_test(NAME ${_NAME} COMMAND athcon::athcon-vmtester $<TARGET_FILE:${_TARGET}>)
endfunction()
